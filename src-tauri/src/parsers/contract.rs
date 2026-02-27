use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::contract::ContractSettings;

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_f64(e: &quick_xml::events::BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

pub fn parse_contract_settings(path: &Path) -> Result<ContractSettings, AppError> {
    let xml_path = path.join("r_contracts.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut settings = ContractSettings {
        lease_vehicle: 1.0,
        mission_per_farm: 1.0,
        allow_clear_add: 1.0,
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "settings" || tag == "contractSettings" {
                    settings.lease_vehicle = attr_f64(e, "leaseVehicle");
                    settings.mission_per_farm = attr_f64(e, "missionPerFarm");
                    settings.allow_clear_add = attr_f64(e, "allowClearAdd");
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
            _ => {}
        }
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    #[test]
    fn test_parse_contract_settings() {
        let path = fixtures_path().join("savegame_complete");
        let settings = parse_contract_settings(&path).unwrap();
        assert!((settings.lease_vehicle - 3.0).abs() < 0.01);
        assert!((settings.mission_per_farm - 2.0).abs() < 0.01);
        assert!((settings.allow_clear_add - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_contract_settings_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_contracts");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_contract_settings(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
