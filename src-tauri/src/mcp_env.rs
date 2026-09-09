// MCP secret env bridging for durable engine updates.
//
// Root cause of the 0.1.5-alpha.1 boot loop: user patch layers use
// !!js process.env.NAME for MCP headers (for example x-api-key from
// AGENTQUEUE_API_KEY). A GUI-launched engine inherits a minimal launchd
// environment, so NAME is unset and the engine resolves the header value to
// undefined. JSON.stringify drops undefined, so logs show headers as an
// empty object while schemastery rejects the dict of strings and the whole
// plugin tree fails to load.
//
// This module scans profile patch layers for process.env references and
// injects the missing values into the engine child environment before spawn.
// Resolution order per name: host process env, then the credentials refs
// file, then known local server configs. Unresolvable names fall back to an
// empty string so validation passes and boot continues (the MCP entry has
// failOnStartupError false, so a bad credential degrades to failed tool
// calls, never a bricked engine). Nothing here ever writes user files;
// bridging is env-only and values are only ever logged redacted.
use std::path::Path;

const CREDENTIALS_FILE: &str = ".credentials.yaml";
const PATCH_FILE: &str = "cordis.patch.yml";
const ROOT_FILE: &str = "cordis.yml";

fn is_env_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

// Collect process.env references from one text blob. Supports the dotted
// form (process.env.NAME) and the subscript form
// (process.env["NAME"] / process.env['NAME']).
fn collect_from_text(text: &str, out: &mut Vec<String>) {
    let bytes = text.as_bytes();
    let needle = b"process.env";
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let mut j = i + needle.len();
            if j < bytes.len() && bytes[j] == b'.' {
                j += 1;
                let start = j;
                while j < bytes.len() && is_env_char(bytes[j]) {
                    j += 1;
                }
                if j > start {
                    let name = &text[start..j];
                    if !out.iter().any(|n| n == name) {
                        out.push(name.to_string());
                    }
                }
                i = j;
                continue;
            }
            if j < bytes.len() && bytes[j] == b'[' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j];
                    j += 1;
                    let start = j;
                    while j < bytes.len() && bytes[j] != quote {
                        j += 1;
                    }
                    if j < bytes.len() {
                        let name = &text[start..j];
                        if !name.is_empty()
                            && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                            && !out.iter().any(|n| n == name)
                        {
                            out.push(name.to_string());
                        }
                    }
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
}

// Every env name referenced via process.env in any profile patch layer.
// Reads only; missing dirs/files yield an empty list.
pub fn wanted_env_names(home: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let profiles = home.join("profiles");
    let Ok(entries) = std::fs::read_dir(&profiles) else {
        return out;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        for file in [PATCH_FILE, ROOT_FILE] {
            let path = dir.join(file);
            if let Ok(text) = std::fs::read_to_string(&path) {
                collect_from_text(&text, &mut out);
            }
        }
    }
    out.sort();
    out
}

fn strip_quotes(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 {
        let b = t.as_bytes();
        if (b[0] == b'"' && b[t.len() - 1] == b'"')
            || (b[0] == b'\'' && b[t.len() - 1] == b'\'')
        {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

// Minimal refs parser for the credentials file. Looks only inside the
// top-level refs block: lines of the form "  NAME: value". Never parses
// records or other blocks, so grant payloads are ignored.
fn lookup_credentials_refs(home: &Path, name: &str) -> Option<String> {
    let text = std::fs::read_to_string(home.join(CREDENTIALS_FILE)).ok()?;
    let mut in_refs = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 {
            if trimmed == "refs:" {
                in_refs = true;
            } else if trimmed.ends_with(':') {
                in_refs = false;
            }
            continue;
        }
        if !in_refs {
            continue;
        }
        if let Some(colon) = trimmed.find(':') {
            let key = trimmed[..colon].trim();
            if key == name {
                let value = strip_quotes(trimmed[colon + 1..].trim());
                if !value.is_empty() {
                    return Some(value);
                }
                return None;
            }
        }
    }
    None
}

// Best-effort Agentqueue key: the local server config lives next to the harness home
// harness home parent at ~/projects/agentqueue/source/data/config.json with an
// apiKey string. Derived from home's grandparent so tests can point home at
// a temp dir. Returns None when the file or key is absent.
fn lookup_agentqueue_config(home: &Path) -> Option<String> {
    let root = home.parent()?.to_path_buf();
    let path = root
        .join("projects")
        .join("agentqueue")
        .join("source")
        .join("data")
        .join("config.json");
    let text = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("apiKey")?.as_str().filter(|s| !s.is_empty()).map(|s| s.to_string())
}

fn resolve_secret(home: &Path, name: &str) -> Option<(String, &'static str)> {
    if let Ok(v) = std::env::var(name) {
        if !v.is_empty() {
            return Some((v, "host"));
        }
    }
    if let Some(v) = lookup_credentials_refs(home, name) {
        return Some((v, "credentials"));
    }
    if name == "AGENTQUEUE_API_KEY" {
        if let Some(v) = lookup_agentqueue_config(home) {
            return Some((v, "agentqueue-config"));
        }
    }
    None
}

// Names missing from the host env plus the value to inject. Unresolvable
// names map to an empty string so schemastery dict validation passes and
// boot continues; the entry still has failOnStartupError false, so a bad
// credential only fails tool calls. Callers must log names redacted.
pub fn bridge_missing_env(home: &Path) -> Vec<(String, String, &'static str)> {
    let mut out = Vec::new();
    for name in wanted_env_names(home) {
        if let Ok(v) = std::env::var(&name) {
            if !v.is_empty() {
                continue;
            }
        }
        match resolve_secret(home, &name) {
            Some((v, source)) if !v.is_empty() => out.push((name, v, source)),
            _ => out.push((name, String::new(), "empty-fallback")),
        }
    }
    out
}

// True when engine output shows an MCP loader entry rejected by config
// validation (for example an undefined header value from an unset
// process.env secret). Kept distinct from native-binding failures, which
// share the plugin tree prefix but need a rebuild instead of env help.
pub fn is_mcp_config_failure(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("invalid config") && lower.contains("mcp-")
}

pub fn mcp_config_error(version: &str, detail: &str) -> String {
    let short: String = detail.trim().chars().take(300).collect();
    format!(
        "harness {version} cannot boot: an MCP entry failed config validation ({short}). This happens when a profile patch references process.env.NAME that is unset in the GUI environment (the value resolves to undefined, which logs as an empty object but fails validation). Set the env var or store it in the credentials refs file, then restart. Your sessions and settings are untouched."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn collects_dotted_and_subscript_refs() {
        let mut out = Vec::new();
        collect_from_text(
            "x: !!js process.env.AGENTQUEUE_API_KEY y: process.env[\"OTHER_KEY\"] z: process.env['THIRD']",
            &mut out,
        );
        let set: HashSet<_> = out.into_iter().collect();
        assert!(set.contains("AGENTQUEUE_API_KEY"));
        assert!(set.contains("OTHER_KEY"));
        assert!(set.contains("THIRD"));
    }

    #[test]
    fn wanted_names_scans_patch_layers_only() {
        let base = std::env::temp_dir().join(format!("dsh-mcpenv-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".dsh").join("profiles").join("web")).unwrap();
        std::fs::write(
            base.join(".dsh").join("profiles").join("web").join("cordis.patch.yml"),
            "headers:\n  x-api-key: !!js process.env.AGENTQUEUE_API_KEY\n",
        )
        .unwrap();
        let names = wanted_env_names(&base.join(".dsh"));
        assert_eq!(names, vec!["AGENTQUEUE_API_KEY".to_string()]);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn credentials_refs_lookup_ignores_records() {
        let base = std::env::temp_dir().join(format!("dsh-mcpcred-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(".dsh")).unwrap();
        std::fs::write(
            base.join(".dsh").join(".credentials.yaml"),
            "version: 1\nrefs:\n  AGENTQUEUE_API_KEY: secret-123\nrecords:\n  other:\n    payload: AGENTQUEUE_API_KEY\n",
        )
        .unwrap();
        assert_eq!(
            lookup_credentials_refs(&base.join(".dsh"), "AGENTQUEUE_API_KEY").as_deref(),
            Some("secret-123")
        );
        assert_eq!(lookup_credentials_refs(&base.join(".dsh"), "MISSING"), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn agentqueue_lookup_reads_sibling_config() {
        let base = std::env::temp_dir().join(format!("dsh-mcpaq-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let home = base.join(".dsh");
        let cfg = base.join("projects").join("agentqueue").join("source").join("data");
        std::fs::create_dir_all(&cfg).unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(cfg.join("config.json"), "{\"apiKey\": \"ke-dev-abc\"}").unwrap();
        assert_eq!(
            lookup_agentqueue_config(&home).as_deref(),
            Some("ke-dev-abc")
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn classifier_spots_mcp_invalid_config() {
        assert!(is_mcp_config_failure(
            "failed to apply loader entry mcp-agentqueue (@deepseek-ai/dsh-mcp-client): invalid config"
        ));
        assert!(!is_mcp_config_failure("dsh web: http://127.0.0.1:3081"));
        assert!(!is_mcp_config_failure("Cannot find module './build/Release/fs_ext.node'"));
    }

    #[test]
    fn error_names_repair_and_data_safety() {
        let msg = mcp_config_error("0.1.5-alpha.1", "invalid config mcp-agentqueue");
        assert!(msg.contains("0.1.5-alpha.1"), "{msg}");
        assert!(msg.contains("process.env"), "{msg}");
        assert!(msg.contains("untouched"), "{msg}");
    }
}
