//! Loopback port handling for the CONFIGURED port only: validation, a
//! short-grace availability check, serving wait, and read-only holder
//! diagnosis. There is deliberately no scan for an alternate port — harness
//! 0.1.6-alpha.2+ enforces single-writer session leases, so a second engine
//! started elsewhere would fight this one over the shared `~/.dsh` sessions.
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::errors::AppError;

/// Grace period to wait for the configured port to become free (covers the
/// brief window after our own child was killed on restart). After the grace
/// a busy port is refused — never redirected elsewhere.
pub const PORT_FREE_GRACE: Duration = Duration::from_secs(2);

pub fn is_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Ensure the configured port can be bound, waiting up to `grace` for a
/// just-released port. Returns `Err(AppError::port_in_use(…))` — naming the
/// holder read-only — when it stays occupied. Never picks another port and
/// never signals the holder.
pub fn ensure_free(port: u16, grace: Duration) -> Result<(), AppError> {
    if port == 0 {
        return Err(AppError::InvalidPort);
    }
    let deadline = Instant::now() + grace;
    loop {
        if is_free(port) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(AppError::port_in_use(port, holder_names(port)));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
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
    Some(format!("port {port} is held by {}", holder_names(port)?))
}

/// The holder's identity for a busy port — `pid 42 (node), pid 7 (node)` —
/// without the `port N` prefix (callers embed it in their own sentence).
/// Read-only: lsof + ps, never signals anything. None when free/undetermined.
pub fn holder_names(port: u16) -> Option<String> {
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
        Some("another process".into())
    } else {
        Some(holders.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_free_port_unchanged() {
        // ensure_free is the spawn gate: a free configured port passes with
        // the port itself — there is no alternate-port answer any more.
        ensure_free(12345, Duration::ZERO).unwrap();
    }

    #[test]
    fn busy_port_is_refused_not_redirected() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = listener.local_addr().unwrap().port();
        // Zero grace: the refusal must come back immediately, as a typed
        // port-in-use error, WITHOUT having probed for a neighbour port.
        let err = ensure_free(busy, Duration::ZERO).expect_err("busy port must be refused");
        assert_eq!(err.code(), "port-in-use");
        assert!(err.to_string().contains(&busy.to_string()), "{err}");
        // The refusal must not have disturbed the holder.
        assert!(listener.local_addr().is_ok(), "holder must stay bound");
    }

    #[test]
    fn grace_waits_for_release_then_allows() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = listener.local_addr().unwrap().port();
        // Release the port shortly after the check starts: the grace loop
        // must pick it up instead of failing.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            drop(listener);
        });
        ensure_free(busy, Duration::from_secs(2)).expect("port frees within grace");
    }

    #[test]
    fn rejects_zero() {
        assert_eq!(
            ensure_free(0, Duration::ZERO).unwrap_err().code(),
            "invalid-port"
        );
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
        assert!(
            listener.local_addr().is_ok(),
            "hint must not disturb the holder"
        );
    }

    #[test]
    fn holder_names_never_mentions_port_prefix() {
        // ensure_free embeds holder_names into "port N is held by …" — the
        // helper itself must not repeat the port.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = listener.local_addr().unwrap().port();
        if let Some(names) = holder_names(busy) {
            assert!(!names.contains("port "), "{names}");
        }
    }
}
