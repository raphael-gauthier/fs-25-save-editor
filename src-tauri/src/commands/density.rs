use std::path::PathBuf;

use crate::error::AppError;
use crate::models::density::{DensityEditPayload, FieldDensityData};
use crate::services::density_map;

#[tauri::command]
pub async fn load_field_density_data(
    savegame_path: String,
    game_path: String,
    map_id: String,
) -> Result<Vec<FieldDensityData>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let save_path = PathBuf::from(&savegame_path);
        let game_path = PathBuf::from(&game_path);

        density_map::aggregate_field_data(&save_path, &game_path, &map_id)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}

#[tauri::command]
pub async fn save_density_edits(
    savegame_path: String,
    game_path: String,
    map_id: String,
    edits: Vec<DensityEditPayload>,
) -> Result<Vec<String>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let save_path = PathBuf::from(&savegame_path);
        let game_path = PathBuf::from(&game_path);

        density_map::save_density_edits(&save_path, &game_path, &map_id, &edits)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}
