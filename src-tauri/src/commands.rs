//! IPC commands for the overlay panel, settings window and tray.
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::port;
use crate::progress;
use crate::runtime;
use crate::state::{self, AppState};
use crate::update;
use crate::versions;
use crate::window;

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub launcher_version: String,
    pub running: bool,
    /// Engine state machine: "stopped" | "starting" | "running" | "stopping".
    pub engine_phase: String,
    pub active_version: Option<String>,
    pub previous_version: Option<String>,
    pub port: u16,
    pub actual_port: u16,
    pub port_changed: bool,
    pub versions: Vec<String>,
    pub installed_versions: Vec<String>,
    pub latest_remote: Option<String>,
    pub update_available: bool,
    pub include_prerelease: bool,
    pub auto_update_harness: bool,
    pub auto_update_interval_hours: u64,
    pub start_on_launch: bool,
    pub boot_error: Option<String>,
    /// In-flight long-running operation, if any (legacy single slot for the
    /// overlay panel; the first by key when several run at once).
    pub current_op: Option<progress::ProgressPayload>,
    /// Every in-flight long-running operation (parallel downloads each keep
    /// their own progress).
    pub current_ops: Vec<progress::ProgressPayload>,
    /// Terminal lines of the current download (only meaningful while a
    /// version download is in progress).
    pub console: Vec<progress::ConsoleLine>,
    /// Authenticated WebUI URL captured from the engine (`?token=…`), if any.
    pub web_url: Option<String>,
}

pub fn harness_url(actual_port: u16) -> String {
    format!("http://127.0.0.1:{actual_port}")
}

/// Port of a captured harness URL, if it parses.
fn url_port(url: &str) -> Option<u16> {
    url::Url::parse(url).ok()?.port_or_known_default()
}

/// The URL the harness window / browser should open: the authenticated
/// `?token=…` URL the running engine printed at boot when one has been
/// captured for this port, otherwise the plain URL (pre-token engines).
/// With `wait` set, briefly waits for the engine to print the URL right
/// after a (re)start; call only from blocking (non-main) contexts.
pub fn harness_web_url(app: &AppHandle, port: u16, wait: Option<Duration>) -> String {
    let runtime = &app.state::<AppState>().runtime;
    let captured = match wait {
        Some(t) => runtime.wait_for_web_url(t),
        None => runtime.captured_web_url(),
    };
    match captured {
        Some(url) if url_port(&url) == Some(port) => url,
        _ => harness_url(port),
    }
}

fn ensure_runtime_dirs(app: &AppHandle) -> std::io::Result<()> {
    let rd = state::runtime_dir(app);
    std::fs::create_dir_all(rd.join("versions"))?;
    std::fs::create_dir_all(rd.join("logs"))?;
    std::fs::create_dir_all(rd.join("npm-cache"))?;
    Ok(())
}

/// Run a blocking operation, clearing that operation's in-flight record on
/// error so UIs never stay stuck in a "busy" state after a failure. Other
/// concurrently running operations (parallel downloads) are untouched.
fn with_progress_cleanup<T>(app: &AppHandle, op: &str, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    match f() {
        Ok(v) => Ok(v),
        Err(e) => {
            progress::clear_op(app, op);
            Err(e)
        }
    }
}

/// Invalidate the cached version list so the next `get_status` refreshes it.
fn invalidate_version_cache(app: &AppHandle) {
    let st = app.state::<AppState>();
    let mut vc = st.version_cache.lock().unwrap();
    vc.fetched_at = None;
}

/// Start the engine if it is not already running and wait until it serves.
/// Emits progress under `op` and refreshes tray state. Returns the port.
fn start_engine(app: &AppHandle, op: &str) -> Result<u16, String> {
    let settings = state::read_settings(app);
    let version = settings
        .current_version
        .clone()
        .ok_or_else(|| "no active harness version installed".to_string())?;
    let st = app.state::<AppState>();
    let rd = state::runtime_dir(app);
    let actual = *st.effective_port.lock().unwrap();

    if st.runtime.is_running() {
        let _ = st.runtime.mark_phase(runtime::PHASE_RUNNING);
        return Ok(actual);
    }

    progress::emit(app, op, Some(&version), "starting", &format!("Starting engine on 127.0.0.1:{actual}…"), None);
    runtime::start(app, &st.runtime, &rd, &version, actual)?;
    versions::ensure_peer_completion(app, &rd, &version, op)?;
    let served = port::wait_until_serving(actual, Duration::from_secs(30));
    if !served {
        runtime::stop(&st.runtime, &rd);
        let msg = format!("harness did not answer on 127.0.0.1:{actual} within 30s");
        progress::emit(app, op, Some(&version), "failed", &msg, Some(0));
        return Err(msg);
    }
    st.runtime.mark_phase(runtime::PHASE_RUNNING);
    progress::emit(app, op, Some(&version), "running", "Engine running", Some(100));
    crate::tray::refresh(app);
    Ok(actual)
}

/// Restart (or force-restart) the engine and wait until it serves again.
/// Used by version switches, port changes and explicit engine commands.
fn restart_engine(app: &AppHandle, op: &str, version: Option<&str>, force: bool) -> Result<u16, String> {
    let st = app.state::<AppState>();
    let rd = state::runtime_dir(app);
    let version = match version {
        Some(v) => v.to_string(),
        None => state::active_version(app).ok_or_else(|| "no active harness version installed".to_string())?,
    };
    let actual = *st.effective_port.lock().unwrap();

    let had_child = if force {
        progress::emit(app, op, Some(&version), "restarting", "Force-restarting engine…", Some(60));
        runtime::force_stop(&st.runtime, &rd, actual)?
    } else {
        progress::emit(app, op, Some(&version), "restarting", "Restarting engine…", Some(60));
        runtime::stop(&st.runtime, &rd)
    };
    if !had_child && !force {
        progress::emit(app, op, Some(&version), "starting", &format!("Starting engine on 127.0.0.1:{actual}…"), None);
    }

    versions::ensure_peer_completion(app, &rd, &version, op)?;
    runtime::start(app, &st.runtime, &rd, &version, actual)?;
    let served = port::wait_until_serving(actual, Duration::from_secs(30));
    if !served {
        runtime::stop(&st.runtime, &rd);
        let msg = format!("harness did not answer on 127.0.0.1:{actual} within 30s");
        progress::emit(app, op, Some(&version), "failed", &msg, Some(0));
        return Err(msg);
    }
    st.runtime.mark_phase(runtime::PHASE_RUNNING);
    progress::emit(app, op, Some(&version), "running", "Engine running", Some(100));
    crate::tray::refresh(app);
    Ok(actual)
}

/// Human-readable feedback for a switch done while the engine was stopped.
/// The radio in the versions table IS the designed way to switch, so the
/// message must describe what the switch did — never bounce the user back to
/// the Control Panel.
fn switch_feedback(version: &str, was_installed: bool, already_active: bool) -> String {
    if already_active {
        format!("v{version} is already the default version")
    } else if was_installed {
        format!("v{version} is now the default version — press Start to run it")
    } else {
        format!("installed v{version} and set it as the default version")
    }
}

/// Human-readable feedback for a switch done while the engine was running:
/// the launcher stopped it, switched, and started the new version itself.
fn switch_running_feedback(version: &str, port: u16) -> String {
    format!("switched to v{version} — engine running on port {port}")
}

/// Install `version` when missing and make it the active (default) version.
/// This is the switch path behind the versions-table radio, the engine card
/// version dropdown and the update banner. When the engine is running, the
/// launcher stops it, switches, and starts the new version again — the user
/// never has to stop the engine by hand. When stopped, switching only records
/// the default (it never auto-starts).
fn switch_version_inner(app: &AppHandle, version: &str, op: &str) -> Result<String, String> {
    ensure_runtime_dirs(app).map_err(|e| format!("runtime dirs: {e}"))?;
    let st = app.state::<AppState>();
    let was_running = st.runtime.is_running();
    if was_running {
        progress::emit(
            app,
            op,
            Some(version),
            "stopping",
            "Stopping the running engine to switch versions…",
            Some(10),
        );
        runtime::stop(&st.runtime, &state::runtime_dir(app));
    }
    let rd = state::runtime_dir(app);
    let was_installed = versions::is_installed(&rd, version);
    versions::install_version(app, &rd, version, op)?;
    invalidate_version_cache(app);

    let settings = state::read_settings(app);
    let already_active = settings.current_version.as_deref() == Some(version);
    if !already_active {
        let previous = settings.current_version.clone();
        state::update_settings(app, |s| {
            s.previous_version = previous;
            s.current_version = Some(version.to_string());
        });
        crate::tray::refresh(app);
    }

    if was_running {
        let actual = restart_engine(app, op, Some(version), false)?;
        let _ = window::navigate(app, &harness_web_url(app, actual, Some(Duration::from_secs(10))));
        let msg = switch_running_feedback(version, actual);
        progress::finish(app, op, Some(version), &msg);
        return Ok(msg);
    }

    let msg = switch_feedback(version, was_installed, already_active);
    progress::finish(app, op, Some(version), &msg);
    Ok(msg)
}

#[cfg(test)]
mod switch_tests {
    use super::switch_feedback;

    #[test]
    fn feedback_describes_the_switch_not_the_panel() {
        assert_eq!(
            switch_feedback("0.1.2", false, false),
            "installed v0.1.2 and set it as the default version"
        );
        assert_eq!(
            switch_feedback("0.1.2", true, false),
            "v0.1.2 is now the default version — press Start to run it"
        );
        assert_eq!(
            switch_feedback("0.1.2", true, true),
            "v0.1.2 is already the default version"
        );
    }

    #[test]
    fn feedback_never_bounces_the_user_back_to_the_panel() {
        for (was_installed, already_active) in [(false, false), (true, false), (false, true)] {
            let msg = switch_feedback("0.1.0-rc.7", was_installed, already_active);
            assert!(!msg.contains("select it from the Control Panel"), "{msg}");
        }
    }

    #[test]
    fn running_switch_feedback_reports_the_live_engine() {
        use super::switch_running_feedback;
        let msg = switch_running_feedback("0.1.2-alpha.4", 3081);
        assert!(msg.contains("switched to v0.1.2-alpha.4"), "{msg}");
        assert!(msg.contains("running on port 3081"), "{msg}");
        // A running switch never asks the user to do anything by hand.
        assert!(!msg.contains("press Start"), "{msg}");
        assert!(!msg.contains("stop the engine"), "{msg}");
    }
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Result<StatusPayload, String> {
    // Async + spawn_blocking: the version-list refresh shells out to npm and
    // must never block the main thread, or every window freezes for seconds
    // (panel shows no data and button clicks are dropped).
    tauri::async_runtime::spawn_blocking(move || {
    let st = app.state::<AppState>();
    let settings = state::read_settings(&app);
    let status = st.runtime.status();
    let actual = *st.effective_port.lock().unwrap();
    let rd = state::runtime_dir(&app);

    let mut installed: Vec<String> = std::fs::read_dir(rd.join("versions"))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|v| versions::is_installed(&rd, v))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    installed.sort();

    let remote = st.latest_remote.lock().unwrap().clone();
    let update_available = remote
        .as_ref()
        .zip(settings.current_version.as_ref())
        .map(|(latest, active)| update::is_newer(active, latest))
        .unwrap_or(false);

    // Use cached version list to avoid spawning an npm process every 3 s.
    // Refresh is triggered explicitly after installs/updates or on a 60 s TTL.
    let version_list = {
        let mut vc = st.version_cache.lock().unwrap();
        let stale = vc
            .fetched_at
            .map(|t| t.elapsed().as_secs() >= 60)
            .unwrap_or(true);
        if stale || vc.include_prerelease != settings.include_prerelease {
            match versions::list_versions(&app, &rd, settings.include_prerelease) {
                Ok(v) => {
                    vc.versions = v.clone();
                    vc.include_prerelease = settings.include_prerelease;
                    vc.fetched_at = Some(std::time::Instant::now());
                    v
                }
                Err(_) => vc.versions.clone(),
            }
        } else {
            vc.versions.clone()
        }
    };

    let boot_error = st.boot_error.lock().unwrap().clone();
    let current_op = st.current_op.lock().unwrap().clone();
    let current_ops = progress::current_ops(&app);
    let console = progress::console_snapshot(&app);

    Ok(StatusPayload {
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        running: status.running,
        engine_phase: status.phase,
        active_version: settings.current_version.clone(),
        previous_version: settings.previous_version.clone(),
        port: settings.port,
        actual_port: actual,
        port_changed: actual != settings.port,
        versions: version_list,
        installed_versions: installed,
        latest_remote: remote,
        update_available,
        include_prerelease: settings.include_prerelease,
        auto_update_harness: settings.auto_update_harness,
        auto_update_interval_hours: settings.auto_update_interval_hours,
        start_on_launch: settings.start_on_launch,
        boot_error,
        current_op,
        current_ops,
        console,
        web_url: st.runtime.captured_web_url(),
    })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn install_and_switch(app: AppHandle, version: String) -> Result<String, String> {
    let op = progress::op_key("switch", Some(&version));
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, &op, || switch_version_inner(&app, &version, &op))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Download (install) a harness version WITHOUT switching: the active version
/// stays as it is and a running engine is never stopped, started or restarted.
/// Downloading is just downloading — it works while the engine runs, because
/// npm installs into this version's own directory and only the shared
/// (concurrency-safe) npm cache is touched.
#[tauri::command]
pub async fn download_version(app: AppHandle, version: String) -> Result<String, String> {
    let op = progress::op_key("download", Some(&version));
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, &op, || {
            if !versions::is_valid_version_name(&version) {
                return Err(format!("invalid version name: {version}"));
            }
            ensure_runtime_dirs(&app).map_err(|e| format!("runtime dirs: {e}"))?;
            let rd = state::runtime_dir(&app);
            if versions::is_installed(&rd, &version) {
                let msg = format!("v{version} is already installed");
                progress::finish(&app, &op, Some(&version), &msg);
                return Ok(msg);
            }
            versions::install_version(&app, &rd, &version, &op)?;
            invalidate_version_cache(&app);
            let msg = format!(
                "installed v{version} — set it as the default to run it"
            );
            progress::finish(&app, &op, Some(&version), &msg);
            Ok(msg)
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_to_latest(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "update", || {
            let rd = state::runtime_dir(&app);
            progress::emit(&app, "update", None, "registry", "Checking npm registry for latest version…", Some(5));
            let latest = versions::latest_dist_tag(&app, &rd)?;
            *app.state::<AppState>().latest_remote.lock().unwrap() = Some(latest.clone());
            if state::active_version(&app).as_deref() == Some(latest.as_str()) {
                progress::finish(&app, "update", None, &format!("already on latest ({latest})"));
                return Ok(format!("already on latest ({latest})"));
            }
            // Continue under the resolved version's own key so the terminal
            // feed and cancellation target this update precisely.
            let op = progress::op_key("update", Some(&latest));
            progress::clear_op(&app, "update");
            switch_version_inner(&app, &latest, &op)
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn rollback(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let previous = state::read_settings(&app)
            .previous_version
            .ok_or_else(|| "no previous version to roll back to".to_string())?;
        let op = progress::op_key("rollback", Some(&previous));
        with_progress_cleanup(&app, &op, || {
            let rd = state::runtime_dir(&app);
            if !versions::is_installed(&rd, &previous) {
                versions::install_version(&app, &rd, &previous, &op)?;
            }
            let current = state::read_settings(&app).current_version.clone();
            state::update_settings(&app, |s| {
                s.current_version = Some(previous.clone());
                s.previous_version = current;
            });
            progress::emit(&app, &op, Some(&previous), "switching", "Switching active version…", Some(90));
            let actual = restart_engine(&app, &op, Some(&previous), false)?;
            let _ = window::navigate(&app, &harness_url(actual));
            let msg = format!("rolled back to {previous} on port {actual}");
            progress::finish(&app, &op, Some(&previous), &msg);
            Ok(msg)
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_port(app: AppHandle, port: u16) -> Result<String, String> {
    if port == 0 {
        return Err("port must be between 1 and 65535".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "port", || {
        let st = app.state::<AppState>();
        let running = st.runtime.is_running();
        let previous_effective = *st.effective_port.lock().unwrap();
        let previous_desired = state::read_settings(&app).port;

        progress::emit(&app, "port", None, "resolving", &format!("Checking port {port}…"), Some(10));
        let (actual, changed) = port::resolve(port)?;

        state::update_settings(&app, |s| s.port = port);
        *st.effective_port.lock().unwrap() = actual;

        // When the engine is stopped, only persist the port — do NOT auto-start.
        if !running {
            let msg = format!(
                "port set to {port}{} — applies when you start the engine",
                if changed { format!(" (busy, will use {actual})") } else { String::new() }
            );
            progress::finish(&app, "port", None, &msg);
            return Ok(msg);
        }

        progress::emit(&app, "port", None, "restarting", &format!("Restarting engine on port {actual}…"), Some(40));
        match restart_engine(&app, "port", None, false) {
            Ok(actual2) => {
                let _ = window::navigate(&app, &harness_web_url(&app, actual2, Some(Duration::from_secs(10))));
                let msg = format!(
                    "port set to {port}{} — harness on {actual2}",
                    if changed { format!(" (busy, using {actual2})") } else { String::new() }
                );
                progress::finish(&app, "port", None, &msg);
                Ok(msg)
            }
            Err(e) => {
                // Revert the persisted selection and effective port, then try
                // to bring the engine back on the previous effective port.
                state::update_settings(&app, |s| s.port = previous_desired);
                *st.effective_port.lock().unwrap() = previous_effective;
                if let Err(revert_err) = restart_engine(&app, "port", None, false) {
                    progress::emit(&app, "port", None, "failed", &format!("{e}; revert also failed: {revert_err}"), Some(0));
                    return Err(format!("{e}; revert also failed: {revert_err}"));
                }
                progress::emit(&app, "port", None, "failed", &e, Some(0));
                Err(format!("{e} — reverted to port {previous_effective}"))
            }
        }
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_prerelease(app: AppHandle, include: bool) -> Result<String, String> {
    state::update_settings(&app, |s| s.include_prerelease = include);
    Ok(format!("pre-release versions {}", if include { "shown" } else { "hidden" }))
}

#[tauri::command]
pub async fn set_auto_update(app: AppHandle, enabled: Option<bool>, interval_hours: Option<u64>) -> Result<String, String> {
    state::update_settings(&app, |s| {
        if let Some(e) = enabled {
            s.auto_update_harness = e;
        }
        if let Some(h) = interval_hours {
            s.auto_update_interval_hours = h.max(1);
        }
    });
    Ok("auto-update settings saved".into())
}

#[tauri::command]
pub async fn check_updates(app: AppHandle) -> Result<String, String> {
    let s = state::read_settings(&app);
    let app2 = app.clone();
    let harness_part = tauri::async_runtime::spawn_blocking(move || {
        let mut parts = Vec::new();
        let rd = state::runtime_dir(&app2);
        match versions::latest_dist_tag(&app2, &rd) {
            Ok(latest) => {
                *app2.state::<AppState>().latest_remote.lock().unwrap() = Some(latest.clone());
                let active = state::active_version(&app2).unwrap_or_default();
                if update::is_newer(&active, &latest) {
                    parts.push(format!("Harness update available: {active} → {latest}"));
                } else {
                    parts.push(format!("Harness is current at version {latest}"));
                }
            }
            Err(e) => parts.push(format!("harness check failed: {e}")),
        }
        parts
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut parts = harness_part;
    if let Some(endpoint) = s.update_endpoint.clone() {
        match app_updater_check(&app, &endpoint).await {
            Ok(Some(v)) => parts.push(format!("app update available: {v}")),
            Ok(None) => parts.push("app up to date".into()),
            Err(e) => parts.push(format!("app check failed: {e}")),
        }
    } else {
        parts.push("App self-update not configured — set updateEndpoint to enable it".into());
    }
    Ok(parts.join(" "))
}

async fn app_updater_check(app: &AppHandle, endpoint: &str) -> Result<Option<String>, String> {
    // Optional app self-update: official Tauri updater against the configured
    // endpoint (signed artifacts required for macOS). Off by default.
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint
            .parse()
            .map_err(|e: url::ParseError| format!("invalid endpoint: {e}"))?])
        .map_err(|e| format!("updater endpoints: {e}"))?
        .build()
        .map_err(|e| format!("updater build: {e}"))?;
    let update = updater.check().await.map_err(|e| format!("update check failed: {e}"))?;
    Ok(update.map(|u| u.version.to_string()))
}

#[tauri::command]
pub fn tail_logs(app: AppHandle, lines: Option<usize>) -> Result<String, String> {
    let rd = state::runtime_dir(&app);
    let path = rd.join("logs").join("harness.log");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let wanted = lines.unwrap_or(200).max(10);
    let all: Vec<&str> = content.lines().collect();
    let start = all.len().saturating_sub(wanted);
    Ok(all[start..].join("\n"))
}

#[tauri::command]
pub fn open_in_browser(app: AppHandle) -> Result<String, String> {
    let actual = *app.state::<AppState>().effective_port.lock().unwrap();
    let running = app.state::<AppState>().runtime.is_running();
    // A running token engine prints its URL right after binding, so a short
    // wait is enough; a stopped engine opens the plain URL immediately.
    let url = harness_web_url(&app, actual, running.then(|| Duration::from_secs(5)));
    std::process::Command::new("open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("failed to open browser: {e}"))?;
    Ok(format!("opened {url}"))
}

#[tauri::command]
pub async fn restart_harness(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "engine", || {
            let actual = restart_engine(&app, "engine", None, false)?;
            let _ = window::navigate(&app, &harness_web_url(&app, actual, Some(Duration::from_secs(10))));
            progress::clear_op(&app, "engine");
            Ok(format!("harness restarted on port {actual}"))
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---- Engine lifecycle ----

#[tauri::command]
pub async fn engine_start(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "engine", || {
            if app.state::<AppState>().runtime.is_running() {
                return Ok("engine already running".into());
            }
            let actual = start_engine(&app, "engine")?;
            state::update_settings(&app, |s| s.start_on_launch = true);
            let _ = window::navigate(&app, &harness_web_url(&app, actual, Some(Duration::from_secs(10))));
            progress::clear_op(&app, "engine");
            Ok(format!("engine started on port {actual}"))
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn engine_stop(app: AppHandle) -> Result<String, String> {
    let st = app.state::<AppState>();
    let rd = state::runtime_dir(&app);
    if !st.runtime.is_running() {
        return Ok("engine already stopped".into());
    }
    progress::emit(&app, "engine", None, "stopping", "Stopping engine…", Some(10));
    runtime::stop(&st.runtime, &rd);
    state::update_settings(&app, |s| s.start_on_launch = false);
    // Identical strings for the progress event and the command result so the
    // panel's terminal feed does not render the same notification twice.
    progress::finish(&app, "engine", None, "engine stopped");
    runtime::emit_status(&app, &st.runtime);
    crate::tray::refresh(&app);
    Ok("engine stopped".into())
}

#[tauri::command]
pub async fn engine_restart(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "engine", || {
            let actual = restart_engine(&app, "engine", None, false)?;
            let _ = window::navigate(&app, &harness_web_url(&app, actual, Some(Duration::from_secs(10))));
            progress::clear_op(&app, "engine");
            Ok(format!("engine restarted on port {actual}"))
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn engine_force_restart(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        with_progress_cleanup(&app, "engine", || {
            let st = app.state::<AppState>();
            if !st.runtime.is_running() && port::is_free(*st.effective_port.lock().unwrap()) {
                progress::clear_op(&app, "engine");
                return Ok("engine is stopped; use Start to launch it".into());
            }
            let actual = restart_engine(&app, "engine", None, true)?;
            let _ = window::navigate(&app, &harness_web_url(&app, actual, Some(Duration::from_secs(10))));
            progress::clear_op(&app, "engine");
            Ok(format!("engine force-restarted on port {actual}"))
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<String, String> {
    window::open_settings_window(&app)?;
    Ok("settings window opened".into())
}

/// Select the active version WITHOUT starting the engine. Installs it if it is
/// not already present locally, then records it as the current version so the
/// next Start (or auto-launch) runs it. Used by the stopped-state version
/// dropdown so picking a version persists the choice instead of auto-launching.
#[tauri::command]
pub async fn set_version(app: AppHandle, version: String) -> Result<String, String> {
    // Async: selecting a version that is not installed yet shells out to npm
    // (install) and must never block the main thread.
    tauri::async_runtime::spawn_blocking(move || {
        let rd = state::runtime_dir(&app);
        ensure_runtime_dirs(&app).map_err(|e| format!("runtime dirs: {e}"))?;
        let st = app.state::<AppState>();
        if st.runtime.is_running() {
            return Err("stop the engine before switching versions".into());
        }
        let op = progress::op_key("select", Some(&version));
        if !versions::is_installed(&rd, &version) {
            versions::install_version(&app, &rd, &version, &op)?;
        }
        progress::finish(&app, &op, Some(&version), &format!("active version set to {version}"));
        let previous = state::read_settings(&app).current_version.clone();
        state::update_settings(&app, |s| {
            s.previous_version = previous;
            s.current_version = Some(version.clone());
        });
        crate::tray::refresh(&app);
        Ok(format!("active version set to {version}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_harness_window(app: AppHandle) -> Result<String, String> {
    crate::tray::show_main_window(&app)?;
    Ok("harness window opened".into())
}

/// Ask the backend to stop an in-flight download/operation. With `op` set
/// only that operation is targeted (parallel downloads cancel independently);
/// without it every in-flight operation is cancelled. The running npm child
/// is killed as soon as it observes the cancellation flag.
#[tauri::command]
pub fn cancel_operation(app: AppHandle, op: Option<String>) -> Result<String, String> {
    let ops = progress::current_ops(&app);
    match op {
        Some(key) if !key.is_empty() => {
            if !ops.iter().any(|p| p.op == key) {
                return Ok("no operation in progress".into());
            }
            progress::request_cancel(&app, &key);
            Ok("stop requested — the operation will abort shortly".into())
        }
        _ => {
            if ops.is_empty() {
                return Ok("no operation in progress".into());
            }
            progress::request_cancel(&app, "");
            Ok(format!(
                "stop requested for {} operation(s) — they will abort shortly",
                ops.len()
            ))
        }
    }
}

/// Delete an installed harness version directory. Refuses to remove the
/// version that is currently active or running in the engine.
#[tauri::command]
pub async fn delete_version(app: AppHandle, version: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let rd = state::runtime_dir(&app);
        let dir = versions::version_dir(&rd, &version);
        if !dir.exists() {
            return Err(format!("version {version} is not installed"));
        }
        {
            let st = app.state::<AppState>();
            let proc = st.runtime.process.lock().unwrap();
            if let Some(state) = proc.as_ref() {
                if state.version == version {
                    return Err(format!("cannot delete {version}: the engine is currently running it"));
                }
            }
        }
        let settings = state::read_settings(&app);
        if settings.current_version.as_deref() == Some(version.as_str()) {
            return Err(format!("cannot delete {version}: it is the active version"));
        }
        std::fs::remove_dir_all(&dir).map_err(|e| format!("delete {version}: {e}"))?;
        invalidate_version_cache(&app);
        if settings.previous_version.as_deref() == Some(version.as_str()) {
            state::update_settings(&app, |s| s.previous_version = None);
        }
        progress::push_console(&app, "", "info", &format!("Removed installed version {version}"));
        Ok(format!("deleted version {version}"))
    })
    .await
    .map_err(|e| e.to_string())?}

/// Open the installation directory of an installed harness version in the
/// system file manager (Finder). The version name is validated first so it
/// can never escape the versions directory.
#[tauri::command]
pub fn open_version_dir(app: AppHandle, version: String) -> Result<String, String> {
    if !versions::is_valid_version_name(&version) {
        return Err(format!("invalid version name: {version}"));
    }
    let rd = state::runtime_dir(&app);
    let dir = versions::version_dir(&rd, &version);
    if !dir.is_dir() {
        return Err(format!("version {version} is not installed"));
    }
    std::process::Command::new("open")
        .arg(&dir)
        .spawn()
        .map_err(|e| format!("failed to open directory: {e}"))?;
    Ok(format!("opened {}", dir.display()))
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    let st = app.state::<AppState>();
    let _ = runtime::stop(&st.runtime, &state::runtime_dir(&app));
    app.exit(0);
    Ok(())
}
