use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::contract::ContractSettings;

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
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

    // Real saves store each setting as a child element holding a number, e.g.
    // <r_contracts><leaseVehicle>2.000000</leaseVehicle>. Older mod versions used
    // attributes on a <settings> element, which is still accepted.
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = e.name().as_ref().to_string();
                if tag == "settings" || tag == "contractSettings" {
                    settings.lease_vehicle = attr_f64(e, "leaseVehicle");
                    settings.mission_per_farm = attr_f64(e, "missionPerFarm");
                    settings.allow_clear_add = attr_f64(e, "allowClearAdd");
                }
                current_tag = tag;
            }
            Ok(Event::Text(ref t)) => {
                let text = t.trim().to_string();
                if let Ok(value) = text.parse::<f64>() {
                    match current_tag.as_str() {
                        "leaseVehicle" => settings.lease_vehicle = value,
                        "missionPerFarm" => settings.mission_per_farm = value,
                        "allowClearAdd" => settings.allow_clear_add = value,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => current_tag.clear(),
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
    fn test_parse_contract_settings_legacy_attribute_form() {
        // Older versions of the contracts mod stored settings as attributes.
        let dir = std::env::temp_dir().join("fs25_test_contracts_legacy");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("r_contracts.xml"),
            r#"<?xml version="1.0" encoding="utf-8"?>
<contracts>
  <settings leaseVehicle="3.000000" missionPerFarm="2.000000" allowClearAdd="1.000000" />
</contracts>"#,
        )
        .unwrap();

        let settings = parse_contract_settings(&dir).unwrap();
        assert!((settings.lease_vehicle - 3.0).abs() < 0.01);
        assert!((settings.mission_per_farm - 2.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&dir);
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
