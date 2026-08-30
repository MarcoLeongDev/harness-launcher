//! Harness auto-update checks + optional app self-update (tauri updater).
use std::path::{Path, PathBuf};
use std::time::Duration;

use semver::Version;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

use crate::settings;
use crate::versions;

pub fn latest_remote(app: &AppHandle, runtime_dir: &Path) -> Result<String, String> {
    versions::latest_dist_tag(app, runtime_dir)
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |s: &str| Version::parse(s.trim_start_matches('v')).ok();
    match (parse(current), parse(latest)) {
        (Some(a), Some(b)) => b > a,
        _ => current != latest,
    }
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn same_version_not_newer() {
        assert!(!is_newer("0.1.1-rc.2", "0.1.1-rc.2"));
        assert!(!is_newer("v1.2.3", "1.2.3"));
    }

    #[test]
    fn newer_minor_wins() {
        assert!(is_newer("0.1.1-rc.2", "0.2.0"));
        assert!(is_newer("0.1.0", "0.1.1-rc.2"));
        assert!(is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn prerelease_vs_stable() {
        assert!(!is_newer("0.1.1-rc.2", "0.1.1-rc.1"));
        assert!(is_newer("0.1.1-rc.2", "0.1.1"));
    }

    #[test]
    fn non_semver_strings_compare_by_equality() {
        assert!(!is_newer("latest", "latest"));
        assert!(is_newer("1.2.3", "latest"));
    }
}

pub fn check_harness_update(
    app: &AppHandle,
    runtime_dir: &Path,
    active: &str,
) -> Result<(String, String, bool), String> {
    let latest = latest_remote(app, runtime_dir)?;
    let available = is_newer(active, &latest);
    if available {
        let _ = app
            .notification()
            .builder()
            .title("DeepSeek Harness Launcher - harness update available")
            .body(format!(
                "Version {latest} is available (you are on {active}). Open the launcher panel to update."
            ))
            .show();
    }
    Ok((active.to_string(), latest, available))
}

pub fn spawn_auto_checker(app: AppHandle, runtime_dir: PathBuf) {
    std::thread::spawn(move || loop {
        let data_dir = match app.path().app_data_dir() {
            Ok(d) => d,
            Err(_) => {
                std::thread::sleep(Duration::from_secs(3600));
                continue;
            }
        };
        let s = settings::load(&data_dir);
        if let Some(active) = s.current_version.clone() {
            if s.auto_update_harness {
                match check_harness_update(&app, &runtime_dir, &active) {
                    Ok(_) => {}
                    Err(e) => settings::log(&data_dir, &format!("auto-update check failed: {e}")),
                }
            }
        }
        std::thread::sleep(Duration::from_secs(s.auto_update_interval_hours.max(1) * 3600));
    });
}