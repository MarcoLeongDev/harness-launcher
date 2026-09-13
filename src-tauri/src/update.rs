//! Harness version comparison + newest-published resolution for manual update
//! checks. Update discovery is manual-only: the user triggers "Get Latest",
//! which compares the active version against the newest PUBLISHED version
//! (every release, pre-releases included — never just the npm `latest`
//! dist-tag, which can lag the actual newest release). There is deliberately
//! no background checker thread (removed; see Harness Auto-Update Checks
//! spec).
use semver::Version;

pub fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |s: &str| Version::parse(s.trim_start_matches('v')).ok();
    match (parse(current), parse(latest)) {
        (Some(a), Some(b)) => b > a,
        _ => current != latest,
    }
}

/// One of three manual-check states for the active vs newest-published
/// versions. A downloaded-but-inactive newest version is reported with a
/// switch hint — the launcher MUST NOT switch for the user.
pub fn update_check_message(active: &str, newest: &str, newest_installed: bool) -> String {
    if !is_newer(active, newest) {
        format!("Harness is up to date (v{active})")
    } else if newest_installed {
        format!(
            "v{newest} is downloaded — switch to it to run the latest (active is still v{active})"
        )
    } else {
        format!("Harness update available: v{active} → v{newest}")
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

    #[test]
    fn check_message_covers_all_three_states() {
        use super::update_check_message;
        // Newest not downloaded: not up to date.
        let msg = update_check_message("0.1.7", "0.1.9", false);
        assert!(msg.contains("0.1.7") && msg.contains("0.1.9"), "{msg}");
        assert!(msg.contains("available"), "{msg}");
        // Newest downloaded but a different version is active: switch hint,
        // never an automatic switch.
        let msg = update_check_message("0.1.7", "0.1.9", true);
        assert!(msg.contains("downloaded"), "{msg}");
        assert!(msg.contains("switch"), "{msg}");
        assert!(msg.contains("0.1.7"), "{msg}");
        // Active is the newest: up to date.
        let msg = update_check_message("0.1.9", "0.1.9", true);
        assert!(msg.contains("up to date"), "{msg}");
        // Active newer than anything published: also up to date.
        let msg = update_check_message("0.1.10", "0.1.9", false);
        assert!(msg.contains("up to date"), "{msg}");
    }
}
