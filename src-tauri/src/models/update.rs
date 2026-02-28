use serde::{Deserialize, Serialize};

/// Simplified response from the GitHub Releases API
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: Option<String>,
}

/// Update information sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// New available version (e.g. "0.2.0")
    pub version: String,
    /// Release name
    pub name: String,
    /// Release body (changelog in markdown)
    pub body: String,
    /// GitHub release page URL
    pub release_url: String,
    /// Publication date
    pub published_at: Option<String>,
}
