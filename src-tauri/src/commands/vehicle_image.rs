use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::error::AppError;
use crate::services::vehicle_image::VehicleImageService;

/// Get the FS25 user profile mods directory.
fn get_mods_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_default()
        .join("My Games")
        .join("FarmingSimulator2025")
        .join("mods")
}

/// Detect the FS25 game installation path by checking common Steam locations.
#[tauri::command]
pub async fn detect_game_path() -> Result<Option<String>, AppError> {
    tauri::async_runtime::spawn_blocking(|| {
        let candidates = vec![
            r"C:\Program Files (x86)\Steam\steamapps\common\Farming Simulator 25",
            r"C:\Program Files\Steam\steamapps\common\Farming Simulator 25",
            r"D:\SteamLibrary\steamapps\common\Farming Simulator 25",
            r"E:\SteamLibrary\steamapps\common\Farming Simulator 25",
            r"D:\Steam\steamapps\common\Farming Simulator 25",
            r"E:\Steam\steamapps\common\Farming Simulator 25",
            r"D:\Games\Steam\steamapps\common\Farming Simulator 25",
            r"E:\Games\Steam\steamapps\common\Farming Simulator 25",
        ];

        for path_str in &candidates {
            let path = PathBuf::from(path_str);
            if path.join("data").join("vehicles").exists() {
                return Ok(Some(path_str.to_string()));
            }
        }

        // Also try reading Steam libraryfolders.vdf for custom library paths
        let steam_config =
            PathBuf::from(r"C:\Program Files (x86)\Steam\config\libraryfolders.vdf");
        if steam_config.exists() {
            if let Ok(content) = std::fs::read_to_string(&steam_config) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("\"path\"") {
                        if let Some(path_str) = line.split('"').nth(3) {
                            let candidate = PathBuf::from(path_str)
                                .join("steamapps\\common\\Farming Simulator 25");
                            if candidate.join("data").join("vehicles").exists() {
                                return Ok(Some(candidate.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    })
    .await
    .map_err(|e| AppError::ImageError {
        message: e.to_string(),
    })?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleImageResult {
    pub filename: String,
    pub image_path: Option<String>,
}

/// Resolve images for a batch of vehicles (async to avoid blocking the UI).
#[tauri::command]
pub async fn get_vehicle_images_batch(
    game_path: String,
    vehicle_filenames: Vec<String>,
    state: State<'_, VehicleImageService>,
) -> Result<Vec<VehicleImageResult>, AppError> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(&game_path);
        let mods_dir = get_mods_dir();
        let results = service.resolve_images_batch(&path, &mods_dir, &vehicle_filenames);
        Ok(results
            .into_iter()
            .map(|(filename, image_path)| VehicleImageResult {
                filename,
                image_path: image_path.map(|p| p.to_string_lossy().to_string()),
            })
            .collect())
    })
    .await
    .map_err(|e| AppError::ImageError {
        message: e.to_string(),
    })?
}

/// Clear the vehicle image cache and return bytes freed.
#[tauri::command]
pub async fn clear_image_cache(
    state: State<'_, VehicleImageService>,
) -> Result<u64, AppError> {
    let service = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || service.clear_cache())
        .await
        .map_err(|e| AppError::ImageError {
            message: e.to_string(),
        })?
}

/// Get the total size of the vehicle image cache in bytes.
#[tauri::command]
pub async fn get_image_cache_size(state: State<'_, VehicleImageService>) -> Result<u64, AppError> {
    let service = state.inner().clone();
    Ok(
        tauri::async_runtime::spawn_blocking(move || service.cache_size())
            .await
            .map_err(|e| AppError::ImageError {
                message: e.to_string(),
            })?,
    )
}
