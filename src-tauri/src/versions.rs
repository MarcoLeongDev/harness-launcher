//! npm-driven harness version operations. Everything runs through the bundled
//! Node sidecar + vendored npm CLI - no system node/npm/npx needed.
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub const PACKAGE: &str = "@deepseek-ai/dsh";

pub fn npm_cli_path(app: &AppHandle) -> Result<PathBuf, String> {
    let res = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource dir unavailable: {e}"))?;
    let cli = res.join("resources").join("npm").join("bin").join("npm-cli.js");
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
    op: &str,
    npm_args: &[&str],
    timeout: Duration,
    mut sink: Option<&mut dyn FnMut(&str, &str)>,
) -> Result<(String, String), String> {
    let cli = npm_cli_path(app)?;
    let cache = runtime_dir.join("npm-cache");
    let _ = std::fs::create_dir_all(&cache);

    let mut args: Vec<String> = vec![cli.to_string_lossy().into_owned()];
    args.extend(npm_args.iter().map(|s| s.to_string()));
    let (mut rx_async, child) = app
        .shell()
        .sidecar("node")
        .map_err(|e| format!("node sidecar unavailable: {e}"))?
        .args(args)
        .current_dir(runtime_dir)
        .env("npm_config_cache", cache.to_string_lossy().into_owned())
        .env("npm_config_update_notifier", "false")
        .env("npm_config_fund", "false")
        .env("npm_config_audit", "false")
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
        // Honour an explicit user cancellation (Stop button) every 250ms so
        // the UI stays responsive and cancel feels instant.
        if crate::progress::cancel_requested(app, op) {
            let _ = child.kill();
            crate::progress::push_console(app, op, "err", "Download stopped by user");
            return Err("operation cancelled by user".into());
        }
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
                // exceeded.  A short poll_interval means we loop back and check
                // cancel_requested every 250 ms even during silent npm phases.
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
pub fn latest_dist_tag(app: &AppHandle, runtime_dir: &Path) -> Result<String, String> {
    let (out, err) = run_npm(
        app,
        runtime_dir,
        "",
        &["view", PACKAGE, "dist-tags.latest", "--json"],
        Duration::from_secs(30),
        None,
    )?;
    if err.trim().contains("E404") {
        return Err(format!("{PACKAGE} not found on the npm registry"));
    }
    let trimmed = out.trim();
    if trimmed.is_empty() {
        return Err(format!("npm view returned nothing: {err}"));
    }
    let v: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("unexpected npm view output {trimmed:?}: {e}"))?;
    let display = v.to_string();
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|i| i.as_str().map(|s| s.to_string()))
            .last()
            .ok_or_else(|| format!("unexpected dist-tags.latest value: {display}")),
        _ => Err(format!("unexpected dist-tags.latest value: {display}")),
    }
}

pub fn list_versions(
    app: &AppHandle,
    runtime_dir: &Path,
    include_prerelease: bool,
) -> Result<Vec<String>, String> {
    let (out, err) = run_npm(
        app,
        runtime_dir,
        "",
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
}

pub fn harness_entry(runtime_dir: &Path, version: &str) -> Option<PathBuf> {
    let p = version_dir(runtime_dir, version)
        .join("node_modules")
        .join(PACKAGE)
        .join("lib")
        .join("bin.js");
    p.exists().then_some(p)
}

pub fn install_version(
    app: &AppHandle,
    runtime_dir: &Path,
    version: &str,
    op: &str,
) -> Result<(), String> {
    if is_installed(runtime_dir, version) {
        return Ok(());
    }
    let dir = version_dir(runtime_dir, version);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {dir:?}: {e}"))?;
    let spec = format!("{PACKAGE}@{version}");
    crate::progress::reset_console(app, op);
    crate::progress::reset_cancel(app, op);
    crate::progress::push_console(app, op, "info", &format!("$ npm install {spec}"));
    crate::progress::emit(app, op, Some(version), "installing", &format!("Installing {spec}…"), None);
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
        let is_summary = line.starts_with("added ") || line.starts_with("changed ")
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
                crate::progress::emit(&app_sink, &op_sink, Some(&ver_sink), "installing", &preview, None);
            }
        }
    };
    let (out, err) = run_npm(
        app,
        runtime_dir,
        op,
        &[
            "install",
            "--prefix",
            dir.to_string_lossy().as_ref(),
            "--no-save",
            "--no-audit",
            "--no-fund",
            "--no-color",
            "--legacy-peer-deps",
            "--fetch-retries=2",
            "--fetch-timeout=120000",
            "--loglevel=http",
            "--progress=false",
            spec.as_str(),
        ],
        Duration::from_secs(120),
        Some(&mut throttled_sink),
    )?;
    if crate::progress::cancel_requested(app, op) {
        crate::progress::push_console(app, op, "err", "Download stopped by user");
        return Err("operation cancelled by user".into());
    }
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
    crate::progress::emit(app, op, Some(version), "verifying", "Verifying installation…", Some(85));
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
            format!("the harness package at {} is present but incomplete", marker.display())
        };
        return Err(format!(
            "install of {spec} did not produce a usable harness\n{hint}\nstdout: {out}\nstderr: {err}"
        ));
    }
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
        if !pkg_bin.is_dir() { continue; }
        let manifest = pkg_bin.join("package.json");
        if !manifest.exists() { continue; }
        let Ok(text) = std::fs::read_to_string(&manifest) else { continue; };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { continue; };
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
                        peers.entry(name.clone())
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
            crate::progress::push_console(app, op, stream, &line.chars().take(600).collect::<String>());
        }
    };
    run_npm(app, runtime_dir, op, &npm_args, Duration::from_secs(120), Some(&mut sink))?;
    crate::progress::push_console(app, op, "info", "Peer packages complete - verifying...");
    crate::progress::emit(app, op, Some(version), "verifying", "Verifying installation...", Some(95));
    if !is_installed(runtime_dir, version) {
        std::thread::sleep(Duration::from_millis(800));
    }
    if !is_installed(runtime_dir, version) {
        return Err(format!("peer completion did not leave a usable harness for {}", version));
    }
    Ok(())
}


