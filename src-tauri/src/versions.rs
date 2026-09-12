//! npm-driven harness version operations. Everything runs through the bundled
//! Node sidecar + vendored npm CLI - no system node/npm/npx needed.
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::mcp_env;

pub const PACKAGE: &str = "@deepseek-ai/dsh";

/// npm 12 blocks install-time lifecycle scripts by default (the `allowScripts`
/// policy): a package whose install script compiles a native binding (e.g.
/// `fs-ext@2.1.1`'s `install: node-gyp configure build`, pulled in by
/// `@deepseek-ai/dsh-session-persistence-jsonl` since harness 0.1.3-alpha)
/// would otherwise install WITHOUT its `.node` binary and the engine would
/// crash at boot with `Cannot find module './build/Release/fs_ext.node'`.
/// The harness tree is a pinned, first-party, isolated prefix, so every
/// harness install/rebuild explicitly opts back into scripts. The flag is
/// logged to the operation console at each use so the allowance is visible.
/// Streaming console sink for npm operations: (stream, line), where stream is
/// either stdout or stderr text.
pub type ProgressSink<'a> = Option<&'a mut dyn FnMut(&str, &str)>;

pub const ALLOW_SCRIPTS_FLAG: &str = "--dangerously-allow-all-scripts";

pub fn npm_cli_path(app: &AppHandle) -> Result<PathBuf, String> {
    let res = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource dir unavailable: {e}"))?;
    let cli = res
        .join("resources")
        .join("npm")
        .join("bin")
        .join("npm-cli.js");
    if !cli.exists() {
        return Err(format!(
            "vendored npm not found at {} (run the prepare step / rebuild the app)",
            cli.display()
        ));
    }
    Ok(cli)
}

pub fn run_npm(
    app: &AppHandle,
    runtime_dir: &Path,
    npm_args: &[&str],
    timeout: Duration,
    mut sink: ProgressSink<'_>,
) -> Result<(String, String), String> {
    let cli = npm_cli_path(app)?;
    let cache = runtime_dir.join("npm-cache");
    let _ = std::fs::create_dir_all(&cache);

    // Native bindings (node-gyp) spawned as lifecycle scripts resolve `node`
    // from PATH. Point PATH at a shim for the BUNDLED node first so builds
    // target the engine's runtime ABI: a system node on PATH (or no node at
    // all under a GUI launch) would otherwise compile for the wrong
    // NODE_MODULE_VERSION or fail outright. Also best-effort restore the
    // vendored node-gyp helper's executable bit (early vendored trees shipped
    // it as 0644, which fails install scripts with `Permission denied` 126).
    // Only needed when scripts actually run (install/rebuild); `view` calls
    // stay on the plain environment.
    let needs_scripts = matches!(npm_args.first(), Some(a) if *a == "install" || *a == "rebuild");
    let mut extra_path: Option<String> = None;
    if needs_scripts {
        ensure_node_gyp_exec(&cli);
        match node_shim_dir(app, runtime_dir) {
            Ok(dir) => {
                let prior = std::env::var("PATH").unwrap_or_default();
                extra_path = Some(format!("{}:{prior}", dir.display()));
            }
            Err(e) => {
                if let Some(s) = sink.as_mut() {
                    s(
                        "err",
                        &format!(
                            "node shim unavailable ({e}); lifecycle scripts may build for the wrong node"
                        ),
                    );
                }
            }
        }
    }

    let mut args: Vec<String> = vec![cli.to_string_lossy().into_owned()];
    args.extend(npm_args.iter().map(|s| s.to_string()));
    let sidecar = app
        .shell()
        .sidecar("node")
        .map_err(|e| format!("node sidecar unavailable: {e}"))?;
    let mut cmd = sidecar.args(args);
    cmd = cmd
        .current_dir(runtime_dir)
        .env("npm_config_cache", cache.to_string_lossy().into_owned())
        .env("npm_config_update_notifier", "false")
        .env("npm_config_fund", "false")
        .env("npm_config_audit", "false");
    if let Some(path) = extra_path {
        cmd = cmd.env("PATH", path);
    }
    let (mut rx_async, child) = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn node: {e}"))?;
    let (tx, rx) = std::sync::mpsc::channel::<CommandEvent>();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_async.recv().await {
            let term = matches!(&event, CommandEvent::Terminated(_));
            if tx.send(event).is_err() {
                break;
            }
            if term {
                break;
            }
        }
    });

    let poll_interval = Duration::from_millis(250);
    let deadline = std::time::Instant::now() + timeout;
    let mut stdout = String::new();
    let mut stderr = String::new();
    loop {
        match rx.recv_timeout(poll_interval) {
            Ok(CommandEvent::Stdout(line)) => {
                let text = String::from_utf8_lossy(&line);
                if let Some(s) = sink.as_mut() {
                    s("out", text.trim_end());
                }
                stdout.push_str(&text);
            }
            Ok(CommandEvent::Stderr(line)) => {
                let text = String::from_utf8_lossy(&line);
                if let Some(s) = sink.as_mut() {
                    s("err", text.trim_end());
                }
                stderr.push_str(&text);
            }
            Ok(CommandEvent::Error(e)) => stderr.push_str(&format!("[node error] {e}\n")),
            Ok(CommandEvent::Terminated(p)) => {
                if let Some(code) = p.code {
                    if code != 0 && stderr.trim().is_empty() && stdout.trim().is_empty() {
                        stderr.push_str(&format!("process exited with code {code}"));
                    }
                }
                break;
            }
            Ok(_) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Only treat as a hard timeout once the cumulative deadline is
                // exceeded. The short poll_interval keeps the deadline check
                // responsive even during silent npm phases.
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    return Err(format!(
                        "npm operation timed out after {}s: {}",
                        timeout.as_secs(),
                        npm_args.join(" ")
                    ));
                }
            }
            Err(_) => break,
        }
    }
    Ok((stdout, stderr))
}
pub fn list_versions(
    app: &AppHandle,
    runtime_dir: &Path,
    include_prerelease: bool,
) -> Result<Vec<String>, String> {
    let (out, err) = run_npm(
        app,
        runtime_dir,
        &["view", PACKAGE, "versions", "--json"],
        Duration::from_secs(30),
        None,
    )?;
    let trimmed = out.trim();
    if trimmed.is_empty() {
        return Err(format!("npm view returned nothing: {err}"));
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("unexpected npm versions output: {e}"))?;
    let mut raw: Vec<String> = match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        other => return Err(format!("unexpected versions value: {other}")),
    };

    let mut parsed: Vec<(Version, String)> = Vec::new();
    for v in raw.drain(..) {
        if let Ok(vv) = Version::parse(v.trim_start_matches('v')) {
            parsed.push((vv, v.clone()));
        }
    }
    parsed.sort_by(|a, b| a.0.cmp(&b.0));
    parsed.dedup_by(|a, b| a.0 == b.0);

    let stable: Vec<String> = parsed
        .iter()
        .filter(|(v, _)| v.pre.is_empty())
        .map(|(_, s)| s.clone())
        .collect();
    if include_prerelease || stable.is_empty() {
        Ok(parsed.into_iter().map(|(_, s)| s).collect())
    } else {
        Ok(stable)
    }
}

/// Newest version of a published-version list: the maximum by semver order.
/// Non-semver entries are ignored; empty (or unparseable) input yields None.
/// The input is expected ascending (as `list_versions` returns), but the max
/// is computed explicitly so unsorted callers stay correct.
pub fn newest_published(versions: &[String]) -> Option<String> {
    versions
        .iter()
        .filter_map(|v| {
            Version::parse(v.trim_start_matches('v'))
                .ok()
                .map(|parsed| (parsed, v.clone()))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, raw)| raw)
}

/// Newest PUBLISHED harness version across every release (pre-releases
/// included). Manual checks, update-to-latest, boot default and the status
/// banner all resolve "latest" through here — never through the npm `latest`
/// dist-tag, which can lag the actual newest release (e.g. while newer
/// pre-releases exist).
pub fn fetch_newest(app: &AppHandle, runtime_dir: &Path) -> Result<String, String> {
    let all = list_versions(app, runtime_dir, true)?;
    newest_published(&all).ok_or_else(|| "npm registry returned no usable versions".to_string())
}

pub fn is_installed(runtime_dir: &Path, version: &str) -> bool {
    let marker = version_dir(runtime_dir, version)
        .join("node_modules")
        .join(PACKAGE)
        .join("package.json");
    marker.exists()
}

pub fn version_dir(runtime_dir: &Path, version: &str) -> PathBuf {
    runtime_dir.join("versions").join(version)
}

/// Guard for anything that turns a user-supplied version string into a path
/// (directory open, delete, …): a version must be a plain npm dist/semver
/// name, never a path segment, so traversal and absolute paths are rejected.
/// Typed gate for anything that turns a user-supplied version string into a
/// path or npm spec: returns the borrowed name on success so call sites keep
/// their types, and propagates [`crate::errors::AppError::InvalidVersion`]
/// (converted to the legacy message at the IPC boundary) on failure.
pub fn checked_version_name(version: &str) -> Result<&str, crate::errors::AppError> {
    if is_valid_version_name(version) {
        Ok(version)
    } else {
        Err(crate::errors::AppError::InvalidVersion {
            name: version.to_string(),
        })
    }
}

pub fn is_valid_version_name(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 64
        && version != "."
        && version != ".."
        && !version.contains(['/', '\\'])
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+' | '~'))
}

#[cfg(test)]
mod name_tests {
    use super::is_valid_version_name;

    #[test]
    fn accepts_npm_version_names() {
        assert!(is_valid_version_name("0.1.29"));
        assert!(is_valid_version_name("0.1.2-alpha.3"));
        assert!(is_valid_version_name("0.1.0-rc.7"));
        assert!(is_valid_version_name("v1.2.3+build.4_meta~x"));
    }

    #[test]
    fn rejects_path_traversal_and_invalid_names() {
        assert!(!is_valid_version_name(""));
        assert!(!is_valid_version_name("."));
        assert!(!is_valid_version_name(".."));
        assert!(!is_valid_version_name("../evil"));
        assert!(!is_valid_version_name("a/b"));
        assert!(!is_valid_version_name("a\\b"));
        assert!(!is_valid_version_name("/absolute/path"));
        assert!(!is_valid_version_name("has space"));
        assert!(!is_valid_version_name(&"x".repeat(65)));
    }

    #[test]
    fn rejects_registry_shaped_attack_strings() {
        // Values a compromised registry (or a crafted settings.json) could
        // feed into install/switch/rollback paths.
        for evil in [
            "@deepseek-ai/dsh@9.9.9",
            "1.2.3 --ignore-scripts=false",
            "1.2.3; rm -rf ~",
            "1.2.3$(id)",
            "1.2.3`id`",
            "..\\..\\evil",
            "v1.2.3/../../evil",
        ] {
            assert!(!is_valid_version_name(evil), "{evil}");
        }
    }

    #[test]
    fn validated_names_stay_inside_versions_dir() {
        use super::version_dir;
        let base = std::path::Path::new("/tmp/rd");
        let parent = base.join("versions");
        for good in [
            "0.1.29",
            "0.1.2-alpha.3",
            "0.1.0-rc.7",
            "v1.2.3+build.4_meta~x",
        ] {
            assert!(is_valid_version_name(good));
            assert!(version_dir(base, good).starts_with(&parent), "{good}");
        }
    }
}

pub fn harness_entry(runtime_dir: &Path, version: &str) -> Option<PathBuf> {
    let p = version_dir(runtime_dir, version)
        .join("node_modules")
        .join(PACKAGE)
        .join("lib")
        .join("bin.js");
    p.exists().then_some(p)
}

#[cfg(test)]
mod newest_tests {
    use super::newest_published;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn picks_max_semver_including_prereleases() {
        // A lagging stable must not shadow a newer pre-release: the dist-tag
        // shape that motivated newest-published resolution.
        assert_eq!(
            newest_published(&names(&["0.1.1-rc.2", "0.1.1", "0.1.2-alpha.4"])).as_deref(),
            Some("0.1.2-alpha.4")
        );
        assert_eq!(
            newest_published(&names(&["0.1.9", "0.1.7", "0.1.10"])).as_deref(),
            Some("0.1.10")
        );
    }

    #[test]
    fn ignores_unparseable_entries_and_empty_input() {
        assert_eq!(
            newest_published(&names(&["latest", "0.1.3", "next"])).as_deref(),
            Some("0.1.3")
        );
        assert_eq!(newest_published(&[]), None);
        assert_eq!(newest_published(&names(&["latest"])), None);
    }
}

pub fn install_version(
    app: &AppHandle,
    runtime_dir: &Path,
    version: &str,
    op: &str,
) -> Result<(), String> {
    // Choke point: every install/switch/update/boot path funnels through here,
    // so a single gate covers registry-supplied and caller-supplied names.
    // Without this, `runtime/versions/<version>` could escape its parent and
    // the npm spec `@deepseek-ai/dsh@<version>` could be attacker-shaped.
    checked_version_name(version)?;
    if is_installed(runtime_dir, version) {
        return Ok(());
    }
    let dir = version_dir(runtime_dir, version);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {dir:?}: {e}"))?;
    let spec = format!("{PACKAGE}@{version}");
    crate::progress::reset_console(app, op);
    crate::progress::push_console(app, op, "info", &format!("$ npm install {spec}"));
    crate::progress::emit(
        app,
        op,
        Some(version),
        "installing",
        &format!("Installing {spec}…"),
        None,
    );
    let app_sink = app.clone();
    let op_sink = op.to_string();
    let ver_sink = version.to_string();
    let mut last_console = std::time::Instant::now() - Duration::from_secs(1);
    let mut last_progress = std::time::Instant::now() - Duration::from_secs(1);
    let mut fetched: u32 = 0;
    let mut throttled_sink = move |stream: &str, line: &str| {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        let is_fetch = line.contains("npm http fetch");
        if is_fetch {
            fetched += 1;
        }
        let is_summary = line.starts_with("added ")
            || line.starts_with("changed ")
            || line.starts_with("removed ");
        let now = std::time::Instant::now();
        // Terminal: surface every line (light throttle) with its stream.
        if now.duration_since(last_console).as_millis() >= 60 {
            last_console = now;
            let text: String = line.chars().take(600).collect();
            crate::progress::push_console(&app_sink, &op_sink, stream, &text);
        }
        // Progress bar: throttled harder, truncated preview.
        if now.duration_since(last_progress).as_millis() >= 200 {
            last_progress = now;
            let preview: String = if is_fetch {
                format!("fetching packages… {fetched}")
            } else if is_summary {
                line.chars().take(160).collect()
            } else {
                String::new()
            };
            if !preview.is_empty() {
                crate::progress::emit(
                    &app_sink,
                    &op_sink,
                    Some(&ver_sink),
                    "installing",
                    &preview,
                    None,
                );
            }
        }
    };
    let (out, err) = run_npm(
        app,
        runtime_dir,
        &[
            "install",
            "--prefix",
            dir.to_string_lossy().as_ref(),
            "--no-save",
            "--no-audit",
            "--no-fund",
            "--no-color",
            "--legacy-peer-deps",
            // npm 12 blocks lifecycle scripts by default; without this the
            // native bindings a harness version needs (e.g. fs-ext since
            // 0.1.3-alpha) are silently skipped and the engine cannot boot.
            ALLOW_SCRIPTS_FLAG,
            "--fetch-retries=2",
            "--fetch-timeout=120000",
            "--loglevel=http",
            "--progress=false",
            spec.as_str(),
        ],
        Duration::from_secs(120),
        Some(&mut throttled_sink),
    )?;
    if !is_installed(runtime_dir, version) {
        // npm may still be flushing files to disk; give it a brief grace period
        // before concluding the harness is missing (avoids a false failure on
        // slow filesystem writes).
        std::thread::sleep(Duration::from_millis(800));
    }
    if is_installed(runtime_dir, version) {
        ensure_peer_completion(app, runtime_dir, version, op)?;
    }
    crate::progress::push_console(app, op, "info", "Install finished — verifying…");
    crate::progress::emit(
        app,
        op,
        Some(version),
        "verifying",
        "Verifying installation…",
        Some(85),
    );
    if !is_installed(runtime_dir, version) {
        std::thread::sleep(Duration::from_millis(800));
    }
    if !is_installed(runtime_dir, version) {
        // Surface a clear, actionable error instead of leaving the UI in a
        // perpetual stuck state. A peer-dependency conflict can make arborist
        // silently skip reify (now mitigated by the --legacy-peer-deps flag on
        // the install command), so call it out.
        let marker = version_dir(runtime_dir, version)
            .join("node_modules")
            .join(PACKAGE);
        let hint = if !marker.exists() {
            format!(
                "the harness package directory is missing at {} - npm likely skipped reify (peer-dependency conflict?). The launcher installs with --legacy-peer-deps to avoid this.",
                marker.display()
            )
        } else {
            format!(
                "the harness package at {} is present but incomplete",
                marker.display()
            )
        };
        return Err(format!(
            "install of {spec} did not produce a usable harness\n{hint}\nstdout: {out}\nstderr: {err}"
        ));
    }
    // Bootability gate: a present package.json is not enough — a tree whose
    // native bindings were skipped (blocked install scripts) or miscompiled
    // must fail HERE, before settings point at it and before the engine
    // spends 30s not serving.
    verify_harness_tree(app, runtime_dir, version, op)?;
    Ok(())
}
/// Find @deepseek-ai/* packages referenced as peerDependencies somewhere in
/// the installed tree but not actually present in node_modules/@deepseek-ai.
///
/// Published alpha/rc builds occasionally move runtime-required modules to
/// peerDependencies of scoped packages. With --legacy-peer-deps npm skips
/// peer auto-install, leaving the harness unable to boot
/// (ERR_MODULE_NOT_FOUND). The launcher compensates by installing the
/// missing peers explicitly alongside the harness package.
fn missing_peer_specs(runtime_dir: &Path, version: &str) -> Result<Vec<String>, String> {
    let scope = version_dir(runtime_dir, version)
        .join("node_modules")
        .join("@deepseek-ai");
    let mut peers: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let entries = std::fs::read_dir(&scope)
        .map_err(|e| format!("read installed scope {}: {e}", scope.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
        let pkg_bin = entry.path();
        if !pkg_bin.is_dir() {
            continue;
        }
        let manifest = pkg_bin.join("package.json");
        if !manifest.exists() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(peers_map) = json.get("peerDependencies").and_then(|v| v.as_object()) {
            for (name, range) in peers_map {
                let name = name.to_string();
                let prefix = "@deepseek-ai/";
                if name.starts_with(prefix) {
                    let short = name.trim_start_matches(prefix);
                    let present = scope.join(short).is_dir();
                    if !present {
                        let range_str = range
                            .as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| range.to_string());
                        peers
                            .entry(name.clone())
                            .or_insert_with(|| format!("{}@{}", name, range_str));
                    }
                }
            }
        }
    }
    Ok(peers.into_values().collect())
}
/// Ensure an installed harness tree has all of its @deepseek-ai/* peer
/// packages materialised. Published alpha/rc builds move runtime-required
/// modules to peerDependencies; with --legacy-peer-deps npm skips peer
/// auto-install, leaving the updated tree unbootable (ERR_MODULE_NOT_FOUND).
/// Runs an idempotent npm install of dsh + the missing peers when needed.
pub fn ensure_peer_completion(
    app: &AppHandle,
    runtime_dir: &Path,
    version: &str,
    op: &str,
) -> Result<(), String> {
    if !is_installed(runtime_dir, version) {
        return Ok(());
    }
    let extra = missing_peer_specs(runtime_dir, version)?;
    if extra.is_empty() {
        return Ok(());
    }
    let dir = version_dir(runtime_dir, version);
    let spec = format!("{}@{}", PACKAGE, version);
    crate::progress::push_console(
        app,
        op,
        "info",
        &format!("Installing {} missing peer packages...", extra.len()),
    );
    crate::progress::emit(
        app,
        op,
        Some(version),
        "installing",
        "Completing missing peer packages...",
        Some(88),
    );
    let mut args: Vec<String> = vec![
        "install".into(),
        "--prefix".into(),
        dir.to_string_lossy().into_owned(),
        "--no-save".into(),
        "--no-audit".into(),
        "--no-fund".into(),
        "--no-color".into(),
        "--legacy-peer-deps".into(),
        // Same script policy as the main install: peer completion must also
        // compile native bindings, not just unpack them.
        ALLOW_SCRIPTS_FLAG.into(),
        "--fetch-retries=2".into(),
        "--fetch-timeout=120000".into(),
        "--loglevel=http".into(),
        "--progress=false".into(),
        spec,
    ];
    args.extend(extra);
    let npm_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut sink = |stream: &str, line: &str| {
        let line = line.trim();
        if !line.is_empty() {
            crate::progress::push_console(
                app,
                op,
                stream,
                &line.chars().take(600).collect::<String>(),
            );
        }
    };
    run_npm(
        app,
        runtime_dir,
        &npm_args,
        Duration::from_secs(120),
        Some(&mut sink),
    )?;
    crate::progress::push_console(app, op, "info", "Peer packages complete - verifying...");
    crate::progress::emit(
        app,
        op,
        Some(version),
        "verifying",
        "Verifying installation...",
        Some(95),
    );
    if !is_installed(runtime_dir, version) {
        std::thread::sleep(Duration::from_millis(800));
    }
    if !is_installed(runtime_dir, version) {
        return Err(format!(
            "peer completion did not leave a usable harness for {}",
            version
        ));
    }
    Ok(())
}

/// Best-effort restore of the vendored node-gyp helper's executable bit.
/// Early vendored npm trees shipped
/// `node_modules/@npmcli/run-script/lib/node-gyp-bin/node-gyp` as 0644, so
/// any install with scripts allowed failed with `Permission denied` (exit
/// 126) instead of compiling. Fixed at vendor time by bundle-npm.mjs; this
/// covers app bundles already deployed. Failures are silent: the npm run
/// will surface them if the bit is still missing.
fn ensure_node_gyp_exec(npm_cli: &Path) {
    #[cfg(unix)]
    {
        let helper = npm_cli
            .parent()
            .map(|bin| bin.join("../node_modules/@npmcli/run-script/lib/node-gyp-bin/node-gyp"));
        if let Some(path) = helper {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let mode = meta.permissions().mode();
                if mode & 0o111 == 0 {
                    let mut perms = meta.permissions();
                    perms.set_mode(mode | 0o755);
                    let _ = std::fs::set_permissions(&path, perms);
                }
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = npm_cli;
    }
}

/// Run the bundled node sidecar with `args` in `cwd` and capture
/// (exit code, stdout, stderr). Used for install verification and shim
/// resolution — never for the engine itself (see runtime::start).
fn run_node_capture(
    app: &AppHandle,
    args: Vec<String>,
    cwd: &Path,
    timeout: Duration,
) -> Result<(Option<i32>, String, String), String> {
    let (mut rx_async, child) = app
        .shell()
        .sidecar("node")
        .map_err(|e| format!("node sidecar unavailable: {e}"))?
        .args(args)
        .current_dir(cwd)
        .env("npm_config_update_notifier", "false")
        .spawn()
        .map_err(|e| format!("failed to spawn node: {e}"))?;
    let (tx, rx) = std::sync::mpsc::channel::<CommandEvent>();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx_async.recv().await {
            let term = matches!(&event, CommandEvent::Terminated(_));
            if tx.send(event).is_err() {
                break;
            }
            if term {
                break;
            }
        }
    });
    let deadline = std::time::Instant::now() + timeout;
    let mut stdout = String::new();
    let mut stderr = String::new();
    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(CommandEvent::Stdout(line)) => {
                stdout.push_str(&String::from_utf8_lossy(&line));
            }
            Ok(CommandEvent::Stderr(line)) => {
                stderr.push_str(&String::from_utf8_lossy(&line));
            }
            Ok(CommandEvent::Error(e)) => {
                stderr.push_str(&format!("[node error] {e}\n"));
            }
            Ok(CommandEvent::Terminated(p)) => {
                return Ok((p.code, stdout, stderr));
            }
            Ok(_) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    return Err("node verification timed out".to_string());
                }
            }
            Err(_) => break,
        }
    }
    Err("node verification ended without an exit code".to_string())
}

/// Absolute path of the bundled node binary, resolved by asking the sidecar
/// itself (`process.execPath`). Needed for the PATH shim below; resolving
/// dynamically keeps dev (`src-tauri/binaries/…`) and prod
/// (`…/Contents/MacOS/node`) layouts working without hardcoding either.
fn bundled_node_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir();
    let (code, out, err) = run_node_capture(
        app,
        vec!["-p".to_string(), "process.execPath".to_string()],
        &dir,
        Duration::from_secs(15),
    )?;
    if code != Some(0) {
        return Err(format!("node execPath check failed: {err}{out}"));
    }
    let path = PathBuf::from(out.trim());
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("node execPath is not a file: {}", path.display()))
    }
}

/// Directory holding a `node` symlink to the bundled sidecar. Prepending it
/// to PATH for npm install/rebuild makes lifecycle scripts (node-gyp)
/// compile against the engine runtime's NODE_MODULE_VERSION instead of
/// whatever system node happens to be on PATH (or none under GUI launch).
fn node_shim_dir(app: &AppHandle, runtime_dir: &Path) -> Result<PathBuf, String> {
    let target = bundled_node_path(app)?;
    let dir = runtime_dir.join("node-bin");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let link = dir.join("node");
    #[cfg(unix)]
    {
        let stale = std::fs::read_link(&link)
            .ok()
            .map(|p| p != target)
            .unwrap_or(true);
        // A regular file (not a symlink) at the link path is never touched.
        let is_link = std::fs::symlink_metadata(&link)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);
        if !is_link && link.exists() {
            return Err(format!(
                "{} exists and is not a symlink; refusing to replace it",
                link.display()
            ));
        }
        if stale {
            let _ = std::fs::remove_file(&link);
            std::os::unix::fs::symlink(&target, &link)
                .map_err(|e| format!("symlink {}: {e}", link.display()))?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (target, link);
        return Err("node shim is macOS-only".to_string());
    }
    Ok(dir)
}

/// True when `text` (engine output, smoke-test output) shows the harness
/// tree failed for a missing/unloadable native binding: the fs-ext shape
/// from 0.1.3-alpha (`Cannot find module './build/Release/fs_ext.node'`),
/// an ABI mismatch (`was compiled against a different Node.js version`),
/// or the resulting plugin-tree load failure.
pub fn is_native_binding_failure(text: &str) -> bool {
    // MCP invalid-config shares the plugin-tree prefix but needs env help,
    // not a native rebuild -- let the MCP classifier own it.
    if mcp_env::is_mcp_config_failure(text) {
        return false;
    }
    let lower = text.to_lowercase();
    lower.contains("fs_ext.node")
        || lower.contains("cannot find module")
        || lower.contains("plugin tree failed to load")
        || lower.contains("compiled against a different node")
        || lower.contains("node_module_version")
}

/// Actionable error for a harness version whose native bindings are missing
/// or unloadable. Points at repair (rebuild with scripts allowed) and at the
/// safe fallback (roll back), and states the data guarantee explicitly.
pub fn native_binding_error(version: &str, detail: &str) -> String {
    let short: String = detail.trim().chars().take(300).collect();
    format!(
        "harness {version} cannot boot: a native module failed to load ({short}). \
         This happens when npm skipped install scripts during download. \
         Reinstall or repair v{version} with install scripts allowed, or roll back to the previous working version. \
         Your sessions and settings in ~/.dsh are untouched."
    )
}

/// Post-install bootability gate. A mere `package.json` presence check let
/// the broken 0.1.3-alpha.2 tree report success; this instead:
///  1. runs `node bin.js --version` with the bundled node (catches
///     incomplete installs), and
///  2. when `fs-ext` is in the tree, requires its native binding to exist
///     AND load under the bundled node (catches blocked scripts as well as
///     ABI mismatches from building with a system node).
///
/// Writes only to the operation console; never touches `~/.dsh`.
pub fn verify_harness_tree(
    app: &AppHandle,
    runtime_dir: &Path,
    version: &str,
    op: &str,
) -> Result<(), String> {
    let entry = harness_entry(runtime_dir, version)
        .ok_or_else(|| format!("harness {version} is not installed (missing bin.js)"))?;
    let (code, out, err) = run_node_capture(
        app,
        vec![
            entry.to_string_lossy().into_owned(),
            "--version".to_string(),
        ],
        runtime_dir,
        Duration::from_secs(20),
    )?;
    if code != Some(0) {
        let detail = format!("{out}{err}");
        if is_native_binding_failure(&detail) {
            return Err(native_binding_error(version, &detail));
        }
        return Err(format!(
            "harness {version} failed its post-install check (bin.js --version exited {:?}): {}",
            code,
            detail.trim().chars().take(300).collect::<String>()
        ));
    }
    let fs_ext_dir = version_dir(runtime_dir, version)
        .join("node_modules")
        .join("fs-ext");
    if fs_ext_dir.is_dir() {
        let binding = fs_ext_dir.join("build").join("Release").join("fs_ext.node");
        if !binding.is_file() {
            return Err(native_binding_error(
                version,
                &format!(
                    "Cannot find module './build/Release/fs_ext.node' ({} is missing)",
                    binding.display()
                ),
            ));
        }
        let (bcode, bout, berr) = run_node_capture(
            app,
            vec![
                "-e".to_string(),
                "require(process.argv[1])".to_string(),
                binding.to_string_lossy().into_owned(),
            ],
            runtime_dir,
            Duration::from_secs(20),
        )?;
        if bcode != Some(0) {
            let detail = format!("{bout}{berr}");
            return Err(native_binding_error(version, &detail));
        }
    }
    crate::progress::push_console(
        app,
        op,
        "info",
        &format!(
            "Verified harness {version} boots (bin.js --version: {})",
            out.trim()
        ),
    );
    Ok(())
}

/// Rebuild the native bindings of an ALREADY-INSTALLED version in place
/// (`npm rebuild` with install scripts allowed, building against the bundled
/// node via the PATH shim), then re-run the bootability gate. Only that
/// version's own `runtime/versions/<version>/` tree is written; `~/.dsh`
/// and sibling versions are never touched.
pub fn repair_native_bindings(
    app: &AppHandle,
    runtime_dir: &Path,
    version: &str,
    op: &str,
) -> Result<(), String> {
    checked_version_name(version)?;
    if !is_installed(runtime_dir, version) {
        return Err(format!("version {version} is not installed"));
    }
    let dir = version_dir(runtime_dir, version);
    crate::progress::push_console(
        app,
        op,
        "info",
        &format!("$ npm rebuild (with install scripts allowed) in {version}"),
    );
    crate::progress::emit(
        app,
        op,
        Some(version),
        "repairing",
        "Rebuilding native modules…",
        None,
    );
    let mut sink = |stream: &str, line: &str| {
        let line = line.trim();
        if !line.is_empty() {
            crate::progress::push_console(
                app,
                op,
                stream,
                &line.chars().take(600).collect::<String>(),
            );
        }
    };
    run_npm(
        app,
        runtime_dir,
        &[
            "rebuild",
            "--prefix",
            dir.to_string_lossy().as_ref(),
            ALLOW_SCRIPTS_FLAG,
            "--no-color",
            "--loglevel=http",
            "--progress=false",
        ],
        Duration::from_secs(240),
        Some(&mut sink),
    )?;
    verify_harness_tree(app, runtime_dir, version, op)
}

#[cfg(test)]
mod native_tests {
    use super::{ALLOW_SCRIPTS_FLAG, is_native_binding_failure, native_binding_error};

    #[test]
    fn scripts_flag_is_the_documented_opt_in() {
        assert_eq!(ALLOW_SCRIPTS_FLAG, "--dangerously-allow-all-scripts");
    }

    #[test]
    fn classifier_spots_the_alpha2_fs_ext_shape() {
        assert!(is_native_binding_failure(
            "Error: Cannot find module './build/Release/fs_ext.node'\nRequire stack:\n- fs-ext/fs-ext.js"
        ));
        assert!(is_native_binding_failure(
            "dsh: plugin tree failed to load: failed to import loader entry session-persistence-jsonl"
        ));
        assert!(is_native_binding_failure(
            "was compiled against a different Node.js version using NODE_MODULE_VERSION 147"
        ));
        assert!(is_native_binding_failure(
            "ERR_DLOPEN_FAILED NODE_MODULE_VERSION mismatch"
        ));
    }

    #[test]
    fn classifier_ignores_healthy_boot_lines() {
        assert!(!is_native_binding_failure(
            "dsh web: http://127.0.0.1:3081/?token=[CENSORED]"
        ));
        assert!(!is_native_binding_failure(
            "[launcher] starting harness 0.1.2-rc.1 on 127.0.0.1:3081"
        ));
        assert!(!is_native_binding_failure("Engine running"));
        assert!(!is_native_binding_failure(""));
    }

    #[test]
    fn error_names_repair_and_data_safety() {
        let msg = native_binding_error(
            "0.1.3-alpha.2",
            "Cannot find module './build/Release/fs_ext.node'",
        );
        assert!(msg.contains("0.1.3-alpha.2"), "{msg}");
        assert!(msg.contains("fs_ext.node"), "{msg}");
        assert!(msg.contains("roll back"), "{msg}");
        assert!(msg.contains("~/.dsh"), "{msg}");
    }
}
