use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::backup::manager;
use crate::error::AppError;
use crate::models::backup::BackupInfo;
use crate::validators::path::validate_savegame_path;

#[tauri::command]
pub fn list_backups(savegame_path: String) -> Result<Vec<BackupInfo>, AppError> {
    let path = validate_savegame_path(&savegame_path)?;
    manager::list_backups(&path)
}

#[tauri::command]
pub fn create_backup(savegame_path: String) -> Result<BackupInfo, AppError> {
    let path = validate_savegame_path(&savegame_path)?;
    manager::create_backup(&path)
}

#[tauri::command]
pub fn restore_backup(savegame_path: String, backup_name: String) -> Result<(), AppError> {
    let path = validate_savegame_path(&savegame_path)?;
    manager::restore_backup(&path, &backup_name)
}

#[tauri::command]
pub fn delete_backup(savegame_path: String, backup_name: String) -> Result<(), AppError> {
    let path = validate_savegame_path(&savegame_path)?;
    manager::delete_backup(&path, &backup_name)
}

#[tauri::command]
pub fn open_backups_folder(app: AppHandle, savegame_path: String) -> Result<(), AppError> {
    let validated = validate_savegame_path(&savegame_path)?;
    let backups_dir = manager::backups_dir_for(&validated);
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
