use std::path::PathBuf;

use crate::error::AppError;
use crate::models::density::{DensityEditPayload, FieldDensityData, MapFruitType};
use crate::models::field::FarmlandInfo;
use crate::models::placeable::StorageCapacities;
use crate::services::{density_map, farmland, storage};

/// Capacities and accepted fill types of the savegame's storages.
#[tauri::command]
pub async fn load_storage_capacities(
    savegame_path: String,
    game_path: String,
    map_id: String,
) -> Result<StorageCapacities, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        storage::load_storage_capacities(&PathBuf::from(&savegame_path), &PathBuf::from(&game_path), &map_id)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}

/// Area, price and buildings of every farmland of the map.
#[tauri::command]
pub async fn load_farmlands_info(
    savegame_path: String,
    game_path: String,
    map_id: String,
) -> Result<Vec<FarmlandInfo>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        farmland::load_farmlands_info(&PathBuf::from(&savegame_path), &PathBuf::from(&game_path), &map_id)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}

/// Crops the map can grow, with their growth states.
#[tauri::command]
pub async fn load_map_fruit_types(game_path: String, map_id: String) -> Result<Vec<MapFruitType>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        density_map::load_map_fruit_types(&PathBuf::from(&game_path), &map_id)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}

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
    create_backup: bool,
) -> Result<Vec<String>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let save_path = PathBuf::from(&savegame_path);
        let game_path = PathBuf::from(&game_path);

        density_map::save_density_edits(&save_path, &game_path, &map_id, &edits, create_backup)
    })
    .await
    .map_err(|e| AppError::DensityMapError {
        message: format!("Task failed: {}", e),
    })?
}
