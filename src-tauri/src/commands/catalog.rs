use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::models::catalog::CatalogVehicle;
use crate::services::catalog::scan_vehicle_catalog;

pub struct CatalogState {
    cache: Mutex<Option<(String, Vec<CatalogVehicle>)>>,
}

impl CatalogState {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }
}

/// Get the FS25 user profile mods directory.
fn get_mods_dir() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_default()
        .join("My Games")
        .join("FarmingSimulator2025")
        .join("mods")
}

#[tauri::command]
pub async fn get_vehicle_catalog(
    game_path: String,
    state: State<'_, CatalogState>,
) -> Result<Vec<CatalogVehicle>, AppError> {
    // Check cache
    {
        let cache = state.cache.lock().unwrap();
        if let Some((cached_path, cached_catalog)) = cache.as_ref() {
            if cached_path == &game_path {
                return Ok(cached_catalog.clone());
            }
        }
    }

    let game_path_clone = game_path.clone();
    let catalog = tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(&game_path_clone);
        let vehicles_dir = path.join("data").join("vehicles");
        if !vehicles_dir.exists() {
            return Err(AppError::IoError {
                message: format!("Vehicles directory not found: {}", vehicles_dir.display()),
            });
        }

        let mods_dir = get_mods_dir();
        Ok(scan_vehicle_catalog(&path, &mods_dir))
    })
    .await
    .map_err(|e| AppError::IoError {
        message: e.to_string(),
    })??;

    // Update cache
    {
        let mut cache = state.cache.lock().unwrap();
        *cache = Some((game_path, catalog.clone()));
    }

    Ok(catalog)
}
