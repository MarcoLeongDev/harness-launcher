//! Cross-version agent-preset compatibility for the shared harness home.
//!
//! rc engines shipped an agent preset named "code"; alpha engines renamed the
//! roster to "standard | ptc | minimal | cordis". The harness home (~/.dsh) is
//! shared across versions, so after an upgrade:
//!
//! * settings.yaml still records agent-presets.default: code — the alpha engine
//!   resolves that default for every NEW session and fails with
//!   'agent-presets: preset "code" not found'.
//! * Sessions created under rc record agentPreset: code in their projections;
//!   resuming any of them under alpha fails with the same error.
//!
//! Before an engine spawns this module reconciles the home with the presets
//! that engine actually ships: it rewrites a now-invalid default to an
//! available preset (preferring "standard", the successor of "code"), and
//! installs a user-root "code" shim (a copy of that engine's "standard"
//! composition) so rc-era sessions keep resuming. Nothing user-owned is ever
//! deleted; the engine's shipped presets always win over the shim.
//!
//! A shim installed by an older engine goes stale when a newer engine changes
//! the "standard" composition (e.g. the persona row switching from `text` to
//! the required `prefix`/`suffix` pair, which fails old copies at mount with
//! "$.prefix missing required value"). The shim therefore carries a `.launcher-shim`
//! marker holding a hash of the composition bytes installed; when the shim
//! still matches what the launcher installed but differs from what this engine
//! ships, it is refreshed in place (the replaced bytes are kept as
//! `agent.cordis.yml.bak`). A shim the user edited after install — or a
//! genuinely user-authored "code" preset — is never overwritten.
use std::path::{Path, PathBuf};

/// The preset id rc engines shipped and alpha engines removed.
pub const LEGACY_CODE_PRESET: &str = "code";
/// The alpha preset that succeeds "code" as the general-purpose default.
pub const STANDARD_PRESET: &str = "standard";
/// Composition file every preset directory must carry.
const COMPOSITION_FILE: &str = "agent.cordis.yml";
/// User-writable preset root under the harness home (USER_PRESET_DIR in
/// @deepseek-ai/dsh-agent-presets).
const USER_PRESET_DIR: &str = ".agent-presets";
/// Sidecar inside the shim dir recording the hash of the composition bytes
/// the launcher installed. Lets later spawns tell a stale launcher shim
/// (safe to refresh) from a user-edited or user-authored preset (hands off).
const SHIM_MARKER: &str = ".launcher-shim";
/// Backup of the composition bytes replaced by a shim refresh.
const SHIM_BACKUP: &str = "agent.cordis.yml.bak";
/// Header every launcher shim carries: it is a verbatim copy of a shipped
/// "standard" composition, which opens with this line. Marker-less shims from
/// before the marker existed are recognised by it (a user-authored "code"
/// preset would not open with another preset's header).
const SHIM_COPIED_HEADER: &[u8] = b"# The `standard` agent preset";

/// Stable FNV-1a 64-bit hash, hex-encoded. std's DefaultHasher is randomly
/// seeded per process and therefore unsuitable for a hash persisted to disk.
fn fnv1a64_hex(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}
/// Settings document at the harness home root.
const SETTINGS_FILE: &str = "settings.yaml";

/// The harness home: $DSH_HOME when set, otherwise ~/.dsh.
pub fn dsh_home() -> PathBuf {
    if let Some(home) = std::env::var_os("DSH_HOME") {
        if !home.is_empty() {
            return PathBuf::from(home);
        }
    }
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".dsh")
}

/// Roster layouts an installed version may ship, in scan order.
fn roster_roots(version_dir: &Path) -> [PathBuf; 2] {
    [
        version_dir
            .join("node_modules")
            .join("@deepseek-ai")
            .join("dsh-agent-presets")
            .join("presets"),
        version_dir
            .join("node_modules")
            .join("@deepseek-ai")
            .join("dsh")
            .join("config")
            .join("agent-presets"),
    ]
}

/// Every agent preset id an installed engine version ships. Covers both
/// roster layouts: alpha's "@deepseek-ai/dsh-agent-presets/presets/*" and
/// rc's "@deepseek-ai/dsh/config/agent-presets/*" (a directory counts only
/// when it carries the required composition file).
pub fn available_presets(version_dir: &Path) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    for root in roster_roots(version_dir) {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let dir = entry.path();
            if dir.is_dir() && dir.join(COMPOSITION_FILE).is_file() {
                if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                    if !ids.iter().any(|id| id == name) {
                        ids.push(name.to_string());
                    }
                }
            }
        }
    }
    ids.sort();
    ids
}

/// Pick the replacement default for a now-invalid recorded default.
fn successor_preset(old: &str, available: &[String]) -> Option<String> {
    if available.is_empty() {
        return None;
    }
    if old == LEGACY_CODE_PRESET && available.iter().any(|p| p == STANDARD_PRESET) {
        return Some(STANDARD_PRESET.to_string());
    }
    available.first().cloned()
}

/// Rewrite agent-presets.default in the harness home's settings.yaml when it
/// names a preset the engine does not ship. The document is preserved
/// byte-for-byte outside that one value; a missing file, block or value is
/// left alone (the engine's own configured default applies then, which is
/// always valid for that engine). Returns whether a repair was made.
fn fix_default_preset(
    home: &Path,
    available: &[String],
    log: &mut dyn FnMut(&str),
) -> Result<bool, String> {
    let path = home.join(SETTINGS_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(false);
    };
    let Some((value_start, value_len, value)) = find_default_entry(&text) else {
        return Ok(false);
    };
    let value = value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if available.contains(&value) {
        return Ok(false);
    }
    let Some(replacement) = successor_preset(&value, available) else {
        return Ok(false);
    };
    let mut repaired = String::with_capacity(text.len() + replacement.len());
    repaired.push_str(&text[..value_start]);
    repaired.push_str(&replacement);
    repaired.push_str(&text[value_start + value_len..]);
    write_atomic(&path, &repaired)?;
    log(&format!(
        "agent-presets.default was \"{value}\", which this engine does not ship — rewrote it to \"{replacement}\" so sessions can resolve it"
    ));
    Ok(true)
}

/// Locate the direct-child "default:" value inside the top-level
/// "agent-presets:" block of the settings document.
/// Returns (byte offset of the value, value byte length, value text).
fn find_default_entry(text: &str) -> Option<(usize, usize, String)> {
    let mut in_block = false;
    let mut offset = 0usize;
    for raw in text.split_inclusive('\n') {
        let trimmed = raw.trim_end_matches(['\n', '\r']);
        let indent_len = trimmed.len() - trimmed.trim_start().len();
        if indent_len == 0 {
            in_block = trimmed.trim_end() == "agent-presets:";
        } else if in_block {
            if let Some(pos) = trimmed.find("default:") {
                if trimmed[..pos].trim().is_empty() {
                    let after = &trimmed[pos + "default:".len()..];
                    let value = after.trim();
                    if !value.is_empty() && !value.starts_with('#') {
                        // Offset of the value within the line, then the document.
                        let in_line = trimmed.len() - value.len();
                        return Some((offset + in_line, value.len(), value.to_string()));
                    }
                }
            }
        }
        offset += raw.len();
    }
    None
}

/// Write via a sibling temp file + rename so a crash never truncates settings.
fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let tmp = path.with_extension("yaml.dsh-tmp");
    std::fs::write(&tmp, contents).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename {}: {e}", path.display()))
}

/// Ensure a user-root "code" preset exists so sessions recorded under rc
/// engines resume under an engine that no longer ships "code". The shim is a
/// copy of this engine's "standard" composition. Skipped entirely when the
/// engine ships "code" itself (the shipped root wins duplicate ids anyway).
/// An existing shim is refreshed when it is launcher-managed but stale — i.e.
/// it still matches what the launcher installed (per the marker) yet differs
/// from what this engine ships — because engine upgrades change the "standard"
/// composition and old copies fail at mount (e.g. persona `text` becoming the
/// required `prefix`/`suffix`). A shim the user edited after install, or a
/// genuinely user-authored "code" preset, is never overwritten. Returns
/// whether a shim was installed or refreshed.
fn ensure_legacy_code_shim(
    home: &Path,
    version_dir: &Path,
    available: &[String],
    log: &mut dyn FnMut(&str),
) -> Result<bool, String> {
    if available.iter().any(|p| p == LEGACY_CODE_PRESET) {
        return Ok(false);
    }
    if !available.iter().any(|p| p == STANDARD_PRESET) {
        return Ok(false);
    }
    let composition = available_composition_dir(version_dir, STANDARD_PRESET)
        .ok_or_else(|| format!("preset {STANDARD_PRESET} has no {COMPOSITION_FILE}"))?;
    let shipped =
        std::fs::read(&composition).map_err(|e| format!("read {}: {e}", composition.display()))?;
    let shim = home.join(USER_PRESET_DIR).join(LEGACY_CODE_PRESET);
    let target = shim.join(COMPOSITION_FILE);
    let marker = shim.join(SHIM_MARKER);
    let Ok(current) = std::fs::read(&target) else {
        std::fs::create_dir_all(&shim).map_err(|e| format!("mkdir {}: {e}", shim.display()))?;
        std::fs::write(&target, &shipped)
            .map_err(|e| format!("write {}: {e}", target.display()))?;
        write_shim_marker(&marker, &shipped)?;
        log(&format!(
            "installed legacy \"{LEGACY_CODE_PRESET}\" preset shim (copy of {STANDARD_PRESET}) so sessions created under older engines resume"
        ));
        return Ok(true);
    };
    if current == shipped {
        // In sync: adopt marker-less shims installed before the marker
        // existed so future drifts are recognised as refreshable.
        if !marker.is_file() {
            write_shim_marker(&marker, &shipped)?;
        }
        return Ok(false);
    }
    // Diverged from what this engine ships. Refresh only when the shim is
    // launcher-managed: either the marker still matches its bytes (installed
    // by the launcher and untouched since), or there is no marker yet but the
    // bytes open with the copied "standard" header (a pre-marker install).
    let marker_hash = std::fs::read_to_string(&marker)
        .ok()
        .map(|s| s.trim().to_string());
    let managed = marker_hash.as_deref() == Some(&fnv1a64_hex(&current))
        || (!marker.is_file() && current.starts_with(SHIM_COPIED_HEADER));
    if !managed {
        log(&format!(
            "legacy \"{LEGACY_CODE_PRESET}\" preset differs from this engine\u{2019}s \"{STANDARD_PRESET}\" but looks user-authored, leaving it alone (rc-era sessions on it may fail to resume)"
        ));
        return Ok(false);
    }
    let backup = shim.join(SHIM_BACKUP);
    std::fs::write(&backup, &current).map_err(|e| format!("write {}: {e}", backup.display()))?;
    std::fs::write(&target, &shipped).map_err(|e| format!("write {}: {e}", target.display()))?;
    write_shim_marker(&marker, &shipped)?;
    log(&format!(
        "refreshed stale legacy \"{LEGACY_CODE_PRESET}\" preset shim from this engine\u{2019}s \"{STANDARD_PRESET}\" (previous copy kept at {SHIM_BACKUP}) so older sessions resume"
    ));
    Ok(true)
}

/// Record the hash of freshly installed shim bytes in the marker sidecar.
fn write_shim_marker(marker: &Path, shipped: &[u8]) -> Result<(), String> {
    std::fs::write(marker, fnv1a64_hex(shipped))
        .map_err(|e| format!("write {}: {e}", marker.display()))
}

/// Find the composition file of preset `id` across both roster layouts.
fn available_composition_dir(version_dir: &Path, id: &str) -> Option<PathBuf> {
    for root in roster_roots(version_dir) {
        let candidate = root.join(id).join(COMPOSITION_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Reconcile the harness home with the presets an engine version ships.
/// Called before every engine spawn. Failures are logged but never block the
/// start: the engine may still work when its configured default is valid.
pub fn repair_preset_compatibility(version_dir: &Path, mut log: impl FnMut(&str)) {
    let available = available_presets(version_dir);
    if available.is_empty() {
        return;
    }
    let home = dsh_home();
    if let Err(e) = fix_default_preset(&home, &available, &mut log) {
        log(&format!("preset default repair failed: {e}"));
    }
    if let Err(e) = ensure_legacy_code_shim(&home, version_dir, &available, &mut log) {
        log(&format!("legacy preset shim failed: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("dsh-launcher-presets-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write_roster(base: &Path, ids: &[&str]) -> PathBuf {
        for id in ids {
            let dir = base.join(id);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(COMPOSITION_FILE), format!("# {id}")).unwrap();
        }
        base.to_path_buf()
    }

    fn alpha_style_version(root: &Path) -> PathBuf {
        write_roster(
            &root.join("node_modules/@deepseek-ai/dsh-agent-presets/presets"),
            &["standard", "ptc", "minimal", "cordis"],
        );
        root.to_path_buf()
    }

    fn rc_style_version(root: &Path) -> PathBuf {
        write_roster(
            &root.join("node_modules/@deepseek-ai/dsh/config/agent-presets"),
            &["code", "standard", "minimal", "cordis"],
        );
        root.to_path_buf()
    }

    #[test]
    fn enumerates_alpha_and_rc_rosters() {
        let tmp = TempDir::new("enumerate");
        let alpha = alpha_style_version(&tmp.0.join("alpha"));
        let rc = rc_style_version(&tmp.0.join("rc"));
        assert_eq!(
            available_presets(&alpha),
            vec!["cordis", "minimal", "ptc", "standard"]
        );
        assert_eq!(
            available_presets(&rc),
            vec!["code", "cordis", "minimal", "standard"]
        );
    }

    #[test]
    fn rewrites_stale_default_to_standard() {
        let tmp = TempDir::new("rewrite");
        let settings = tmp.0.join(SETTINGS_FILE);
        std::fs::write(
            &settings,
            "ui-theme:\n  preference: system\nagent-presets:\n  default: code\nui-conversation:\n  busyEnter: steer\n",
        )
        .unwrap();
        let available = vec![
            "cordis".to_string(),
            "minimal".to_string(),
            "ptc".to_string(),
            "standard".to_string(),
        ];
        let mut lines = Vec::new();
        let changed =
            fix_default_preset(&tmp.0, &available, &mut |l| lines.push(l.to_string())).unwrap();
        assert!(changed);
        let text = std::fs::read_to_string(&settings).unwrap();
        assert!(text.contains("default: standard"), "{text}");
        assert!(text.contains("busyEnter: steer"));
        assert!(!text.contains("default: code"));
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn keeps_default_when_engine_ships_it() {
        let tmp = TempDir::new("keep");
        std::fs::write(
            tmp.0.join(SETTINGS_FILE),
            "agent-presets:\n  default: code\n",
        )
        .unwrap();
        let available = vec!["code".to_string(), "standard".to_string()];
        let changed = fix_default_preset(&tmp.0, &available, &mut |_| {}).unwrap();
        assert!(!changed);
        assert!(
            std::fs::read_to_string(tmp.0.join(SETTINGS_FILE))
                .unwrap()
                .contains("default: code")
        );
    }

    #[test]
    fn missing_block_or_file_is_left_alone() {
        let tmp = TempDir::new("missing");
        // No settings file at all.
        let changed = fix_default_preset(&tmp.0, &["standard".to_string()], &mut |_| {}).unwrap();
        assert!(!changed);
        // File without an agent-presets block.
        std::fs::write(
            tmp.0.join(SETTINGS_FILE),
            "ui-theme:\n  preference: system\n",
        )
        .unwrap();
        let changed = fix_default_preset(&tmp.0, &["standard".to_string()], &mut |_| {}).unwrap();
        assert!(!changed);
    }

    #[test]
    fn repair_rewrites_quoted_values() {
        let tmp = TempDir::new("quoted");
        std::fs::write(
            tmp.0.join(SETTINGS_FILE),
            "agent-presets:\n  default: \"code\"\n",
        )
        .unwrap();
        let available = vec!["standard".to_string()];
        let changed = fix_default_preset(&tmp.0, &available, &mut |_| {}).unwrap();
        assert!(changed);
        let text = std::fs::read_to_string(tmp.0.join(SETTINGS_FILE)).unwrap();
        assert!(text.contains("default: standard"), "{text}");
    }

    #[test]
    fn shim_copies_standard_composition_and_never_overwrites() {
        let tmp = TempDir::new("shim");
        let home = TempDir::new("shim-home");
        let version = alpha_style_version(&tmp.0);
        let available = available_presets(&version);
        let mut lines = Vec::new();
        let made = ensure_legacy_code_shim(&home.0, &version, &available, &mut |l| {
            lines.push(l.to_string())
        })
        .unwrap();
        assert!(made);
        let shim = home
            .0
            .join(USER_PRESET_DIR)
            .join(LEGACY_CODE_PRESET)
            .join(COMPOSITION_FILE);
        assert!(shim.is_file());
        assert_eq!(std::fs::read_to_string(&shim).unwrap(), "# standard");
        // A second run must not clobber (even a user-edited) shim.
        std::fs::write(&shim, "# user edited").unwrap();
        let again = ensure_legacy_code_shim(&home.0, &version, &available, &mut |_| {}).unwrap();
        assert!(!again);
        assert!(
            std::fs::read_to_string(&shim)
                .unwrap()
                .contains("user edited")
        );
    }

    fn engine_with_standard(root: &Path, contents: &str) -> PathBuf {
        let dir = root.join("node_modules/@deepseek-ai/dsh-agent-presets/presets/standard");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(COMPOSITION_FILE), contents).unwrap();
        root.to_path_buf()
    }

    fn shim_files(home: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let dir = home.join(USER_PRESET_DIR).join(LEGACY_CODE_PRESET);
        (
            dir.join(COMPOSITION_FILE),
            dir.join(SHIM_MARKER),
            dir.join(SHIM_BACKUP),
        )
    }

    #[test]
    fn shim_refreshes_stale_managed_copy_when_standard_changes() {
        let tmp = TempDir::new("shim-refresh");
        let home = TempDir::new("shim-refresh-home");
        // v1 engine ships a standard composition with the old persona shape.
        let v1 = engine_with_standard(
            &tmp.0.join("v1"),
            "# The `standard` agent preset\n    text: old\n",
        );
        let available = available_presets(&v1);
        let mut lines = Vec::new();
        assert!(
            ensure_legacy_code_shim(&home.0, &v1, &available, &mut |l| lines.push(l.to_string()))
                .unwrap()
        );
        // v2 changes the composition (persona `text` -> required `prefix`).
        let v2 = engine_with_standard(
            &tmp.0.join("v2"),
            "# The `standard` agent preset\n    prefix: new\n",
        );
        let available2 = available_presets(&v2);
        let mut lines2 = Vec::new();
        let refreshed = ensure_legacy_code_shim(&home.0, &v2, &available2, &mut |l| {
            lines2.push(l.to_string())
        })
        .unwrap();
        assert!(refreshed);
        let (target, _, backup) = shim_files(&home.0);
        assert!(
            std::fs::read_to_string(&target)
                .unwrap()
                .contains("prefix: new")
        );
        assert!(
            std::fs::read_to_string(&backup)
                .unwrap()
                .contains("text: old")
        );
        assert!(lines2.iter().any(|l| l.contains("refreshed")));
        // A third run is a no-op once in sync.
        let again = ensure_legacy_code_shim(&home.0, &v2, &available2, &mut |_| {}).unwrap();
        assert!(!again);
    }

    #[test]
    fn shim_preserves_user_edits_after_install() {
        let tmp = TempDir::new("shim-edited");
        let home = TempDir::new("shim-edited-home");
        let v1 = engine_with_standard(
            &tmp.0.join("v1"),
            "# The `standard` agent preset\n    prefix: v1\n",
        );
        let available = available_presets(&v1);
        assert!(ensure_legacy_code_shim(&home.0, &v1, &available, &mut |_| {}).unwrap());
        let (target, _, backup) = shim_files(&home.0);
        std::fs::write(&target, "# my custom code preset\n    prefix: mine\n").unwrap();
        let v2 = engine_with_standard(
            &tmp.0.join("v2"),
            "# The `standard` agent preset\n    prefix: v2\n",
        );
        let available2 = available_presets(&v2);
        let mut lines = Vec::new();
        let changed = ensure_legacy_code_shim(&home.0, &v2, &available2, &mut |l| {
            lines.push(l.to_string())
        })
        .unwrap();
        assert!(!changed);
        assert!(
            std::fs::read_to_string(&target)
                .unwrap()
                .contains("prefix: mine")
        );
        assert!(!backup.is_file());
        assert!(lines.iter().any(|l| l.contains("leaving it alone")));
    }

    #[test]
    fn shim_adopts_in_sync_legacy_copy_without_marker() {
        let tmp = TempDir::new("shim-adopt");
        let home = TempDir::new("shim-adopt-home");
        let version = engine_with_standard(
            &tmp.0.join("v"),
            "# The `standard` agent preset\n    prefix: same\n",
        );
        let available = available_presets(&version);
        // Simulate a pre-marker install: bytes already match, no marker yet.
        let shim = home.0.join(USER_PRESET_DIR).join(LEGACY_CODE_PRESET);
        std::fs::create_dir_all(&shim).unwrap();
        std::fs::write(
            shim.join(COMPOSITION_FILE),
            "# The `standard` agent preset\n    prefix: same\n",
        )
        .unwrap();
        let mut lines = Vec::new();
        let changed = ensure_legacy_code_shim(&home.0, &version, &available, &mut |l| {
            lines.push(l.to_string())
        })
        .unwrap();
        assert!(!changed);
        let (_, marker, _) = shim_files(&home.0);
        assert!(marker.is_file());
        assert!(lines.is_empty());
    }

    #[test]
    fn shim_migrates_stale_premarker_copy() {
        let tmp = TempDir::new("shim-migrate");
        let home = TempDir::new("shim-migrate-home");
        // Pre-marker shim: stale bytes with the copied header, no marker.
        let shim = home.0.join(USER_PRESET_DIR).join(LEGACY_CODE_PRESET);
        std::fs::create_dir_all(&shim).unwrap();
        std::fs::write(
            shim.join(COMPOSITION_FILE),
            "# The `standard` agent preset\n    text: old\n",
        )
        .unwrap();
        let version = engine_with_standard(
            &tmp.0.join("v"),
            "# The `standard` agent preset\n    prefix: new\n",
        );
        let available = available_presets(&version);
        let mut lines = Vec::new();
        let changed = ensure_legacy_code_shim(&home.0, &version, &available, &mut |l| {
            lines.push(l.to_string())
        })
        .unwrap();
        assert!(changed);
        let (target, marker, backup) = shim_files(&home.0);
        assert!(
            std::fs::read_to_string(&target)
                .unwrap()
                .contains("prefix: new")
        );
        assert!(
            std::fs::read_to_string(&backup)
                .unwrap()
                .contains("text: old")
        );
        assert!(marker.is_file());
        assert!(lines.iter().any(|l| l.contains("refreshed")));
    }

    #[test]
    fn shim_preserves_user_authored_code_preset() {
        let tmp = TempDir::new("shim-authored");
        let home = TempDir::new("shim-authored-home");
        // No marker, and the bytes do not look like a launcher copy.
        let shim = home.0.join(USER_PRESET_DIR).join(LEGACY_CODE_PRESET);
        std::fs::create_dir_all(&shim).unwrap();
        std::fs::write(
            shim.join(COMPOSITION_FILE),
            "# my own code preset\n- id: persona\n",
        )
        .unwrap();
        let version = engine_with_standard(
            &tmp.0.join("v"),
            "# The `standard` agent preset\n    prefix: new\n",
        );
        let available = available_presets(&version);
        let changed = ensure_legacy_code_shim(&home.0, &version, &available, &mut |_| {}).unwrap();
        assert!(!changed);
        let (target, _, backup) = shim_files(&home.0);
        assert!(
            std::fs::read_to_string(&target)
                .unwrap()
                .contains("my own code preset")
        );
        assert!(!backup.is_file());
    }

    #[test]
    fn shim_skipped_when_engine_ships_code() {
        let tmp = TempDir::new("shim-rc");
        let home = TempDir::new("shim-rc-home");
        let version = rc_style_version(&tmp.0);
        let available = available_presets(&version);
        let made = ensure_legacy_code_shim(&home.0, &version, &available, &mut |_| {}).unwrap();
        assert!(!made);
        assert!(
            !home
                .0
                .join(USER_PRESET_DIR)
                .join(LEGACY_CODE_PRESET)
                .exists()
        );
    }
}
