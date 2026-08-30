//! Harness process lifecycle: spawn/stop/restart of
//! `dsh --profile web --no-open --host 127.0.0.1 --port <port>` with the
//! bundled node, log capture with rotation, crash supervision, and engine
//! phase tracking (stopped/starting/running/stopping).
use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use crate::versions;

pub const LOG_MAX_BYTES: u64 = 2 * 1024 * 1024;
pub const TAIL_BUFFER_LINES: usize = 800;

pub const PHASE_STOPPED: &str = "stopped";
pub const PHASE_STARTING: &str = "starting";
pub const PHASE_RUNNING: &str = "running";
pub const PHASE_STOPPING: &str = "stopping";

#[derive(Clone)]
pub struct HarnessRuntime {
    pub log_tail: Arc<Mutex<VecDeque<String>>>,
    pub process: Arc<Mutex<Option<ChildState>>>,
    /// Current engine phase: stopped|starting|running|stopping.
    pub phase: Arc<Mutex<String>>,
    next_id: Arc<AtomicU64>,
}

pub struct ChildState {
    pub child: tauri_plugin_shell::process::CommandChild,
    pub version: String,
    pub port: u16,
    /// Monotonic spawn id; the supervisor only clears the process slot when
    /// the Terminated event matches this id so a fast force-restart is not
    /// clobbered by the old child's late termination event.
    pub id: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessStatus {
    pub running: bool,
    /// Engine state machine: "stopped" | "starting" | "running" | "stopping".
    pub phase: String,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub last_log: String,
}

impl HarnessRuntime {
    pub fn new() -> Self {
        Self {
            log_tail: Arc::new(Mutex::new(VecDeque::new())),
            process: Arc::new(Mutex::new(None)),
            phase: Arc::new(Mutex::new(PHASE_STOPPED.to_string())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn status(&self) -> HarnessStatus {
        let proc = self.process.lock().unwrap();
        let (running, version, port) = match proc.as_ref() {
            Some(p) => (true, Some(p.version.clone()), Some(p.port)),
            None => (false, None, None),
        };
        let phase = self.phase.lock().unwrap().clone();
        let last_log = self
            .log_tail
            .lock()
            .unwrap()
            .iter()
            .rev()
            .next()
            .cloned()
            .unwrap_or_default();
        HarnessStatus { running, phase, version, port, last_log }
    }

    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    pub fn phase(&self) -> String {
        self.phase.lock().unwrap().clone()
    }

    pub fn mark_phase(&self, phase: &str) {
        *self.phase.lock().unwrap() = phase.to_string();
    }
}

fn log_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("logs").join("harness.log")
}

fn rotate_if_needed(runtime_dir: &Path) {
    let path = log_path(runtime_dir);
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > LOG_MAX_BYTES {
            let _ = std::fs::rename(&path, path.with_extension("log.1"));
        }
    }
}

fn append_log(runtime_dir: &Path, line: &str) {
    let _ = std::fs::create_dir_all(runtime_dir.join("logs"));
    rotate_if_needed(runtime_dir);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path(runtime_dir))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Emit a `launcher://status` event with the current engine status.
pub fn emit_status(app: &AppHandle, runtime: &HarnessRuntime) {
    let _ = app.emit("launcher://status", runtime.status());
}

/// Stop the engine: kill the tracked child (SIGKILL via the shell plugin) and
/// clear the process slot immediately. Returns true if a child was running.
pub fn stop(runtime: &HarnessRuntime, runtime_dir: &Path) -> bool {
    let mut proc = runtime.process.lock().unwrap();
    if let Some(state) = proc.take() {
        runtime.mark_phase(PHASE_STOPPING);
        let _ = state.child.kill();
        runtime.mark_phase(PHASE_STOPPED);
        append_log(runtime_dir, "[launcher] harness stopped");
        true
    } else {
        runtime.mark_phase(PHASE_STOPPED);
        false
    }
}

/// Force-stop the engine: kill the child, then verify the port is released and
/// escalate to killing any process still bound to it (macOS `lsof` + SIGKILL).
/// Returns Ok(true) if a child was running, Ok(false) if the engine was already
/// stopped but the port was reclaimed.
pub fn force_stop(
    runtime: &HarnessRuntime,
    runtime_dir: &Path,
    port: u16,
) -> Result<bool, String> {
    let mut proc = runtime.process.lock().unwrap();
    let had_child = if let Some(state) = proc.take() {
        runtime.mark_phase(PHASE_STOPPING);
        state.child.kill().map_err(|e| format!("force-stop kill failed: {e}"))?;
        runtime.mark_phase(PHASE_STOPPED);
        append_log(runtime_dir, "[launcher] force-stopping harness");
        true
    } else {
        runtime.mark_phase(PHASE_STOPPED);
        false
    };
    drop(proc);

    // Give the child a moment to release the port, then escalate.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < deadline {
        if crate::port::is_free(port) {
            return Ok(had_child);
        }
        std::thread::sleep(Duration::from_millis(150));
    }

    if !crate::port::is_free(port) {
        // Escalate: kill anything still listening on the port.
        if let Ok(out) = std::process::Command::new("lsof")
            .args(["-ti", &format!("tcp:{port}")])
            .output()
        {
            let pids = String::from_utf8_lossy(&out.stdout);
            for pid in pids.split_whitespace() {
                let _ = std::process::Command::new("kill").args(["-9", pid]).status();
                append_log(runtime_dir, &format!("[launcher] killed lingering pid {pid} on port {port}"));
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    }
    Ok(had_child)
}

pub fn start(
    app: &AppHandle,
    runtime: &HarnessRuntime,
    runtime_dir: &Path,
    version: &str,
    port: u16,
    host: &str,
) -> Result<(), String> {
    let entry = versions::harness_entry(runtime_dir, version).ok_or_else(|| {
        format!("harness {version} is not installed (missing bin.js)")
    })?;

    let (mut rx_async, child) = app
        .shell()
        .sidecar("node")
        .map_err(|e| format!("node sidecar unavailable: {e}"))?
        .args([
            entry.to_string_lossy().into_owned(),
            "--profile".into(),
            "web".into(),
            "--host".into(),
            host.to_string(),
            "--port".into(),
            port.to_string(),
            "--no-open".into(),
        ])
        .current_dir(runtime_dir)
        .env("npm_config_update_notifier", "false")
        .spawn()
        .map_err(|e| format!("failed to spawn harness: {e}"))?;
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
    let id = runtime.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let (port_n, version_s) = (port, version.to_string());
    runtime.mark_phase(PHASE_STARTING);
    *runtime.process.lock().unwrap() = Some(ChildState {
        child,
        version: version_s.clone(),
        port: port_n,
        id,
    });
    append_log(runtime_dir, &format!("[launcher] starting harness {version} on 127.0.0.1:{port}"));

    let runtime_dir_buf = runtime_dir.to_path_buf();
    let tail = Arc::clone(&runtime.log_tail);
    let proc = Arc::clone(&runtime.process);
    let phase = Arc::clone(&runtime.phase);
    let app_handle = app.clone();
    let version_display = version_s.clone();
    std::thread::spawn(move || {
        loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(CommandEvent::Stdout(line)) | Ok(CommandEvent::Stderr(line)) => {
                    let text = String::from_utf8_lossy(&line);
                    let trimmed = text.trim_end().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    append_log(&runtime_dir_buf, &trimmed);
                    let mut t = tail.lock().unwrap();
                    t.push_back(trimmed);
                    while t.len() > TAIL_BUFFER_LINES {
                        t.pop_front();
                    }
                }
                Ok(CommandEvent::Terminated(payload)) => {
                    let code = payload.code.map(|c| c.to_string()).unwrap_or_else(|| "?".into());
                    append_log(&runtime_dir_buf, &format!("[harness] exited with code {code}"));
                    // Only clear the slot if this event belongs to the current child.
                    let mut guard = proc.lock().unwrap();
                    let matches = matches!(guard.as_ref(), Some(s) if s.id == id);
                    if matches {
                        *guard = None;
                        *phase.lock().unwrap() = PHASE_STOPPED.to_string();
                    }
                    drop(guard);
                    break;
                }
                Ok(CommandEvent::Error(e)) => {
                    append_log(&runtime_dir_buf, &format!("[harness] pipe error: {e}"));
                }
                Ok(_) => {}
                // Heartbeat: the harness is alive but quiet (no output for 30s).
                // Keep waiting — do NOT clear the process slot or report stopped,
                // otherwise a long-running idle engine would be misreported as
                // stopped and restarts would spawn a second harness on the port.
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                // The relay task ended: the child's stream closed (Terminated is
                // normally delivered first). This is real termination.
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        {
            let mut guard = proc.lock().unwrap();
            let matches = matches!(guard.as_ref(), Some(s) if s.id == id);
            if matches {
                *guard = None;
                *phase.lock().unwrap() = PHASE_STOPPED.to_string();
            }
        }
        let _ = app_handle.emit("launcher://status", HarnessStatus {
            running: false,
            phase: PHASE_STOPPED.to_string(),
            version: Some(version_display),
            port: Some(port_n),
            last_log: String::new(),
        });
    });
    Ok(())
}
