use std::path::PathBuf;

use crate::backup::manager as backup_manager;
use crate::error::AppError;
use crate::models::career::SavegameSummary;
use crate::models::changes::{SavegameChanges, SaveResult};
use crate::models::common::LocalizedMessage;
use crate::models::SavegameData;
use crate::parsers::career::{parse_career, parse_career_summary};
use crate::parsers::farm::parse_farms;
use crate::parsers::environment::parse_environment;
use crate::parsers::field::{parse_farmlands, parse_fields};
use crate::parsers::sale::parse_sales;
use crate::parsers::collectible::parse_collectibles;
use crate::parsers::contract::parse_contract_settings;
use crate::parsers::mission::parse_missions;
use crate::parsers::placeable::parse_placeables;
use crate::parsers::vehicle::parse_vehicles;
use crate::validators::path::{validate_savegame_path, validate_savegames_base_path};
use crate::validators::savegame::validate_savegame;
use crate::writers;

/// Returns the default FarmingSimulator2025 save folder path based on the OS.
fn default_savegame_path() -> Result<PathBuf, AppError> {
    #[cfg(target_os = "windows")]
    {
        let docs = dirs::document_dir().ok_or_else(|| AppError::IoError {
            message: "Cannot find Documents folder".to_string(),
        })?;
        Ok(docs.join("My Games").join("FarmingSimulator2025"))
    }

    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or_else(|| AppError::IoError {
            message: "Cannot find home folder".to_string(),
        })?;
        Ok(home
            .join("Library")
            .join("Application Support")
            .join("FarmingSimulator2025"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or_else(|| AppError::IoError {
            message: "Cannot find home folder".to_string(),
        })?;
        Ok(home
            .join(".local")
            .join("share")
            .join("FarmingSimulator2025"))
    }
}

#[tauri::command]
pub fn list_savegames(custom_path: Option<String>) -> Result<Vec<SavegameSummary>, AppError> {
    let base_path = match custom_path {
        Some(p) => validate_savegames_base_path(&p)?,
        None => default_savegame_path()?,
    };

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&base_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with("savegame")
                && entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
        })
        .collect();

    // Sort by modification time descending
    entries.sort_by(|a, b| {
        let time_a = a
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = b
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    let summaries: Vec<SavegameSummary> = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.join("careerSavegame.xml").exists() {
                parse_career_summary(&path).ok()
            } else {
                None
            }
        })
        .collect();

    Ok(summaries)
}

#[tauri::command]
pub fn load_savegame(path: String) -> Result<SavegameData, AppError> {
    let save_path = validate_savegame_path(&path).map_err(|_| AppError::SavegameNotFound {
        path: path.clone(),
    })?;

    if !save_path.exists() {
        return Err(AppError::SavegameNotFound { path });
    }

    let mut warnings: Vec<LocalizedMessage> = Vec::new();

    // Parse career (required)
    let career = parse_career(&save_path)?;

    // Parse farms (required)
    let farms = parse_farms(&save_path)?;

    // Parse vehicles (required)
    let vehicles = match parse_vehicles(&save_path) {
        Ok(v) => v,
        Err(e) => {
            warnings.push(
                LocalizedMessage::new("errors.vehicleParseError")
                    .with_param("details", e),
            );
            Vec::new()
        }
    };

    // Parse sales (optional - file may not exist)
    let sales = match parse_sales(&save_path) {
        Ok(s) => s,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "sales.xml"),
            );
            Vec::new()
        }
    };

    // Parse fields (optional)
    let fields = match parse_fields(&save_path) {
        Ok(f) => f,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "fields.xml"),
            );
            Vec::new()
        }
    };

    // Parse farmlands (optional)
    let farmlands = match parse_farmlands(&save_path) {
        Ok(f) => f,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "farmland.xml"),
            );
            Vec::new()
        }
    };

    // Parse placeables (optional)
    let placeables = match parse_placeables(&save_path) {
        Ok(p) => p,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "placeables.xml"),
            );
            Vec::new()
        }
    };

    // Parse missions (optional)
    let missions = match parse_missions(&save_path) {
        Ok(m) => m,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "missions.xml"),
            );
            Vec::new()
        }
    };

    // Parse collectibles (optional)
    let collectibles = match parse_collectibles(&save_path) {
        Ok(c) => c,
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "collectibles.xml"),
            );
            Vec::new()
        }
    };

    // Parse contract settings (optional)
    let contract_settings = match parse_contract_settings(&save_path) {
        Ok(s) => Some(s),
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "r_contracts.xml"),
            );
            None
        }
    };

    // Parse environment (optional)
    let environment = match parse_environment(&save_path) {
        Ok(env) => Some(env),
        Err(_) => {
            warnings.push(
                LocalizedMessage::new("errors.fileUnreadable")
                    .with_param("file", "environment.xml"),
            );
            None
        }
    };

    let mut data = SavegameData {
        path,
        career,
        farms,
        vehicles,
        sales,
        fields,
        farmlands,
        placeables,
        missions,
        collectibles,
        contract_settings,
        environment,
        warnings,
    };

    // Run cross-file validators and append any warnings
    let validation_warnings = validate_savegame(&data);
    data.warnings.extend(validation_warnings);

    Ok(data)
}

#[tauri::command]
pub fn save_changes(path: String, changes: SavegameChanges) -> Result<SaveResult, AppError> {
    let save_path = validate_savegame_path(&path).map_err(|_| AppError::SavegameNotFound {
        path: path.clone(),
    })?;

    if !save_path.exists() {
        return Err(AppError::SavegameNotFound { path });
    }

    let mut files_modified: Vec<String> = Vec::new();
    let mut errors: Vec<LocalizedMessage> = Vec::new();

    // Check if there are any changes to apply
    let has_changes = changes.finance.is_some()
        || changes.vehicles.is_some()
        || changes.sales.is_some()
        || changes.sale_additions.is_some()
        || changes.fields.is_some()
        || changes.farmlands.is_some()
        || changes.placeables.is_some()
        || changes.missions.is_some()
        || changes.collectibles.is_some()
        || changes.contract_settings.is_some()
        || changes.environment.is_some();

    if !has_changes {
        return Ok(SaveResult {
            success: true,
            backup_path: None,
            files_modified,
            errors,
        });
    }

    // Create backup before any write (mandatory)
    let backup_info = backup_manager::create_backup(&save_path)?;

    // Apply finance changes
    if let Some(ref finance) = changes.finance {
        if let Some(money) = finance.money {
            // Write money to careerSavegame.xml
            match writers::career::write_career_money(&save_path, money) {
                Ok(()) => files_modified.push("careerSavegame.xml".to_string()),
                Err(e) => errors.push(
                    LocalizedMessage::new("errors.fileWriteError")
                        .with_param("file", "careerSavegame.xml")
                        .with_param("details", e),
                ),
            }
            // Sync money to farms.xml (farm 1)
            match writers::farm::write_farm_finances(&save_path, 1, Some(money), None) {
                Ok(()) => {
                    if !files_modified.contains(&"farms.xml".to_string()) {
                        files_modified.push("farms.xml".to_string());
                    }
                }
                Err(e) => errors.push(
                    LocalizedMessage::new("errors.fileWriteError")
                        .with_param("file", "farms.xml")
                        .with_param("details", e),
                ),
            }
        }
        if let Some(loan) = finance.loan {
            match writers::farm::write_farm_finances(&save_path, 1, None, Some(loan)) {
                Ok(()) => {
                    if !files_modified.contains(&"farms.xml".to_string()) {
                        files_modified.push("farms.xml".to_string());
                    }
                }
                Err(e) => errors.push(
                    LocalizedMessage::new("errors.fileWriteError")
                        .with_param("file", "farms.xml")
                        .with_param("details", e),
                ),
            }
        }
    }

    // Apply vehicle changes
    if let Some(ref vehicle_changes) = changes.vehicles {
        match writers::vehicle::write_vehicle_changes(&save_path, vehicle_changes) {
            Ok(()) => {
                if !files_modified.contains(&"vehicles.xml".to_string()) {
                    files_modified.push("vehicles.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "vehicles.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply sale changes
    if let Some(ref sale_changes) = changes.sales {
        match writers::sale::write_sale_changes(&save_path, sale_changes) {
            Ok(()) => {
                if !files_modified.contains(&"sales.xml".to_string()) {
                    files_modified.push("sales.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "sales.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply sale additions (new items)
    if let Some(ref sale_additions) = changes.sale_additions {
        match writers::sale::write_sale_additions(&save_path, sale_additions) {
            Ok(()) => {
                if !files_modified.contains(&"sales.xml".to_string()) {
                    files_modified.push("sales.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "sales.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply field changes
    if let Some(ref field_changes) = changes.fields {
        match writers::field::write_field_changes(&save_path, field_changes) {
            Ok(()) => {
                if !files_modified.contains(&"fields.xml".to_string()) {
                    files_modified.push("fields.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "fields.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply farmland changes
    if let Some(ref farmland_changes) = changes.farmlands {
        match writers::field::write_farmland_changes(&save_path, farmland_changes) {
            Ok(()) => {
                if !files_modified.contains(&"farmland.xml".to_string()) {
                    files_modified.push("farmland.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "farmland.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply placeable changes
    if let Some(ref placeable_changes) = changes.placeables {
        match writers::placeable::write_placeable_changes(&save_path, placeable_changes) {
            Ok(()) => {
                if !files_modified.contains(&"placeables.xml".to_string()) {
                    files_modified.push("placeables.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "placeables.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply mission changes
    if let Some(ref mission_changes) = changes.missions {
        match writers::mission::write_mission_changes(&save_path, mission_changes) {
            Ok(()) => {
                if !files_modified.contains(&"missions.xml".to_string()) {
                    files_modified.push("missions.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "missions.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply collectible changes
    if let Some(ref collectible_changes) = changes.collectibles {
        match writers::collectible::write_collectible_changes(&save_path, collectible_changes) {
            Ok(()) => {
                if !files_modified.contains(&"collectibles.xml".to_string()) {
                    files_modified.push("collectibles.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "collectibles.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply contract settings changes
    if let Some(ref contract_changes) = changes.contract_settings {
        match writers::contract::write_contract_settings(&save_path, contract_changes) {
            Ok(()) => {
                if !files_modified.contains(&"r_contracts.xml".to_string()) {
                    files_modified.push("r_contracts.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "r_contracts.xml")
                    .with_param("details", e),
            ),
        }
    }

    // Apply environment changes
    if let Some(ref env_changes) = changes.environment {
        match writers::environment::write_environment_changes(&save_path, env_changes) {
            Ok(()) => {
                if !files_modified.contains(&"environment.xml".to_string()) {
                    files_modified.push("environment.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "environment.xml")
                    .with_param("details", e),
            ),
        }
    }

    Ok(SaveResult {
        success: errors.is_empty(),
        backup_path: Some(backup_info.path),
        files_modified,
        errors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_path() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test_saves")
            .display()
            .to_string()
    }

    fn complete_fixture_path() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete")
            .display()
            .to_string()
    }

    #[test]
    fn test_list_savegames_with_fixtures() {
        let result = list_savegames(Some(fixtures_path())).unwrap();
        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Ma partie"));
        assert!(names.contains(&"Partie 2"));
    }

    #[test]
    fn test_list_savegames_empty_dir() {
        let dir = std::env::temp_dir().join("fs25_test_empty_saves");
        let _ = std::fs::create_dir_all(&dir);

        let result = list_savegames(Some(dir.display().to_string())).unwrap();
        assert!(result.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_savegames_nonexistent_path() {
        let result = list_savegames(Some("/nonexistent/path/nowhere".to_string())).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_savegame_complete() {
        let data = load_savegame(complete_fixture_path()).unwrap();
        assert_eq!(data.career.savegame_name, "Test Complete");
        assert_eq!(data.career.map_title, "Riverbend Springs");
        assert!((data.career.money - 1_000_000.0).abs() < 0.01);
        assert_eq!(data.career.growth_mode, 1);
        assert_eq!(data.career.planned_days_per_period, 3);
        assert!(data.career.plowing_required);
        assert!(data.career.weeds_enabled);
        assert_eq!(data.career.fuel_usage, 2);

        assert_eq!(data.farms.len(), 1);
        assert_eq!(data.vehicles.len(), 3);
        assert_eq!(data.sales.len(), 2);
        assert_eq!(data.fields.len(), 4);
        assert_eq!(data.farmlands.len(), 5);
        assert_eq!(data.placeables.len(), 4);
        assert_eq!(data.missions.len(), 3);
        assert_eq!(data.collectibles.len(), 25);
        assert!(data.contract_settings.is_some());
        assert!(data.environment.is_some());
        let env = data.environment.unwrap();
        assert_eq!(env.current_day, 54);
        assert_eq!(env.weather_forecast.len(), 4);
        assert!(data.warnings.is_empty());
    }

    #[test]
    fn test_load_savegame_missing_sales() {
        // Use savegame1 fixture which has career but no sales
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test_saves")
            .join("savegame1");

        // Add a minimal farms.xml and vehicles.xml for this test
        let farms_content = r#"<?xml version="1.0" encoding="utf-8"?>
<farms>
  <farm farmId="1" name="Test" color="1" loan="0" money="100">
    <players></players>
    <finances></finances>
  </farm>
</farms>"#;
        let vehicles_content = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicles></vehicles>"#;

        std::fs::write(path.join("farms.xml"), farms_content).unwrap();
        std::fs::write(path.join("vehicles.xml"), vehicles_content).unwrap();

        let data = load_savegame(path.display().to_string()).unwrap();
        assert_eq!(data.career.savegame_name, "Ma partie");
        assert!(data.sales.is_empty());
        assert!(data.warnings.iter().any(|w| w.code == "errors.fileUnreadable" && w.params.get("file").map(|f| f.as_str()) == Some("sales.xml")));

        // Cleanup
        let _ = std::fs::remove_file(path.join("farms.xml"));
        let _ = std::fs::remove_file(path.join("vehicles.xml"));
    }

    #[test]
    fn test_load_savegame_invalid_path() {
        let result = load_savegame("/nonexistent/path".to_string());
        assert!(matches!(result, Err(AppError::SavegameNotFound { .. })));
    }

    fn setup_writable_fixture(name: &str) -> String {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_save_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        let mut opts = fs_extra::dir::CopyOptions::new();
        opts.copy_inside = true;
        fs_extra::dir::copy(&src, &dst, &opts).unwrap();
        dst.display().to_string()
    }

    fn cleanup_writable_fixture(path: &str) {
        let p = PathBuf::from(path);
        let _ = std::fs::remove_dir_all(&p);
        let backups = p.parent().unwrap().join(format!(
            "{}_backups",
            p.file_name().unwrap().to_string_lossy()
        ));
        let _ = std::fs::remove_dir_all(backups);
    }

    #[test]
    fn test_save_changes_creates_backup() {
        let path = setup_writable_fixture("backup_check");
        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(999.0),
                loan: None,
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        let result = save_changes(path.clone(), changes).unwrap();
        assert!(result.success);
        assert!(result.backup_path.is_some());
        assert!(PathBuf::from(result.backup_path.unwrap()).exists());
        cleanup_writable_fixture(&path);
    }

    #[test]
    fn test_save_changes_money_sync() {
        let path = setup_writable_fixture("money_sync");
        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(555555.0),
                loan: None,
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        save_changes(path.clone(), changes).unwrap();

        // Verify both files
        let data = load_savegame(path.clone()).unwrap();
        assert!((data.career.money - 555555.0).abs() < 0.01);
        assert!((data.farms[0].money - 555555.0).abs() < 0.01);
        cleanup_writable_fixture(&path);
    }

    #[test]
    fn test_save_changes_empty_changes() {
        let path = setup_writable_fixture("empty_changes");
        let changes = SavegameChanges {
            finance: None,
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        let result = save_changes(path.clone(), changes).unwrap();
        assert!(result.success);
        assert!(result.backup_path.is_none());
        assert!(result.files_modified.is_empty());
        cleanup_writable_fixture(&path);
    }

    #[test]
    fn test_full_save_cycle() {
        let path = setup_writable_fixture("full_cycle");

        // Load original
        let data = load_savegame(path.clone()).unwrap();
        assert!((data.career.money - 1_000_000.0).abs() < 0.01);
        assert_eq!(data.vehicles.len(), 3);

        // Modify finances
        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(777777.0),
                loan: Some(25000.0),
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        let result = save_changes(path.clone(), changes).unwrap();
        assert!(result.success);

        // Reload and verify changes persisted
        let data2 = load_savegame(path.clone()).unwrap();
        assert!((data2.career.money - 777777.0).abs() < 0.01);
        assert!((data2.farms[0].money - 777777.0).abs() < 0.01);
        assert!((data2.farms[0].loan - 25000.0).abs() < 0.01);

        // Vehicles should still be intact
        assert_eq!(data2.vehicles.len(), 3);

        cleanup_writable_fixture(&path);
    }

    #[test]
    fn test_save_changes_atomic_write() {
        let path = setup_writable_fixture("atomic_write");
        let save_path = PathBuf::from(&path);

        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(123456.0),
                loan: None,
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        save_changes(path.clone(), changes).unwrap();

        // No .xml.tmp files should remain after save
        let tmp_files: Vec<_> = std::fs::read_dir(&save_path)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .ends_with(".xml.tmp")
            })
            .collect();
        assert!(
            tmp_files.is_empty(),
            "Found leftover .tmp files: {:?}",
            tmp_files.iter().map(|f| f.file_name()).collect::<Vec<_>>()
        );

        cleanup_writable_fixture(&path);
    }

    #[test]
    fn test_write_preserves_xml_header() {
        let path = setup_writable_fixture("xml_header");
        let save_path = PathBuf::from(&path);

        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(42.0),
                loan: None,
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        save_changes(path.clone(), changes).unwrap();

        // Check XML header is preserved in careerSavegame.xml
        let content = std::fs::read_to_string(save_path.join("careerSavegame.xml")).unwrap();
        assert!(
            content.starts_with("<?xml"),
            "XML header not preserved. File starts with: {}",
            &content[..content.len().min(50)]
        );

        // Check farms.xml too
        let farms_content = std::fs::read_to_string(save_path.join("farms.xml")).unwrap();
        assert!(
            farms_content.starts_with("<?xml"),
            "farms.xml XML header not preserved"
        );

        cleanup_writable_fixture(&path);
    }

    fn modded_fixture_path() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_modded")
            .display()
            .to_string()
    }

    #[test]
    fn test_parse_modded_vehicles() {
        // Parser should not crash on unknown elements/attributes from mods
        let data = load_savegame(modded_fixture_path()).unwrap();
        assert_eq!(data.vehicles.len(), 2);
        assert_eq!(data.career.savegame_name, "Modded Save");

        // Verify known fields are still parsed correctly
        let tractor = &data.vehicles[0];
        assert!((tractor.price - 348000.0).abs() < 0.01);
        assert_eq!(tractor.farm_id, 1);

        // Mod vehicle from mods/ folder
        let trailer = &data.vehicles[1];
        assert!(trailer.filename.contains("mods/"));
        assert!((trailer.price - 75000.0).abs() < 0.01);
    }

    #[test]
    fn test_write_modded_preserves_unknown() {
        // Copy modded fixture to writable location
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_modded");
        let dst = std::env::temp_dir().join("fs25_test_save_modded_write");
        let _ = std::fs::remove_dir_all(&dst);
        let mut opts = fs_extra::dir::CopyOptions::new();
        opts.copy_inside = true;
        fs_extra::dir::copy(&src, &dst, &opts).unwrap();
        let path = dst.display().to_string();

        // Read original vehicles.xml content for reference
        let original_content = std::fs::read_to_string(dst.join("vehicles.xml")).unwrap();
        assert!(original_content.contains("modGpsData"));
        assert!(original_content.contains("modAutoSteer"));
        assert!(original_content.contains("modSpecialFeature"));

        // Save finance changes (should not touch vehicles.xml content if no vehicle changes)
        let changes = SavegameChanges {
            finance: Some(crate::models::changes::FinanceChanges {
                money: Some(999999.0),
                loan: None,
            }),
            vehicles: None,
            sales: None,
            sale_additions: None,
            fields: None,
            farmlands: None,
            placeables: None,
            missions: None,
            collectibles: None,
            contract_settings: None,
            environment: None,
        };
        save_changes(path.clone(), changes).unwrap();

        // vehicles.xml should be untouched (no vehicle changes)
        let after_content = std::fs::read_to_string(dst.join("vehicles.xml")).unwrap();
        assert!(after_content.contains("modGpsData"), "modGpsData lost after save");
        assert!(after_content.contains("modAutoSteer"), "modAutoSteer lost after save");
        assert!(after_content.contains("modSpecialFeature"), "modSpecialFeature lost after save");

        // Verify finance change was applied
        let data = load_savegame(path.clone()).unwrap();
        assert!((data.career.money - 999999.0).abs() < 0.01);

        cleanup_writable_fixture(&path);
    }
}
