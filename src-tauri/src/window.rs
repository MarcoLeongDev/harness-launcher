//! Windows: the main harness window (loads http://127.0.0.1:<port> and
//! injects the overlay panel) and the launcher-owned Control Panel window
//! served over the `dsh-ui://` custom protocol.
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WindowEvent};

pub const LABEL: &str = "main";
pub const SETTINGS_LABEL: &str = "settings";

/// Green-button fullscreen (macOS): mark a freshly built window
/// fullscreen-primary so a click on the green traffic light enters native
/// fullscreen instead of zooming (Option-click / long-press still zoom).
/// Must run on the main thread (AppKit); the applied behavior bits are
/// logged so launcher.log proves the flag stuck.
#[cfg(target_os = "macos")]
fn apply_fullscreen_primary(app: &AppHandle, window: &tauri::WebviewWindow) {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
    let win = window.clone();
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Ok(ptr) = win.ns_window() else { return };
        // SAFETY: the pointer comes from Tauri for a live window and is used
        // synchronously here while that window exists.
        let ns_window: &NSWindow = unsafe { &*ptr.cast() };
        ns_window.setCollectionBehavior(
            ns_window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenPrimary,
        );
        let actual = ns_window.collectionBehavior();
        crate::settings::log(
            &crate::state::data_dir(&handle),
            &format!("fullscreen green button: collectionBehavior={actual:?}"),
        );
    });
}

#[cfg(all(test, target_os = "macos"))]
mod fullscreen_tests {
    // Guards the exact flag: Primary (green enters fullscreen) must not be
    // confused with Auxiliary (helper window) or None (explicitly blocked).
    #[test]
    fn green_button_flag_is_primary() {
        use objc2_app_kit::NSWindowCollectionBehavior as Behavior;
        assert_eq!(Behavior::FullScreenPrimary.bits(), 1 << 7);
        assert_ne!(Behavior::FullScreenPrimary, Behavior::FullScreenAuxiliary);
        assert_ne!(Behavior::FullScreenPrimary, Behavior::FullScreenNone);
    }
}

#[cfg(test)]
mod findzoom_tests {
    use super::findzoom_script;

    #[test]
    fn script_is_self_contained_and_guarded() {
        let js = findzoom_script();
        assert!(js.len() > 1000, "findzoom.js looks truncated");
        // Single-install guard so re-navigation never double-binds shortcuts.
        assert!(js.contains("__DSH_FINDZOOM__"), "missing install guard");
    }

    #[test]
    fn script_handles_find_shortcut_and_bar() {
        let js = findzoom_script();
        // Cmd (macOS) or Ctrl with F opens the bar; bar has search semantics.
        assert!(js.contains("e.metaKey || e.ctrlKey"), "missing mod-key check");
        assert!(js.contains("dsh-fz-bar"), "missing find bar id");
        assert!(js.contains("role', 'search'") || js.contains("role\",\"search\""),
            "find bar must expose role=search");
        // Next/previous/close affordances.
        for id in ["dsh-fz-prev", "dsh-fz-next", "dsh-fz-close", "dsh-fz-count"] {
            assert!(js.contains(id), "missing find bar element {id}");
        }
    }

    #[test]
    fn script_handles_zoom_keys_and_persistence() {
        let js = findzoom_script();
        // Zoom in/out/reset keys (incl. '=' for Shift-less Cmd+=, numpad codes).
        for key in ["NumpadAdd", "NumpadSubtract", "Numpad0"] {
            assert!(js.contains(key), "missing zoom key {key}");
        }
        // Bounded range persisted per origin so it survives redeploys.
        assert!(js.contains("dsh-zoom"), "missing zoom storage key");
        assert!(js.contains("MIN_ZOOM = 50"), "missing lower zoom bound");
        assert!(js.contains("MAX_ZOOM = 200"), "missing upper zoom bound");
    }
}

pub fn overlay_script() -> &'static str {
    include_str!("../resources/overlay.js")
}

/// Browser-style find-in-page + zoom (Cmd/Ctrl+F, Cmd/Ctrl +/−/0), injected
/// into every webview. Self-contained (no IPC): safe for both launcher-owned
/// pages and untrusted engine-served content.
pub fn findzoom_script() -> &'static str {
    include_str!("../resources/findzoom.js")
}

fn parse_url(url: &str) -> Result<tauri::Url, String> {
    url.parse().map_err(|e: url::ParseError| format!("invalid url {url}: {e}"))
}

fn dsh_ui_url(path: &str) -> Result<WebviewUrl, String> {
    let url: tauri::Url = format!("dsh-ui://localhost/{path}").parse().map_err(|e: url::ParseError| e.to_string())?;
    Ok(WebviewUrl::CustomProtocol(url))
}

pub fn ensure_window(app: &AppHandle, url: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.navigate(parse_url(url)?);
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let url2 = WebviewUrl::External(parse_url(url)?);
    let window = WebviewWindowBuilder::new(app, LABEL, url2)
        .title("Harness Launcher - Harness")
        .inner_size(1280.0, 800.0)
        .min_inner_size(900.0, 600.0)
        .initialization_script(overlay_script())
        .initialization_script(findzoom_script())
        .build()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    apply_fullscreen_primary(app, &window);

    let win = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = win.hide();
        }
    });
    let _ = window.show();
    Ok(())
}

/// Open (or focus) the launcher Control Panel window.
pub fn open_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, SETTINGS_LABEL, dsh_ui_url("settings")?)
        .title("Harness Launcher - Control Panel")
        .inner_size(680.0, 800.0)
        .min_inner_size(560.0, 640.0)
        .initialization_script(findzoom_script())
        .build()
        .map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    apply_fullscreen_primary(app, &window);

    let win = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = win.hide();
        }
    });
    let _ = window.show();
    Ok(())
}

/// Navigate the main window away from a dead harness URL to a launcher-owned
/// page (used when the engine is stopped).
pub fn show_stopped_page(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.navigate(parse_url("dsh-ui://localhost/stopped")?);
        let _ = window.show();
    }
    Ok(())
}

pub fn navigate(app: &AppHandle, url: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.navigate(parse_url(url)?);
        let _ = window.show();
    }
    Ok(())
}
