//! Harness version comparison + optional app self-update (tauri updater).
//! Update discovery is manual-only: the user triggers "Check now", which
//! compares the active version against the registry. There is deliberately no
//! background checker thread (removed; see Harness Auto-Update Checks spec).
use semver::Version;

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