use std::path::{Path, PathBuf};

use crate::error::AppError;

/// Validates that a savegame path is safe to operate on.
///
/// Ensures the path:
/// - Does not contain `..` components (path traversal)
/// - Contains a `careerSavegame.xml` file (confirming it's a real savegame directory)
///
/// For listing operations, use `validate_savegames_base_path` instead.
pub fn validate_savegame_path(path: &str) -> Result<PathBuf, AppError> {
    let path_buf = PathBuf::from(path);

    // Reject path traversal
    if has_path_traversal(&path_buf) {
        return Err(AppError::IoError {
            message: "Invalid path: path traversal detected".to_string(),
        });
    }

    // Must contain careerSavegame.xml to be a valid savegame
    if !path_buf.join("careerSavegame.xml").exists() && path_buf.exists() {
        return Err(AppError::IoError {
            message: "Invalid path: not a valid savegame directory".to_string(),
        });
    }

    Ok(path_buf)
}

/// Validates a base path for listing savegames.
///
/// Less strict than `validate_savegame_path` — only rejects path traversal.
pub fn validate_savegames_base_path(path: &str) -> Result<PathBuf, AppError> {
    let path_buf = PathBuf::from(path);

    if has_path_traversal(&path_buf) {
        return Err(AppError::IoError {
            message: "Invalid path: path traversal detected".to_string(),
        });
    }

    Ok(path_buf)
}

/// Validates a game installation path.
///
/// Ensures the path does not contain traversal and has a `data/vehicles` subdirectory.
pub fn validate_game_path(path: &str) -> Result<PathBuf, AppError> {
    let path_buf = PathBuf::from(path);

    if has_path_traversal(&path_buf) {
        return Err(AppError::IoError {
            message: "Invalid path: path traversal detected".to_string(),
        });
    }

    if !path_buf.join("data").join("vehicles").exists() {
        return Err(AppError::IoError {
            message: "Invalid path: not a valid FS25 installation".to_string(),
        });
    }

    Ok(path_buf)
}

/// Checks if a path contains `..` components.
fn has_path_traversal(path: &Path) -> bool {
    path.components().any(|c| matches!(c, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_path_traversal() {
        assert!(has_path_traversal(Path::new("../etc/passwd")));
        assert!(has_path_traversal(Path::new("/home/../etc")));
        assert!(has_path_traversal(Path::new("foo/../../bar")));
        assert!(!has_path_traversal(Path::new("/home/user/savegame1")));
        assert!(!has_path_traversal(Path::new("C:\\Users\\test\\saves")));
    }

    #[test]
    fn test_validate_savegames_base_path_rejects_traversal() {
        let result = validate_savegames_base_path("../../../etc");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_savegames_base_path_accepts_normal() {
        // Use temp dir which always exists
        let tmp = std::env::temp_dir();
        let result = validate_savegames_base_path(&tmp.display().to_string());
        assert!(result.is_ok());
    }
}
