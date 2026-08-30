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

    let mut stdout = String::new();
    let mut stderr = String::new();
    loop {
        // Honour an explicit user cancellation (Stop button) as soon as it is
        // observed, killing the child process so the download aborts.
        if crate::progress::cancel_requested(app) {
            let _ = child.kill();
            crate::progress::push_console(app, "err", "Download stopped by user");
            return Err("operation cancelled by user".into());
        }
        match rx.recv_timeout(timeout) {
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
                let _ = child.kill();
                return Err(format!(
                    "npm operation timed out after {}s: {}",
                    timeout.as_secs(),
                    npm_args.join(" ")
                ));
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
    crate::progress::reset_console(app);
    crate::progress::reset_cancel(app);
    crate::progress::push_console(app, "info", &format!("$ npm install {spec}"));
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
            crate::progress::push_console(&app_sink, stream, &text);
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
        &[
            "install",
            "--prefix",
            dir.to_string_lossy().as_ref(),
            "--no-save",
            "--no-audit",
            "--no-fund",
            "--no-color",
            "--prefer-offline",
            "--fetch-retries=1",
            "--fetch-timeout=60000",
            "--loglevel=http",
            "--progress=false",
            spec.as_str(),
        ],
        Duration::from_secs(300),
        Some(&mut throttled_sink),
    )?;
    if crate::progress::cancel_requested(app) {
        crate::progress::push_console(app, "err", "Download stopped by user");
        return Err("operation cancelled by user".into());
    }
    crate::progress::push_console(app, "info", "Install finished — verifying…");
    crate::progress::emit(app, op, Some(version), "verifying", "Verifying installation…", Some(85));
    if !is_installed(runtime_dir, version) {
        return Err(format!(
            "install of {spec} did not produce a usable harness\nstdout: {out}\nstderr: {err}"
        ));
    }
    Ok(())
}