//! Operation progress: a `launcher://progress` event stream plus per-operation
//! snapshots exposed through `get_status`.
//!
//! Long-running operations (npm install / update / rollback, engine
//! start/stop/restart, port change) emit phase updates so UIs can show a
//! live progress bar and message. Every operation carries a unique key
//! (`op:version` for version operations) so SEVERAL downloads can run at the
//! same time without fighting over one progress slot, one console or one
//! cancel flag: consoles are tagged per operation, cancellation is per
//! operation, and the most recent payload of every in-flight operation is
//! mirrored into `AppState.current_ops` so windows opening mid-operation can
//! render all of them.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub const CONSOLE_MAX_LINES: usize = 600;

/// Sentinel operation key meaning "cancel every in-flight operation".
pub const CANCEL_ALL: &str = "*";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    /// Operation key: "engine" | "port" | "install:<version>" |
    /// "switch:<version>" | "download:<version>" | "update:<version>" |
    /// "rollback:<version>" | "select:<version>". The suffix keeps concurrent
    /// version operations distinguishable.
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
    /// True only while npm is actually running for this operation (its
    /// cancellation flag is polled by `run_npm`). UIs show the terminal stop
    /// button only for stoppable payloads — pure message phases (registry
    /// checks, engine/port ops, verification, notices) have nothing to stop.
    #[serde(default)]
    pub stoppable: bool,
}

/// A single line of an operation console (npm output while downloading).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleLine {
    /// Operation key this line belongs to ("" = launcher-level notice).
    pub op: String,
    /// "out" | "err" | "info" - terminal styling hint.
    pub stream: String,
    /// Line text (UIs must escape before rendering).
    pub text: String,
}

/// Build the unique operation key for a version operation.
pub fn op_key(op: &str, version: Option<&str>) -> String {
    match version {
        Some(v) if !v.is_empty() => format!("{op}:{v}"),
        _ => op.to_string(),
    }
}

/// Append a line to the shared operation console ring buffer and forward it
/// to every listening window so the terminal can scroll live.
pub fn push_console(app: &AppHandle, op: &str, stream: &str, text: &str) {
    let line = ConsoleLine {
        op: op.to_string(),
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

/// Drop the console lines of one operation (or every line when the key is
/// empty) so a retried operation starts with a clean terminal.
pub fn reset_console(app: &AppHandle, op: &str) {
    let st = app.state::<AppState>();
    let mut buf = st.console.lock().unwrap();
    if op.is_empty() {
        buf.clear();
    } else {
        buf.retain(|line| line.op != op);
    }
}

/// Arm cancellation for one operation: the in-flight work aborts once it
/// observes the flag.
pub fn request_cancel(app: &AppHandle, op: &str) {
    let st = app.state::<AppState>();
    let mut set = st.cancel.lock().unwrap();
    if op.is_empty() {
        set.insert(CANCEL_ALL.to_string());
    } else {
        set.insert(op.to_string());
    }
}

/// Clear any pending cancellation request for one operation (called when that
/// operation starts). Also clears a global stop-all so fresh work can run.
pub fn reset_cancel(app: &AppHandle, op: &str) {
    let st = app.state::<AppState>();
    let mut set = st.cancel.lock().unwrap();
    set.remove(op);
    set.remove(CANCEL_ALL);
}

/// Whether the user has asked to stop this operation (or all operations).
pub fn cancel_requested(app: &AppHandle, op: &str) -> bool {
    let st = app.state::<AppState>();
    let set = st.cancel.lock().unwrap();
    set.contains(op) || set.contains(CANCEL_ALL)
}

/// Emit a progress update and record it as the current state of that
/// operation. The legacy single-slot `current_op` keeps mirroring the
/// first (deterministic) in-flight operation for older UIs (overlay).
/// Progress emitted here is never stoppable — use [`emit_stoppable`] for
/// phases where npm is actually running.
pub fn emit(app: &AppHandle, op: &str, version: Option<&str>, phase: &str, message: &str, percent: Option<u8>) {
    emit_stoppable(app, op, version, phase, message, percent, false)
}

/// Like [`emit`], with an explicit `stoppable` flag: true only for phases
/// where npm is running and the user could cancel the download.
pub fn emit_stoppable(app: &AppHandle, op: &str, version: Option<&str>, phase: &str, message: &str, percent: Option<u8>, stoppable: bool) {
    let payload = ProgressPayload {
        op: op.to_string(),
        version: version.map(|s| s.to_string()),
        phase: phase.to_string(),
        message: message.to_string(),
        percent,
        stoppable,
    };
    {
        let st = app.state::<AppState>();
        let mut ops = st.current_ops.lock().unwrap();
        if phase == "failed" {
            // A failed op is terminal for the current moment: drop it so UIs
            // do not stay stuck in a "busy" state after the failure.
            ops.remove(op);
        } else {
            ops.insert(op.to_string(), payload.clone());
        }
        // Legacy single slot: mirror the first in-flight op (BTreeMap keeps
        // the order stable across polls).
        *st.current_op.lock().unwrap() = ops.values().next().cloned();
    }
    let _ = app.emit("launcher://progress", payload);
}

/// Clear the recorded state of one operation (after done/failed/cancelled).
pub fn clear_op(app: &AppHandle, op: &str) {
    let st = app.state::<AppState>();
    {
        let mut ops = st.current_ops.lock().unwrap();
        ops.remove(op);
        *st.current_op.lock().unwrap() = ops.values().next().cloned();
    }
}

/// Clear every recorded in-flight operation (used on engine-wide resets).
#[allow(dead_code)]
pub fn clear(app: &AppHandle) {
    let st = app.state::<AppState>();
    {
        let mut ops = st.current_ops.lock().unwrap();
        ops.clear();
        *st.current_op.lock().unwrap() = None;
    }
}

/// All in-flight operations, ordered by key.
pub fn current_ops(app: &AppHandle) -> Vec<ProgressPayload> {
    app.state::<AppState>().current_ops.lock().unwrap().values().cloned().collect()
}

/// Convenience: emit a "done" phase for an operation, drop its in-flight
/// record and its pending cancellation.
pub fn finish(app: &AppHandle, op: &str, version: Option<&str>, message: &str) {
    emit(app, op, version, "done", message, Some(100));
    clear_op(app, op);
    reset_cancel(app, op);
}

// BTreeMap is re-exported nowhere but used through AppState; keep the import
// honest with a trivial assertion.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_key_suffixes_versions() {
        assert_eq!(op_key("install", Some("0.1.2")), "install:0.1.2");
        assert_eq!(op_key("engine", None), "engine");
        assert_eq!(op_key("update", Some("")), "update");
    }
}