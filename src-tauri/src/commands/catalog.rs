use std::path::PathBuf;
use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::models::catalog::CatalogVehicle;
use crate::services::catalog::scan_vehicle_catalog;
use crate::validators::path::validate_game_path;

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

    let validated_path = validate_game_path(&game_path)?;
    let catalog = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<CatalogVehicle>, AppError> {
        let mods_dir = get_mods_dir();
        Ok(scan_vehicle_catalog(&validated_path, &mods_dir))
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
