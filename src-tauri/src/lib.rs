//! DeepSeek Harness Launcher \u2014 a Tauri macOS menubar app that installs,
//! updates, versions and manages the DeepSeek Harness background engine.
mod commands;
mod port;
mod presets;
mod progress;
mod runtime;
mod settings;
mod state;
mod tray;
mod update;
mod versions;
mod window;

use std::sync::atomic::Ordering;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::state::AppState;

pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("dsh-ui", |ctx, request| {
            // Launcher-owned pages: the settings/control panel, the "engine
            // stopped" placeholder and the brand logo, served from a custom
            // protocol so they work in dev and prod without the harness server.
            let path = request.uri().path().to_string();
            // Bootstrap icon SVGs, bundled under resources/bootstrap-icons and
            // served from the resource dir at runtime (raw files, never
            // hardcoded; names are validated to prevent path traversal).
            if let Some(name) = path.strip_prefix("/bootstrap-icons/") {
                if name.contains("..") || name.contains('/') {
                    return tauri::http::Response::builder()
                        .status(404)
                        .body(Vec::new())
                        .unwrap();
                }
                if let Ok(rd) = ctx.app_handle().path().resource_dir() {
                    let file = rd.join("resources").join("bootstrap-icons").join(name);
                    if file.is_file() {
                        if let Ok(bytes) = std::fs::read(&file) {
                            return tauri::http::Response::builder()
                                .header("Content-Type", "image/svg+xml")
                                .body(bytes)
                                .unwrap();
                        }
                    }
                }
                return tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap();
            }
            match path.as_str() {
                "/brand/logo.png" => tauri::http::Response::builder()
                    .header("Content-Type", "image/png")
                    .body(include_bytes!("../resources/brand/logo.png").to_vec())
                    .unwrap(),
                "/stopped" => tauri::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(include_str!("../resources/stopped.html").as_bytes().to_vec())
                    .unwrap(),
                _ => tauri::http::Response::builder()
                    .header("Content-Type", "text/html; charset=utf-8")
                    .body(include_str!("../resources/settings.html").as_bytes().to_vec())
                    .unwrap(),
            }
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::install_and_switch,
            commands::download_version,
            commands::update_to_latest,
            commands::rollback,
            commands::set_port,
            commands::set_prerelease,
            commands::set_auto_update,
            commands::check_updates,
            commands::tail_logs,
            commands::open_in_browser,
            commands::restart_harness,
            commands::engine_start,
            commands::engine_stop,
            commands::engine_restart,
            commands::engine_force_restart,
            commands::set_version,
            commands::open_settings,
            commands::open_harness_window,
                    commands::delete_version,
            commands::open_version_dir,
            commands::quit_app
        ])
        .manage(AppState::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let data_dir = state::data_dir(app.handle());
            let saved = settings::load(&data_dir);
            *app.state::<AppState>().settings.lock().unwrap() = saved;
            tray::setup_tray(app.handle())?;
            let handle = app.handle().clone();
            boot(handle.clone());
            update::spawn_auto_checker(handle, state::runtime_dir(app.handle()));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Harness Launcher")
        .run(|app_handle, event| {
            // Stop the harness child on ANY app exit path (tray/panel Quit,
            // AppleScript quit, Cmd+Q) so no orphaned harness process keeps
            // holding the configured port after the launcher exits.
            if let tauri::RunEvent::Exit = event {
                let st = app_handle.state::<AppState>();
                let _ = runtime::stop(&st.runtime, &state::runtime_dir(app_handle));
            }
        });
}

fn boot(app: AppHandle) {
    std::thread::spawn(move || match boot_inner(&app) {
        Ok(actual_port) => {
            app.state::<AppState>().booted.store(true, Ordering::Relaxed);
            // Token engines (0.1.2-alpha.2+) print their authenticated URL
            // right after binding — wait briefly for it so the first window
            // open already carries the launch token. Blocking here is safe:
            // this is a background boot thread, not the main thread.
            let url = commands::harness_web_url(&app, actual_port, Some(Duration::from_secs(10)));
            let settings = state::read_settings(&app);
            let open_on_launch = settings.open_on_launch;
            let engine_running = app.state::<AppState>().runtime.is_running();
            let app2 = app.clone();
            let _ = app.run_on_main_thread(move || {
                if open_on_launch {
                    if engine_running {
                        let _ = window::ensure_window(&app2, &url);
                    } else {
                        let _ = window::show_stopped_page(&app2);
                    }
                }
                let status = app2.state::<AppState>().runtime.status();
                let _ = app2.emit("launcher://status", status);
                tray::refresh(&app2);
            });
        }
        Err(e) => {
            *app.state::<AppState>().boot_error.lock().unwrap() = Some(e.clone());
            settings::log(&state::data_dir(&app), &format!("boot failed: {e}"));
            let _ = app.notification()
                .builder()
                .title("Harness Launcher - boot failed")
                .body(&e)
                .show();
        }
    });
}

fn boot_inner(app: &AppHandle) -> Result<u16, String> {
    let rd = state::runtime_dir(app);
    std::fs::create_dir_all(rd.join("versions"))
        .map_err(|e| format!("create versions dir: {e}"))?;
    std::fs::create_dir_all(rd.join("logs")).map_err(|e| format!("create logs dir: {e}"))?;
    std::fs::create_dir_all(rd.join("npm-cache")).map_err(|e| format!("create npm cache: {e}"))?;

    let has_active = state::active_version(app)
        .map(|v| versions::is_installed(&rd, &v))
        .unwrap_or(false);
    if !has_active {
        let data_dir = state::data_dir(app);
        settings::log(&data_dir, "installing default (latest) harness version…");
        let latest = versions::latest_dist_tag(app, &rd)?;
        let op = crate::progress::op_key("install", Some(&latest));
        versions::install_version(app, &rd, &latest, &op)?;
        {
            let st = app.state::<AppState>();
            let mut vc = st.version_cache.lock().unwrap();
            vc.fetched_at = None;
        }
        state::update_settings(app, |s| s.current_version = Some(latest));
    }
    let version = state::active_version(app).ok_or("no harness version installed")?;

    let settings_snapshot = state::read_settings(app);
    let (actual, _) = port::resolve(settings_snapshot.port)?;
    *app.state::<AppState>().effective_port.lock().unwrap() = actual;

    if !settings_snapshot.start_on_launch {
        settings::log(
            &state::data_dir(app),
            &format!("engine start disabled on launch (start_on_launch=false); stopped on {actual}"),
        );
        return Ok(actual);
    }

    let st = app.state::<AppState>();
    versions::ensure_peer_completion(app, &rd, &version, "boot")?;
    runtime::start(app, &st.runtime, &rd, &version, actual)?;
    let served = port::wait_until_serving(actual, Duration::from_secs(30));
    if !served {
        return Err(format!("harness {version} did not answer on 127.0.0.1:{actual} within 30s"));
    }
    st.runtime.mark_phase(runtime::PHASE_RUNNING);
    settings::log(&state::data_dir(app), &format!("harness {version} serving on 127.0.0.1:{actual}"));
    Ok(actual)
}
