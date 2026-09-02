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

use crate::presets;
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
    /// Authenticated WebUI URL the harness printed at boot
    /// ("dsh web: http://127.0.0.1:<port>/?token=…"). Harness 0.1.2-alpha.2+
    /// gates the WebUI behind a per-process launch token, so windows must open
    /// this URL instead of the plain one. None for older engines.
    pub web_url: Arc<Mutex<Option<String>>>,
    next_id: Arc<AtomicU64>,
}

/// Extract the WebUI URL the harness printed on one stdout/stderr line.
/// Harness prints: `dsh web: http://127.0.0.1:<port>/?token=<token>`
/// optionally followed by ` (LAN: http://…)`. Token engines (0.1.2-alpha.2+)
/// print the authenticated URL; pre-token engines print a plain loopback URL,
/// which is stored as-is (it equals the constructed fallback). Only loopback
/// URLs are accepted — the LAN variant is never what our windows should open.
pub fn parse_web_url_line(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("dsh web:")?.trim();
    let candidate = rest.split_whitespace().next()?.trim().to_string();
    if candidate.starts_with("http://127.0.0.1:") || candidate.starts_with("http://localhost:") {
        Some(candidate)
    } else {
        None
    }
}

impl HarnessRuntime {
    /// The captured authenticated WebUI URL, if any.
    pub fn captured_web_url(&self) -> Option<String> {
        self.web_url.lock().unwrap().clone()
    }

    /// Wait up to `timeout` for the harness to print its authenticated URL
    /// (it is emitted right after the web server binds). Returns the captured
    /// URL or None on timeout.
    pub fn wait_for_web_url(&self, timeout: Duration) -> Option<String> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Some(url) = self.captured_web_url() {
                return Some(url);
            }
            if std::time::Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn clear_web_url(&self) {
        *self.web_url.lock().unwrap() = None;
    }
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
            web_url: Arc::new(Mutex::new(None)),
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
        runtime.clear_web_url();
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
        runtime.clear_web_url();
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
) -> Result<(), String> {
    let entry = versions::harness_entry(runtime_dir, version).ok_or_else(|| {
        format!("harness {version} is not installed (missing bin.js)")
    })?;

    // Reconcile the shared harness home with the presets THIS engine ships
    // before it spawns: a default recorded by another version (rc "code" vs
    // alpha "standard"…) would make every new session fail to resolve, and
    // sessions created under rc engines reference a preset alpha removed.
    {
        let log_dir = runtime_dir.to_path_buf();
        let app_sink = app.clone();
        presets::repair_preset_compatibility(&versions::version_dir(runtime_dir, version), move |line| {
            append_log(&log_dir, &format!("[launcher] {line}"));
            crate::progress::push_console(&app_sink, "", "info", &format!("[launcher] {line}"));
        });
    }

    let (mut rx_async, child) = app
        .shell()
        .sidecar("node")
        .map_err(|e| format!("node sidecar unavailable: {e}"))?
        .args([
            entry.to_string_lossy().into_owned(),
            "--profile".into(),
            "web".into(),
            "--host".into(),
            "127.0.0.1".to_string(),
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
    // A new process mints a new launch token: drop any stale captured URL so
    // callers can never navigate to the previous process's token.
    runtime.clear_web_url();
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
    let web_url = Arc::clone(&runtime.web_url);
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
                    // Capture the authenticated WebUI URL the harness prints
                    // at boot (token engines) and surface it to open windows.
                    if let Some(url) = parse_web_url_line(&trimmed) {
                        *web_url.lock().unwrap() = Some(url.clone());
                        let _ = app_handle.emit("launcher://status", HarnessStatus {
                            running: true,
                            phase: PHASE_RUNNING.to_string(),
                            version: Some(version_display.clone()),
                            port: Some(port_n),
                            last_log: trimmed.clone(),
                        });
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

#[cfg(test)]
mod tests {
    use super::parse_web_url_line;

    #[test]
    fn parses_token_url_line() {
        assert_eq!(
            parse_web_url_line("dsh web: http://127.0.0.1:3081/?token=test-token-0123456789abcdefghijklmnopqrstuvwxyz"),
            Some("http://127.0.0.1:3081/?token=test-token-0123456789abcdefghijklmnopqrstuvwxyz".to_string())
        );
    }

    #[test]
    fn parses_token_url_line_with_lan_suffix() {
        assert_eq!(
            parse_web_url_line("dsh web: http://127.0.0.1:3081/?token=abc_DEF-123 (LAN: http://192.168.1.5:3081/?token=abc_DEF-123)"),
            Some("http://127.0.0.1:3081/?token=abc_DEF-123".to_string())
        );
    }

    #[test]
    fn plain_url_line_is_stored_verbatim() {
        // Pre-token engines (≤ 0.1.1-rc.x) print the plain loopback URL; the
        // printed value is authoritative, so it is captured as-is.
        assert_eq!(
            parse_web_url_line("dsh web: http://127.0.0.1:3081"),
            Some("http://127.0.0.1:3081".to_string())
        );
    }

    #[test]
    fn lan_url_line_yields_none() {
        // A LAN-prefixed line alone (never our bind shape) must not be used.
        assert_eq!(parse_web_url_line("dsh web: http://192.168.1.5:3081/?token=abc"), None);
    }

    #[test]
    fn ignores_unrelated_lines() {
        assert_eq!(parse_web_url_line("[launcher] starting harness 0.1.2-alpha.3 on 127.0.0.1:3081"), None);
        assert_eq!(parse_web_url_line(""), None);
        assert_eq!(parse_web_url_line("listening on http://127.0.0.1:3081"), None);
    }

    #[test]
    fn tolerates_leading_whitespace() {
        assert_eq!(
            parse_web_url_line("  dsh web: http://127.0.0.1:9/?token=x"),
            Some("http://127.0.0.1:9/?token=x".to_string())
        );
    }
}
