//! Loopback port selection: validation and free-port fallback.
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub fn is_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub fn resolve(desired: u16) -> Result<(u16, bool), String> {
    if desired == 0 {
        return Err("port must be between 1 and 65535".into());
    }
    if is_free(desired) {
        return Ok((desired, false));
    }
    for candidate in desired.saturating_add(1)..=desired.saturating_add(500) {
        if candidate != 0 && is_free(candidate) {
            return Ok((candidate, true));
        }
    }
    Err(format!("no free port found near {desired}"))
}
pub fn wait_until_serving(port: u16, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let ip: std::net::IpAddr = "127.0.0.1".parse().unwrap();
    let addr = std::net::SocketAddr::new(ip, port);
    while std::time::Instant::now() < deadline {
        if TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

/// Read-only diagnosis for a port the engine failed to serve: names the
/// process holding it (macOS `lsof` + `ps`, never signals anything) so the
/// user can free the port or pick another one. Returns None when the port is
/// free or the holder cannot be determined.
pub fn holder_hint(port: u16) -> Option<String> {
    if is_free(port) {
        return None;
    }
    let out = std::process::Command::new("lsof")
        .args(["-ti", &format!("tcp:{port}")])
        .output()
        .ok()?;
    let raw = String::from_utf8_lossy(&out.stdout).to_string();
    let mut holders: Vec<String> = Vec::new();
    for pid in raw.split_whitespace() {
        if pid.chars().all(|c| c.is_ascii_digit()) {
            let name = std::process::Command::new("ps")
                .args(["-o", "comm=", "-p", pid])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| "unknown".into());
            holders.push(format!("pid {pid} ({name})"));
        }
    }
    if holders.is_empty() {
        Some(format!("port {port} is held by another process"))
    } else {
        Some(format!("port {port} is held by {}", holders.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_free_port_unchanged() {
        let (actual, changed) = resolve(12345).unwrap();
        assert_eq!(actual, 12345);
        assert!(!changed);
    }

    #[test]
    fn busy_port_falls_back() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = listener.local_addr().unwrap().port();
        let (actual, changed) = resolve(busy).unwrap();
        assert!(changed);
        assert_ne!(actual, busy);
        // The OS may momentarily re-hand the just-released candidate to a
        // concurrent listener; give it a brief window instead of a hard assert.
        let mut free = false;
        for _ in 0..20 {
            if is_free(actual) {
                free = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        assert!(free, "resolved fallback port {actual} should be free");
    }

    #[test]
    fn rejects_zero() {
        assert!(resolve(0).is_err());
    }

    #[test]
    fn wait_until_serving_ok() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(wait_until_serving(port, Duration::from_secs(1)));
        assert!(!wait_until_serving(1, Duration::from_secs(1)));
    }

    #[test]
    fn holder_hint_reports_but_never_signals() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = listener.local_addr().unwrap().port();
        // lsof may be absent in some environments: only assert when a hint
        // comes back.
        if let Some(hint) = holder_hint(busy) {
            assert!(hint.contains(&format!("port {busy}")), "{hint}");
        }
        assert!(listener.local_addr().is_ok(), "hint must not disturb the holder");
    }
}