//! Profile-local links for user patch-layer plugins.
//!
//! The harness home (`~/.dsh`) is shared across engine versions, and each
//! profile (e.g. `web`) carries its own `cordis.patch.yml` user patch layer
//! whose `insert` entries name plugins by bare package name
//! (`dsh-opencode-patch`, …). Older engines resolved those names through
//! Node's ordinary parent-walk, which reaches the shared
//! `~/.dsh/profiles/node_modules` fallback (where users typically keep
//! symlinks to local plugin checkouts). Newer engines resolve patch entry
//! names against the profile-local `node_modules` (plus the installation)
//! instead, so after an engine update the tree fails to boot with
//! `ERR_MODULE_NOT_FOUND ... imported from .../profiles/web/` even though
//! the packages are still present in the shared fallback. The failure
//! recurs on every update because nothing ever re-materialises the
//! profile-local links.
//!
//! Before every engine spawn this module mirrors each patch-referenced bare
//! name found in the shared fallback into the profile-local `node_modules`
//! as a symlink to the same target. Additive only: existing entries are
//! never overwritten, user patch files are never edited, and names with no
//! known source are left alone for the engine's own fail-loud diagnostic.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// User patch layer inside a profile directory.
const PATCH_FILE: &str = "cordis.patch.yml";
/// Profile-local module dir the engine resolves patch entry names against.
const PROFILE_MODULES: &str = "node_modules";
/// Shared fallback that still carries user-maintained plugin links.
const SHARED_MODULES: &str = "node_modules";
/// Profiles root under the harness home.
const PROFILES_DIR: &str = "profiles";

/// Extract bare package names from `name:` fields of a patch document.
///
/// The patch files are simple YAML (`insert` lists of `{id, name, ...}` maps
/// with `!!js` expressions elsewhere), so a line scanner is enough — no YAML
/// dependency needed. Returns sorted, de-duplicated names; relative paths,
/// URLs, `cordis:` builtins and empty values are dropped.
pub fn patch_bare_names(text: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for raw in text.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix("name:") else {
            continue;
        };
        let mut value = rest.trim();
        // Drop trailing YAML comments first (a `#` inside a bare package
        // name never occurs), then one layer of matching quotes.
        if let Some(hash) = value.find(" #") {
            value = value[..hash].trim();
        }
        if value.len() >= 2 {
            let bytes = value.as_bytes();
            let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
            if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
                value = value[1..value.len() - 1].trim();
            }
        }
        if value.is_empty() {
            continue;
        }
        // Not a bare package name: relative/absolute paths, URLs, builtins.
        if value.starts_with('.') || value.starts_with('/') || value.contains("://") {
            continue;
        }
        if value.starts_with("cordis:") {
            continue;
        }
        if value.chars().any(|c| c.is_whitespace()) {
            continue;
        }
        // At most one slash (a scope); deeper paths are subpath imports.
        if value.matches('/').count() > 1 {
            continue;
        }
        if value == "node_modules" || value == "." || value == ".." {
            continue;
        }
        out.insert(value.to_string());
    }
    out.into_iter().collect()
}

fn has_manifest(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

/// Ultimate target of a shared-fallback entry: one `read_link` hop when it
/// is a symlink (the usual user-maintained link to a local checkout),
/// otherwise the entry itself. Returns `None` when the target carries no
/// package manifest.
fn shared_target(shared_modules: &Path, name: &str) -> Option<PathBuf> {
    let entry = shared_modules.join(name);
    let meta = std::fs::symlink_metadata(&entry).ok()?;
    let target = if meta.file_type().is_symlink() {
        let raw = std::fs::read_link(&entry).ok()?;
        if raw.is_absolute() {
            raw
        } else {
            shared_modules.join(raw)
        }
    } else {
        if !meta.is_dir() {
            return None;
        }
        entry
    };
    has_manifest(&target).then_some(target)
}

/// Mirror one missing profile-local package link. Never overwrites: any
/// existing file, directory or symlink at the link path is left alone.
fn ensure_profile_link(profile_modules: &Path, name: &str, target: &Path) -> Result<bool, String> {
    let link = profile_modules.join(name);
    if std::fs::symlink_metadata(&link).is_ok() {
        return Ok(false);
    }
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, &link)
            .map_err(|e| format!("symlink {}: {e}", link.display()))?;
        Ok(true)
    }
    #[cfg(not(unix))]
    {
        let _ = (target, link);
        Err("profile-local plugin links are macOS-only".to_string())
    }
}

/// Repair one profile directory. Returns the number of links created.
fn repair_profile(
    profile_dir: &Path,
    shared_modules: &Path,
    install_modules: &Path,
    log: &mut dyn FnMut(&str),
) -> usize {
    let patch_path = profile_dir.join(PATCH_FILE);
    let Ok(text) = std::fs::read_to_string(&patch_path) else {
        return 0;
    };
    let profile_name = profile_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?");
    let mut made = 0;
    for name in patch_bare_names(&text) {
        let local = profile_dir.join(PROFILE_MODULES).join(&name);
        if has_manifest(&local) {
            continue;
        }
        // The installation itself provides it (bundled plugin): nothing to do.
        if has_manifest(&install_modules.join(&name)) {
            continue;
        }
        match shared_target(shared_modules, &name) {
            Some(target) => {
                match ensure_profile_link(&profile_dir.join(PROFILE_MODULES), &name, &target) {
                    Ok(true) => {
                        made += 1;
                        log(&format!(
                            "linked profile \"{profile_name}\" plugin {name} into its node_modules so engine updates keep resolving it (target kept as-is)"
                        ));
                    }
                    Ok(false) => {}
                    Err(e) => log(&format!("profile \"{profile_name}\" plugin link failed: {e}")),
                }
            }
            None => log(&format!(
                "profile \"{profile_name}\" patch references {name}, which is installed nowhere (profile, shared fallback or engine) — leaving it for the engine to report"
            )),
        }
    }
    made
}

/// Mirror shared-fallback plugin links into every profile's local
/// `node_modules` before an engine spawn. Failures are logged but never
/// block the start.
pub fn repair_profile_patch_links(
    home: &Path,
    version_dir: &Path,
    log: &mut dyn FnMut(&str),
) {
    let profiles = home.join(PROFILES_DIR);
    let Ok(entries) = std::fs::read_dir(&profiles) else {
        return;
    };
    let shared_modules = profiles.join(SHARED_MODULES);
    let install_modules = version_dir.join(PROFILE_MODULES);
    for entry in entries.filter_map(|e| e.ok()) {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        // Skip the shared fallback dir itself if it ever appears as an entry
        // shape differs (it holds packages, not profiles).
        if dir.join(PATCH_FILE).is_file() || dir.join("package.json").is_file() {
            repair_profile(&dir, &shared_modules, &install_modules, log);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "dsh-launcher-links-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn make_package(dir: &Path, name: &str) -> PathBuf {
        let pkg = dir.join(name);
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            format!("{{\"name\":\"{name}\",\"version\":\"0.0.1\"}}"),
        )
        .unwrap();
        pkg
    }

    fn layout(tag: &str) -> (TempDir, PathBuf, PathBuf, PathBuf) {
        let tmp = TempDir::new(tag);
        let home = tmp.path().join("home");
        let shared = home.join("profiles").join("node_modules");
        let profile = home.join("profiles").join("web");
        std::fs::create_dir_all(&shared).unwrap();
        std::fs::create_dir_all(profile.join("node_modules")).unwrap();
        let version = tmp.path().join("versions").join("v");
        std::fs::create_dir_all(version.join("node_modules")).unwrap();
        (tmp, home, profile, version)
    }

    #[test]
    fn extracts_bare_names_and_ignores_non_packages() {
        let names = patch_bare_names(
            "- insert:\n    - id: a\n      name: 'dsh-opencode-patch'\n    - id: b\n      name: \"@scope/pkg\" # comment\n    - id: c\n      name: ./relative\n    - id: d\n      name: cordis:include\n    - id: e\n      name: https://x/y\n    - id: f\n      name: a/b/c\n",
        );
        assert_eq!(names, vec!["@scope/pkg", "dsh-opencode-patch"]);
    }

    #[test]
    fn mirrors_shared_link_into_profile_local_modules() {
        let (_tmp, home, profile, version) = layout("mirror");
        let real = make_package(&home.join("spoon"), "dsh-opencode-patch");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, home.join("profiles").join("node_modules").join("dsh-opencode-patch")).unwrap();
        std::fs::write(
            profile.join(PATCH_FILE),
            "- insert:\n    - id: x\n      name: 'dsh-opencode-patch'\n",
        )
        .unwrap();
        let mut lines: Vec<String> = Vec::new();
        repair_profile_patch_links(&home, &version, &mut |l| lines.push(l.to_string()));
        let link = profile.join("node_modules").join("dsh-opencode-patch");
        assert!(has_manifest(&link), "profile-local link missing");
        assert!(lines.iter().any(|l| l.contains("dsh-opencode-patch")));
    }

    #[test]
    fn never_overwrites_existing_entries() {
        let (_tmp, home, profile, version) = layout("nooverwrite");
        let real = make_package(&home.join("spoon"), "dsh-opencode-patch");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, home.join("profiles").join("node_modules").join("dsh-opencode-patch")).unwrap();
        // A stale-but-present directory must survive untouched.
        let existing = profile.join("node_modules").join("dsh-opencode-patch");
        std::fs::create_dir_all(&existing).unwrap();
        std::fs::write(
            profile.join(PATCH_FILE),
            "- insert:\n    - id: x\n      name: 'dsh-opencode-patch'\n",
        )
        .unwrap();
        let mut lines: Vec<String> = Vec::new();
        repair_profile_patch_links(&home, &version, &mut |l| lines.push(l.to_string()));
        assert!(existing.is_dir() && !has_manifest(&existing));
        assert!(!lines.iter().any(|l| l.contains("linked profile")));
    }

    #[test]
    fn installation_provided_names_need_no_mirror() {
        let (_tmp, home, profile, version) = layout("install");
        make_package(&version.join("node_modules"), "@deepseek-ai/dsh-mcp-client");
        std::fs::write(
            profile.join(PATCH_FILE),
            "- insert:\n    - id: mcp\n      name: '@deepseek-ai/dsh-mcp-client'\n",
        )
        .unwrap();
        let mut lines: Vec<String> = Vec::new();
        repair_profile_patch_links(&home, &version, &mut |l| lines.push(l.to_string()));
        assert!(!profile.join("node_modules/@deepseek-ai/dsh-mcp-client").exists());
        assert!(lines.is_empty());
    }

    #[test]
    fn unknown_names_only_warn_and_create_nothing() {
        let (_tmp, home, profile, version) = layout("unknown");
        std::fs::write(
            profile.join(PATCH_FILE),
            "- insert:\n    - id: ghost\n      name: 'dsh-no-such-plugin'\n",
        )
        .unwrap();
        let mut lines: Vec<String> = Vec::new();
        repair_profile_patch_links(&home, &version, &mut |l| lines.push(l.to_string()));
        assert!(!profile.join("node_modules/dsh-no-such-plugin").exists());
        assert!(lines.iter().any(|l| l.contains("dsh-no-such-plugin")));
    }

    #[test]
    fn missing_patch_or_profiles_is_a_noop() {
        let tmp = TempDir::new("noop");
        let mut lines: Vec<String> = Vec::new();
        repair_profile_patch_links(tmp.path(), tmp.path(), &mut |l| lines.push(l.to_string()));
        assert!(lines.is_empty());
    }
}
