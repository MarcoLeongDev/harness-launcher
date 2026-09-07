//! Persisted launcher settings. Atomic JSON write on change.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 3080;

/// UI language codes offered by the Control Panel switcher and the tray
/// menu. Codes double as locale keys in the panel script.
pub const LANGUAGES: [&str; 5] = ["en", "zh-Hant", "zh-Hans", "ja", "es"];
pub const DEFAULT_LANGUAGE: &str = "en";

/// Allowlist a caller-supplied language code: unknown values (hand-edited
/// settings files, stale payloads) fall back to English.
pub fn normalize_language(lang: &str) -> String {
    let trimmed = lang.trim();
    if LANGUAGES.contains(&trimmed) {
        trimmed.to_string()
    } else {
        DEFAULT_LANGUAGE.to_string()
    }
}

/// launcher.log rotates at 1 MB (SN10): it previously grew without bound.
pub const LAUNCHER_LOG_MAX_BYTES: u64 = 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub port: u16,
    pub include_prerelease: bool,
    pub update_endpoint: Option<String>,
    pub current_version: Option<String>,
    pub previous_version: Option<String>,
    pub open_on_launch: bool,
    /// Whether the harness engine should be started automatically on app launch.
    pub start_on_launch: bool,
    /// UI language for the Control Panel + tray menu ("en" | "zh-Hant" |
    /// "zh-Hans" | "ja" | "es"). Missing in old files (serde default) and
    /// normalized on load, so pre-language installs open in English.
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            include_prerelease: false,
            update_endpoint: None,
            current_version: None,
            previous_version: None,
            open_on_launch: true,
            start_on_launch: true,
            language: DEFAULT_LANGUAGE.into(),
        }
    }
}

pub fn settings_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("settings.json")
}

pub fn load(data_dir: &PathBuf) -> Settings {
    match fs::read_to_string(settings_path(data_dir)) {
        Ok(text) => {
            let mut s: Settings = serde_json::from_str(&text).unwrap_or_else(|e| {
                log(data_dir, &format!("settings.json corrupt, using defaults: {e}"));
                Settings::default()
            });
            s.language = normalize_language(&s.language);
            s
        }
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
    if let Ok(meta) = fs::metadata(&path) {
        if meta.len() > LAUNCHER_LOG_MAX_BYTES {
            let _ = fs::rename(&path, path.with_extension("log.1"));
        }
    }
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
        assert!(s.start_on_launch);
        assert!(!s.include_prerelease);
        assert!(s.current_version.is_none());
        assert_eq!(s.language, "en");
    }

    #[test]
    fn normalize_language_allowlists() {
        assert_eq!(normalize_language("ja"), "ja");
        assert_eq!(normalize_language(" zh-Hant "), "zh-Hant");
        assert_eq!(normalize_language("fr"), "en");
        assert_eq!(normalize_language(""), "en");
    }

    #[test]
    fn language_roundtrips_and_old_files_default_to_english() {
        let dir = std::env::temp_dir().join(format!("dsh-settings-lang-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut s = Settings::default();
        s.language = "ja".into();
        save(&dir, &s).unwrap();
        assert_eq!(load(&dir).language, "ja");
        // Pre-language file (no `language` key): serde default + normalize.
        let _ = std::fs::write(settings_path(&dir), r#"{"port": 3080}"#).unwrap();
        assert_eq!(load(&dir).language, "en");
        // Hand-edited bogus value falls back to English.
        let _ = std::fs::write(settings_path(&dir), r#"{"language": "xx"}"#).unwrap();
        assert_eq!(load(&dir).language, "en");
        let _ = std::fs::remove_dir_all(&dir);
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

    #[test]
    fn launcher_log_rotates() {
        let dir = std::env::temp_dir().join(format!("dsh-settings-logrot-{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("logs"));
        // Pre-fill past the cap, then log once: the old file must move aside.
        let big = "x".repeat((LAUNCHER_LOG_MAX_BYTES + 1024) as usize);
        std::fs::write(dir.join("logs").join("launcher.log"), big).unwrap();
        log(&dir, "after rotation");
        assert!(dir.join("logs").join("launcher.log.1").exists());
        let cur = std::fs::read_to_string(dir.join("logs").join("launcher.log")).unwrap();
        assert!(cur.contains("after rotation"), "{cur}");
        assert!((cur.len() as u64) < LAUNCHER_LOG_MAX_BYTES);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
