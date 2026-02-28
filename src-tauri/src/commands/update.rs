use crate::error::AppError;
use crate::models::update::{GitHubRelease, UpdateInfo};
use tauri::AppHandle;

const GITHUB_REPO: &str = "raphael-gauthier/fs-25-save-editor";

/// Check if a new version is available on GitHub Releases.
/// Returns Some(UpdateInfo) if an update is available, None otherwise.
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, AppError> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "fs25-save-editor")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| AppError::Generic(format!("Network error: {}", e)))?;

    // If no release (404) or error, return None silently
    if !response.status().is_success() {
        return Ok(None);
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| AppError::Generic(format!("Release parse error: {}", e)))?;

    // Extract version from tag (strip "v" prefix if present)
    let remote_version = release.tag_name.strip_prefix('v').unwrap_or(&release.tag_name);

    // Current app version
    let current_version = app.package_info().version.to_string();

    // Semver comparison
    if is_newer_version(remote_version, &current_version) {
        // Validate that the release URL points to GitHub to prevent open-redirect attacks
        let expected_prefix = format!("https://github.com/{}/releases/", GITHUB_REPO);
        let release_url = if release.html_url.starts_with(&expected_prefix) {
            release.html_url
        } else {
            expected_prefix
        };

        Ok(Some(UpdateInfo {
            version: remote_version.to_string(),
            name: release
                .name
                .unwrap_or_else(|| format!("v{}", remote_version)),
            body: release.body.unwrap_or_default(),
            release_url,
            published_at: release.published_at,
        }))
    } else {
        Ok(None)
    }
}

/// Compare two semver versions. Returns true if `remote` > `current`.
fn is_newer_version(remote: &str, current: &str) -> bool {
    let parse = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ))
    };

    match (parse(remote), parse(current)) {
        (Some(r), Some(c)) => r > c,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        // Standard: newer version
        assert!(is_newer_version("0.2.0", "0.1.0"));
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.1.1", "0.1.0"));

        // Same version
        assert!(!is_newer_version("0.1.0", "0.1.0"));

        // Older version
        assert!(!is_newer_version("0.0.9", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_is_newer_version_invalid() {
        // Invalid versions → return false
        assert!(!is_newer_version("invalid", "0.1.0"));
        assert!(!is_newer_version("0.1.0", "invalid"));
        assert!(!is_newer_version("1.0", "0.1.0"));
    }
}
