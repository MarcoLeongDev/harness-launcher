//! Persisted launcher settings. Atomic JSON write on change.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 3080;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub port: u16,
    pub host: String,
    pub auto_update_harness: bool,
    pub auto_update_interval_hours: u64,
    pub include_prerelease: bool,
    pub update_endpoint: Option<String>,
    pub current_version: Option<String>,
    pub previous_version: Option<String>,
    pub open_on_launch: bool,
    /// Whether the harness engine should be started automatically on app launch.
    pub start_on_launch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: "127.0.0.1".to_string(),
            auto_update_harness: true,
            auto_update_interval_hours: 6,
            include_prerelease: false,
            update_endpoint: None,
            current_version: None,
            previous_version: None,
            open_on_launch: true,
            start_on_launch: true,
        }
    }
}

pub fn settings_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("settings.json")
}

pub fn load(data_dir: &PathBuf) -> Settings {
    match fs::read_to_string(settings_path(data_dir)) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log(data_dir, &format!("settings.json corrupt, using defaults: {e}"));
            Settings::default()
        }),
        Err(_) => Settings::default(),
    }
}

pub fn save(data_dir: &PathBuf, settings: &Settings) -> std::io::Result<()> {
    let path = settings_path(data_dir);
    let json = serde_json::to_string_pretty(settings)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn log(data_dir: &PathBuf, line: &str) {
    let dir = data_dir.join("logs");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("launcher.log");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let _ = writeln!(f, "{}", timestamp());
        let _ = writeln!(f, "    {line}");
    }
}

fn timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("[+{}s]", now.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = Settings::default();
        assert_eq!(s.port, DEFAULT_PORT);
        assert!(s.auto_update_harness);
        assert!(s.start_on_launch);
        assert_eq!(s.auto_update_interval_hours, 6);
        assert!(!s.include_prerelease);
        assert!(s.current_version.is_none());
    }

    #[test]
    fn roundtrip_preserves_fields() {
        let dir = std::env::temp_dir().join(format!("dsh-settings-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut s = Settings::default();
        s.port = 4101;
        s.current_version = Some("1.2.3".into());
        s.start_on_launch = false;
        save(&dir, &s).unwrap();

        let loaded = load(&dir);
        assert_eq!(loaded.port, 4101);
        assert_eq!(loaded.current_version.as_deref(), Some("1.2.3"));
        assert!(!loaded.start_on_launch);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_file_falls_back() {
        let dir = std::env::temp_dir().join(format!("dsh-settings-corrupt-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(settings_path(&dir), "{ not json !!!").unwrap();
        let s = load(&dir);
        assert_eq!(s.port, DEFAULT_PORT);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
