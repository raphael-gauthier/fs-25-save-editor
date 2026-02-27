use std::path::{Path, PathBuf};

use chrono::Local;
use fs_extra::dir::{self, CopyOptions};

use crate::error::AppError;
use crate::models::backup::BackupInfo;

/// Returns the backups directory for a given savegame path.
/// For savegame1/, backups go in savegame1_backups/
fn backups_dir(savegame_path: &Path) -> PathBuf {
    let dir_name = format!(
        "{}_backups",
        savegame_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    savegame_path.parent().unwrap_or(savegame_path).join(dir_name)
}

/// Calculates the total size of a directory recursively.
fn dir_size(path: &Path) -> Result<u64, AppError> {
    let mut total: u64 = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if ft.is_dir() {
                total += dir_size(&entry.path())?;
            } else {
                total += entry.metadata()?.len();
            }
        }
    }
    Ok(total)
}

/// Creates a full backup of the savegame directory.
pub fn create_backup(savegame_path: &Path) -> Result<BackupInfo, AppError> {
    if !savegame_path.exists() {
        return Err(AppError::SavegameNotFound {
            path: savegame_path.display().to_string(),
        });
    }

    let backups = backups_dir(savegame_path);
    std::fs::create_dir_all(&backups)?;

    let now = Local::now();
    let backup_name = now.format("backup_%Y-%m-%d_%Hh%Mm%Ss").to_string();
    let backup_path = backups.join(&backup_name);

    let mut opts = CopyOptions::new();
    opts.copy_inside = true;
    dir::copy(savegame_path, &backup_path, &opts).map_err(|e| AppError::BackupError {
        message: e.to_string(),
    })?;

    let size_bytes = dir_size(&backup_path)?;

    Ok(BackupInfo {
        name: backup_name,
        path: backup_path.display().to_string(),
        created_at: now.to_rfc3339(),
        size_bytes,
    })
}

/// Lists all existing backups for a savegame, sorted by date descending.
pub fn list_backups(savegame_path: &Path) -> Result<Vec<BackupInfo>, AppError> {
    let backups = backups_dir(savegame_path);

    if !backups.exists() {
        return Ok(Vec::new());
    }

    let mut infos: Vec<BackupInfo> = Vec::new();

    for entry in std::fs::read_dir(&backups)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if !name.starts_with("backup_") || !entry.file_type()?.is_dir() {
            continue;
        }

        let path = entry.path();
        let created_at = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| {
                let datetime: chrono::DateTime<Local> = t.into();
                Some(datetime.to_rfc3339())
            })
            .unwrap_or_default();

        let size_bytes = dir_size(&path)?;

        infos.push(BackupInfo {
            name,
            path: path.display().to_string(),
            created_at,
            size_bytes,
        });
    }

    // Sort by name descending (timestamp in name ensures chronological order)
    infos.sort_by(|a, b| b.name.cmp(&a.name));

    Ok(infos)
}

/// Restores a backup by replacing the savegame content.
/// Creates a safety backup first, then replaces.
pub fn restore_backup(savegame_path: &Path, backup_name: &str) -> Result<(), AppError> {
    let backups = backups_dir(savegame_path);
    let backup_path = backups.join(backup_name);

    if !backup_path.exists() {
        return Err(AppError::BackupError {
            message: backup_name.to_string(),
        });
    }

    // Create a safety backup first
    create_backup(savegame_path)?;

    // Remove current savegame contents
    for entry in std::fs::read_dir(savegame_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }

    // Copy backup contents into savegame directory
    for entry in std::fs::read_dir(&backup_path)? {
        let entry = entry?;
        let dest = savegame_path.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            let mut opts = CopyOptions::new();
            opts.copy_inside = true;
            dir::copy(entry.path(), &dest, &opts).map_err(|e| AppError::BackupError {
                message: e.to_string(),
            })?;
        } else {
            std::fs::copy(entry.path(), dest)?;
        }
    }

    Ok(())
}

/// Deletes a specific backup.
pub fn delete_backup(savegame_path: &Path, backup_name: &str) -> Result<(), AppError> {
    let backups = backups_dir(savegame_path);
    let backup_path = backups.join(backup_name);

    if !backup_path.exists() {
        return Err(AppError::BackupError {
            message: backup_name.to_string(),
        });
    }

    std::fs::remove_dir_all(&backup_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_savegame(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("fs25_backup_test_{}", name));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("careerSavegame.xml"), "<test>data</test>").unwrap();
        fs::write(base.join("farms.xml"), "<farms/>").unwrap();
        fs::write(base.join("vehicles.xml"), "<vehicles/>").unwrap();
        base
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
        let backups = path
            .parent()
            .unwrap()
            .join(format!("{}_backups", path.file_name().unwrap().to_string_lossy()));
        let _ = fs::remove_dir_all(backups);
    }

    #[test]
    fn test_create_backup_creates_directory() {
        let save = setup_temp_savegame("create");
        let info = create_backup(&save).unwrap();
        assert!(PathBuf::from(&info.path).exists());
        assert!(info.name.starts_with("backup_"));
        assert!(info.size_bytes > 0);
        cleanup(&save);
    }

    #[test]
    fn test_create_backup_copies_all_files() {
        let save = setup_temp_savegame("copy");
        let info = create_backup(&save).unwrap();
        let backup_path = PathBuf::from(&info.path);
        assert!(backup_path.join("careerSavegame.xml").exists());
        assert!(backup_path.join("farms.xml").exists());
        assert!(backup_path.join("vehicles.xml").exists());
        cleanup(&save);
    }

    #[test]
    fn test_create_backup_timestamp_format() {
        let save = setup_temp_savegame("timestamp");
        let info = create_backup(&save).unwrap();
        // Format: backup_YYYY-MM-DD_HHhMMmSSs
        let re_pattern = regex_lite::Regex::new(
            r"^backup_\d{4}-\d{2}-\d{2}_\d{2}h\d{2}m\d{2}s$",
        )
        .unwrap();
        assert!(
            re_pattern.is_match(&info.name),
            "Name '{}' doesn't match expected format",
            info.name
        );
        cleanup(&save);
    }

    #[test]
    fn test_list_backups_sorted_by_date() {
        let save = setup_temp_savegame("list");
        create_backup(&save).unwrap();
        // Small delay to get different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1100));
        create_backup(&save).unwrap();

        let list = list_backups(&save).unwrap();
        assert_eq!(list.len(), 2);
        // First should be newest (descending order)
        assert!(list[0].name > list[1].name);
        cleanup(&save);
    }

    #[test]
    fn test_list_backups_empty() {
        let save = setup_temp_savegame("empty_list");
        let list = list_backups(&save).unwrap();
        assert!(list.is_empty());
        cleanup(&save);
    }

    #[test]
    fn test_restore_backup_replaces_files() {
        let save = setup_temp_savegame("restore");

        // Create backup of original state
        let backup = create_backup(&save).unwrap();

        // Modify the savegame
        fs::write(save.join("careerSavegame.xml"), "<modified>new</modified>").unwrap();

        // Restore
        restore_backup(&save, &backup.name).unwrap();

        // Original content should be back
        let content = fs::read_to_string(save.join("careerSavegame.xml")).unwrap();
        assert_eq!(content, "<test>data</test>");
        cleanup(&save);
    }

    #[test]
    fn test_delete_backup_removes_directory() {
        let save = setup_temp_savegame("delete");
        let info = create_backup(&save).unwrap();
        assert!(PathBuf::from(&info.path).exists());

        delete_backup(&save, &info.name).unwrap();
        assert!(!PathBuf::from(&info.path).exists());
        cleanup(&save);
    }

}
