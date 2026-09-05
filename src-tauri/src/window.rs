//! Windows: the main harness window (loads http://127.0.0.1:<port> and
//! injects the overlay panel) and the launcher-owned Control Panel window
//! served over the `dsh-ui://` custom protocol.
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WindowEvent};

pub const LABEL: &str = "main";
pub const SETTINGS_LABEL: &str = "settings";

pub fn overlay_script() -> &'static str {
    include_str!("../resources/overlay.js")
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
        .build()
        .map_err(|e| e.to_string())?;

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
        .build()
        .map_err(|e| e.to_string())?;

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
