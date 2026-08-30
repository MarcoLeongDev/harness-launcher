//! Operation progress: a `launcher://progress` event stream plus the
//! in-flight operation snapshot exposed through `get_status`.
//!
//! Long-running operations (npm install / update / rollback, engine
//! start/stop/restart, port change) emit phase updates so UIs can show a
//! live progress bar and message. The most recent payload is also mirrored
//! into `AppState.current_op` so that windows opening mid-operation can
//! immediately display what is running.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub const CONSOLE_MAX_LINES: usize = 600;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    /// Operation kind: "install" | "update" | "rollback" | "engine" | "port".
    pub op: String,
    /// Target version when the operation is version-related.
    pub version: Option<String>,
    /// Machine-readable phase id, e.g. "registry", "installing", "verifying",
    /// "switching", "starting", "stopping", "restarting", "running",
    /// "stopped", "done", "failed".
    pub phase: String,
    /// Human-readable progress message.
    pub message: String,
    /// 0-100 when known; `None` means indeterminate.
    pub percent: Option<u8>,
}

/// A single line of the operation console (npm output while downloading).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLine {
    /// "out" | "err" | "info" - terminal styling hint.
    pub stream: String,
    /// Line text (UIs must escape before rendering).
    pub text: String,
}

/// Append a line to the shared operation console ring buffer and forward it
/// to every listening window so the terminal can scroll live.
pub fn push_console(app: &AppHandle, stream: &str, text: &str) {
    let line = ConsoleLine {
        stream: stream.to_string(),
        text: text.to_string(),
    };
    {
        let st = app.state::<AppState>();
        let mut buf = st.console.lock().unwrap();
        buf.push_back(line.clone());
        while buf.len() > CONSOLE_MAX_LINES {
            buf.pop_front();
        }
    }
    let _ = app.emit("launcher://console", line);
}

/// Snapshot of the recent console lines (for windows that open mid-operation).
pub fn console_snapshot(app: &AppHandle) -> Vec<ConsoleLine> {
    app.state::<AppState>().console.lock().unwrap().iter().cloned().collect()
}

/// Clear the console ring buffer when a new operation starts, so the
/// terminal only ever shows the current download.
pub fn reset_console(app: &AppHandle) {
    app.state::<AppState>().console.lock().unwrap().clear();
}

/// Arm cancellation: the in-flight long-running operation will abort once it
/// observes the flag.
pub fn request_cancel(app: &AppHandle) {
    app.state::<AppState>().cancel.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Clear any pending cancellation request (called when a new op starts).
pub fn reset_cancel(app: &AppHandle) {
    app.state::<AppState>().cancel.store(false, std::sync::atomic::Ordering::SeqCst);
}

/// Whether the user has asked to stop the in-flight operation.
pub fn cancel_requested(app: &AppHandle) -> bool {
    app.state::<AppState>().cancel_requested()
}

/// Emit a progress update and record it as the current operation.
pub fn emit(app: &AppHandle, op: &str, version: Option<&str>, phase: &str, message: &str, percent: Option<u8>) {
    let payload = ProgressPayload {
        op: op.to_string(),
        version: version.map(|s| s.to_string()),
        phase: phase.to_string(),
        message: message.to_string(),
        percent,
    };
    if phase == "failed" {
        // A failed op is terminal for the current moment: surface the event
        // but clear the recorded op so UIs do not stay disabled forever.
        *app.state::<AppState>().current_op.lock().unwrap() = None;
    } else {
        *app.state::<AppState>().current_op.lock().unwrap() = Some(payload.clone());
    }
    let _ = app.emit("launcher://progress", payload);
}

/// Clear the recorded in-flight operation.
pub fn clear(app: &AppHandle) {
    *app.state::<AppState>().current_op.lock().unwrap() = None;
}

/// Convenience: emit a "done" phase then clear the recorded op.
pub fn finish(app: &AppHandle, op: &str, version: Option<&str>, message: &str) {
    emit(app, op, version, "done", message, Some(100));
    clear(app);
}
