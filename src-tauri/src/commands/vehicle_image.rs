use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::error::AppError;
use crate::services::vehicle_image::VehicleImageService;

/// Detect the FS25 game installation path by checking common Steam locations.
#[tauri::command]
pub fn detect_game_path() -> Result<Option<String>, AppError> {
    let candidates = vec![
        // Steam default on Windows
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
    let steam_config = PathBuf::from(r"C:\Program Files (x86)\Steam\config\libraryfolders.vdf");
    if steam_config.exists() {
        if let Ok(content) = std::fs::read_to_string(&steam_config) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("\"path\"") {
                    if let Some(path_str) = line.split('"').nth(3) {
                        let candidate =
                            PathBuf::from(path_str).join("steamapps\\common\\Farming Simulator 25");
                        if candidate.join("data").join("vehicles").exists() {
                            return Ok(Some(candidate.to_string_lossy().to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Resolve a single vehicle image. Returns the absolute path to the cached PNG.
#[tauri::command]
pub fn get_vehicle_image(
    game_path: String,
    vehicle_filename: String,
    state: State<'_, VehicleImageService>,
) -> Result<Option<String>, AppError> {
    let path = PathBuf::from(&game_path);
    match state.resolve_image(&path, &vehicle_filename)? {
        Some(p) => Ok(Some(p.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleImageResult {
    pub filename: String,
    pub image_path: Option<String>,
}

/// Resolve images for a batch of vehicles.
#[tauri::command]
pub fn get_vehicle_images_batch(
    game_path: String,
    vehicle_filenames: Vec<String>,
    state: State<'_, VehicleImageService>,
) -> Result<Vec<VehicleImageResult>, AppError> {
    let path = PathBuf::from(&game_path);
    let results = state.resolve_images_batch(&path, &vehicle_filenames);
    Ok(results
        .into_iter()
        .map(|(filename, image_path)| VehicleImageResult {
            filename,
            image_path: image_path.map(|p| p.to_string_lossy().to_string()),
        })
        .collect())
}

/// Clear the vehicle image cache and return bytes freed.
#[tauri::command]
pub fn clear_image_cache(state: State<'_, VehicleImageService>) -> Result<u64, AppError> {
    state.clear_cache()
}

/// Get the total size of the vehicle image cache in bytes.
#[tauri::command]
pub fn get_image_cache_size(state: State<'_, VehicleImageService>) -> u64 {
    state.cache_size()
}
