use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::backup::manager;
use crate::error::AppError;
use crate::models::backup::BackupInfo;

#[tauri::command]
pub fn list_backups(savegame_path: String) -> Result<Vec<BackupInfo>, AppError> {
    manager::list_backups(&PathBuf::from(savegame_path))
}

#[tauri::command]
pub fn create_backup(savegame_path: String) -> Result<BackupInfo, AppError> {
    manager::create_backup(&PathBuf::from(savegame_path))
}

#[tauri::command]
pub fn restore_backup(savegame_path: String, backup_name: String) -> Result<(), AppError> {
    manager::restore_backup(&PathBuf::from(savegame_path), &backup_name)
}

#[tauri::command]
pub fn delete_backup(savegame_path: String, backup_name: String) -> Result<(), AppError> {
    manager::delete_backup(&PathBuf::from(savegame_path), &backup_name)
}

#[tauri::command]
pub fn open_backups_folder(app: AppHandle, savegame_path: String) -> Result<(), AppError> {
    let backups_dir = manager::backups_dir_for(&PathBuf::from(savegame_path));
    if !backups_dir.exists() {
        return Err(AppError::BackupError {
            message: "Backups directory not found".into(),
        });
    }
    app.opener()
        .open_path(backups_dir.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::IoError {
            message: e.to_string(),
        })?;
    Ok(())
}
