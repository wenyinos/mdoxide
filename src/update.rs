//! Update checking module for Ferrite.
//!
//! Checks GitHub Releases API for newer versions. Only activated by explicit
//! user action (Settings → Check for Updates). No automatic or background checking.

use serde::Deserialize;
use std::sync::mpsc;

const GITHUB_API: &str = "https://api.github.com/repos/OlaProeis/Ferrite/releases/latest";
const GITHUB_RELEASES_PREFIX: &str = "https://github.com/OlaProeis/Ferrite/releases/";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Types
// ============================================================================

/// Information about a GitHub release.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    /// Version tag (e.g. "v0.2.7")
    pub tag_name: String,
    /// URL to the release page on GitHub
    pub html_url: String,
}

/// Result of an update check.
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// Current version is the latest
    UpToDate,
    /// A newer version is available
    UpdateAvailable {
        version: String,
        release_url: String,
    },
    /// An error occurred during the check
    Error(String),
}

/// State of the update checker (used by the settings panel).
#[derive(Debug, Clone)]
pub enum UpdateState {
    /// Ready to check (initial state)
    Idle,
    /// Currently checking GitHub API
    Checking,
    /// Check completed: up to date
    UpToDate,
    /// Check completed: update available
    UpdateAvailable {
        version: String,
        release_url: String,
    },
    /// Check failed
    Error(String),
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::Idle
    }
}

// ============================================================================
// Version Comparison
// ============================================================================

/// Parse a version string like "v0.2.6" or "0.2.6-hotfix.1" into (major, minor, patch).
fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let v = version.strip_prefix('v').unwrap_or(version);
    // Take only the numeric part before any pre-release suffix
    let numeric = v.split('-').next()?;
    let parts: Vec<&str> = numeric.split('.').collect();
    if parts.len() >= 3 {
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    } else {
        None
    }
}

/// Check if `latest` version is newer than `current` version.
pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some((l_maj, l_min, l_patch)), Some((c_maj, c_min, c_patch))) => {
            (l_maj, l_min, l_patch) > (c_maj, c_min, c_patch)
        }
        _ => false,
    }
}

/// Get the current application version string.
pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

// ============================================================================
// GitHub API Check
// ============================================================================

/// Check GitHub for the latest release (blocking call — run on a background thread).
///
/// Returns `UpdateCheckResult` indicating whether we're up to date, an update
/// is available, or an error occurred.
fn check_for_update_blocking() -> UpdateCheckResult {
    let response = match ureq::get(GITHUB_API)
        .set("User-Agent", &format!("Ferrite/{}", CURRENT_VERSION))
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, _)) => {
            return UpdateCheckResult::Error(format!("GitHub API returned status {}", code));
        }
        Err(e) => {
            return UpdateCheckResult::Error(format!("Network error: {}", e));
        }
    };

    let release: GitHubRelease = match response.into_json() {
        Ok(r) => r,
        Err(e) => {
            return UpdateCheckResult::Error(format!("Failed to parse response: {}", e));
        }
    };

    if is_newer(&release.tag_name, CURRENT_VERSION) {
        let version = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name)
            .to_string();

        // Security: validate that the release URL actually points to our GitHub repo.
        // This prevents a compromised API response from redirecting users to a malicious site.
        // If validation fails, we construct the expected URL ourselves from the tag name.
        let release_url = if release.html_url.starts_with(GITHUB_RELEASES_PREFIX) {
            release.html_url
        } else {
            log::warn!(
                "Update check: html_url '{}' doesn't match expected prefix, using constructed URL",
                release.html_url
            );
            format!("{}tag/{}", GITHUB_RELEASES_PREFIX, release.tag_name)
        };

        UpdateCheckResult::UpdateAvailable {
            version,
            release_url,
        }
    } else {
        UpdateCheckResult::UpToDate
    }
}

/// Spawn a background thread to check for updates.
///
/// Returns a `mpsc::Receiver` that will receive exactly one `UpdateCheckResult`.
pub fn spawn_update_check() -> mpsc::Receiver<UpdateCheckResult> {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("update-checker".to_string())
        .spawn(move || {
            let result = check_for_update_blocking();
            let _ = tx.send(result);
        })
        .expect("Failed to spawn update check thread");
    rx
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_basic() {
        assert_eq!(parse_version("0.2.6"), Some((0, 2, 6)));
        assert_eq!(parse_version("v0.2.6"), Some((0, 2, 6)));
        assert_eq!(parse_version("1.0.0"), Some((1, 0, 0)));
    }

    #[test]
    fn test_parse_version_with_prerelease() {
        assert_eq!(parse_version("0.2.6-hotfix.1"), Some((0, 2, 6)));
        assert_eq!(parse_version("v1.0.0-beta.2"), Some((1, 0, 0)));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("0.2"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_is_newer() {
        // Newer versions
        assert!(is_newer("v0.2.7", "0.2.6"));
        assert!(is_newer("v0.3.0", "0.2.6"));
        assert!(is_newer("v1.0.0", "0.2.99"));
        assert!(is_newer("0.2.7", "0.2.6"));

        // Same version
        assert!(!is_newer("v0.2.6", "0.2.6"));

        // Older versions
        assert!(!is_newer("v0.2.5", "0.2.6"));
        assert!(!is_newer("v0.1.9", "0.2.0"));

        // Pre-release suffix stripped
        assert!(!is_newer("v0.2.6-hotfix.1", "0.2.6"));
    }

    #[test]
    fn test_current_version() {
        let v = current_version();
        assert!(!v.is_empty());
        // Should be parseable
        assert!(parse_version(v).is_some());
    }

    #[test]
    fn test_github_url_validation() {
        // Valid URLs pass through
        let valid = "https://github.com/OlaProeis/Ferrite/releases/tag/v0.2.7";
        assert!(valid.starts_with(GITHUB_RELEASES_PREFIX));

        // Malicious URLs are rejected
        let malicious = "https://evil.com/phishing";
        assert!(!malicious.starts_with(GITHUB_RELEASES_PREFIX));

        // Subtle spoofs are rejected
        let spoof = "https://github.com.evil.com/OlaProeis/Ferrite/releases/tag/v1.0";
        assert!(!spoof.starts_with(GITHUB_RELEASES_PREFIX));
    }
}
