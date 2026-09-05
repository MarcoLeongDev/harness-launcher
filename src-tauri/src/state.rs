//! Shared application state + path helpers.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use tauri::Manager;

use crate::progress::ProgressPayload;
use crate::runtime::HarnessRuntime;
use crate::settings::{self, Settings};

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub effective_port: Mutex<u16>,
    pub runtime: HarnessRuntime,
    pub booted: AtomicBool,
    pub boot_error: Mutex<Option<String>>,
    pub latest_remote: Mutex<Option<String>>,
    /// The in-flight long-running operation (install/update/rollback/engine/port).
    /// Legacy single slot mirroring the first entry of `current_ops` for the
    /// overlay panel; newer UIs read `current_ops` directly.
    pub current_op: Mutex<Option<ProgressPayload>>,
    /// Every in-flight long-running operation, keyed by its unique operation
    /// key ("install:<version>", "switch:<version>", …) so parallel version
    /// downloads each keep their own progress.
    pub current_ops: Mutex<BTreeMap<String, ProgressPayload>>,
    /// Ring buffer of the most recent npm/operation console lines, surfaced
    /// to the Control Panel and overlay as a "terminal" while downloading.
    /// Lines are tagged with their operation key.
    pub console: Mutex<VecDeque<crate::progress::ConsoleLine>>,
    /// Tray menu items that reflect engine state (set by tray::setup_tray).
    pub tray_state: Mutex<Option<crate::tray::TrayState>>,
    /// Cached remote version list to avoid spawning npm every 3 s.
    pub version_cache: Mutex<VersionCache>,
    /// Serializes version-set mutations (install/switch/download/rollback/
    /// set/delete) so concurrent operations cannot interleave directory,
    /// engine or settings changes (SN12). Held only by the outermost IPC
    /// entry bodies — never nested — so queued operations simply wait.
    pub version_mutation: Mutex<()>,
}

pub struct VersionCache {
    pub versions: Vec<String>,
    pub include_prerelease: bool,
    pub fetched_at: Option<std::time::Instant>,
}

impl Default for VersionCache {
    fn default() -> Self {
        Self {
            versions: Vec::new(),
            include_prerelease: false,
            fetched_at: None,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(Settings::default()),
            effective_port: Mutex::new(settings::DEFAULT_PORT),
            runtime: HarnessRuntime::new(),
            booted: AtomicBool::new(false),
            boot_error: Mutex::new(None),
            latest_remote: Mutex::new(None),
            current_op: Mutex::new(None),
            current_ops: Mutex::new(BTreeMap::new()),
            console: Mutex::new(VecDeque::new()),
                tray_state: Mutex::new(None),
            version_cache: Mutex::new(VersionCache::default()),
            version_mutation: Mutex::new(()),
        }
    }
}



pub fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn runtime_dir(app: &tauri::AppHandle) -> PathBuf {
    data_dir(app).join("runtime")
}

pub fn read_settings(app: &tauri::AppHandle) -> Settings {
    app.state::<AppState>().settings.lock().unwrap().clone()
}

pub fn update_settings(app: &tauri::AppHandle, mutate: impl FnOnce(&mut Settings)) {
    let state = app.state::<AppState>();
    let mut guard = state.settings.lock().unwrap();
    mutate(&mut guard);
    let snapshot = guard.clone();
    drop(guard);
    let _ = settings::save(&data_dir(app), &snapshot);
}

pub fn active_version(app: &tauri::AppHandle) -> Option<String> {
    read_settings(app).current_version
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn version_mutation_lock_excludes_concurrent_holders() {
        let state = Arc::new(AppState::default());
        let counter = Arc::new(AtomicUsize::new(0));
        let max_seen = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let (s, c, m) = (state.clone(), counter.clone(), max_seen.clone());
            handles.push(std::thread::spawn(move || {
                for _ in 0..50 {
                    let _guard = s.version_mutation.lock().unwrap();
                    let n = c.fetch_add(1, Ordering::SeqCst) + 1;
                    m.fetch_max(n, Ordering::SeqCst);
                    std::thread::yield_now();
                    let left = c.fetch_sub(1, Ordering::SeqCst) - 1;
                    assert_eq!(left, 0, "two mutation bodies overlapped");
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(max_seen.load(Ordering::SeqCst), 1);
    }
}
