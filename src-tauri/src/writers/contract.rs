use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::ContractSettingsChange;

pub fn write_contract_settings(
    path: &Path,
    changes: &ContractSettingsChange,
) -> Result<(), AppError> {
    let xml_path = path.join("r_contracts.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "settings" || tag == "contractSettings" {
                    let elem = patch_settings(e, &tag, changes);
                    write_event(&mut writer, &xml_path, Event::Start(elem))?;
                } else {
                    write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "settings" || tag == "contractSettings" {
                    let elem = patch_settings(e, &tag, changes);
                    write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                } else {
                    write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                write_event(&mut writer, &xml_path, event.into_owned())?;
            }
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

fn patch_settings(e: &BytesStart, tag_name: &str, changes: &ContractSettingsChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new(tag_name.to_string());
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "leaseVehicle" if changes.lease_vehicle.is_some() => {
                elem.push_attribute(("leaseVehicle", format!("{:.6}", changes.lease_vehicle.unwrap()).as_str()));
            }
            "missionPerFarm" if changes.mission_per_farm.is_some() => {
                elem.push_attribute(("missionPerFarm", format!("{:.6}", changes.mission_per_farm.unwrap()).as_str()));
            }
            "allowClearAdd" if changes.allow_clear_add.is_some() => {
                elem.push_attribute(("allowClearAdd", format!("{:.6}", changes.allow_clear_add.unwrap()).as_str()));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    String::from_utf8_lossy(&attr.value).as_ref(),
                ));
            }
        }
    }
    elem
}

fn write_event(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer.write_event(event).map_err(|e| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::contract::parse_contract_settings;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wct_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
            }
        }
        dst
    }

    #[test]
    fn test_write_contract_settings() {
        let save = setup_fixture("settings");
        let changes = ContractSettingsChange {
            lease_vehicle: Some(5.0),
            mission_per_farm: Some(4.0),
            allow_clear_add: None,
        };
        write_contract_settings(&save, &changes).unwrap();
        let settings = parse_contract_settings(&save).unwrap();
        assert!((settings.lease_vehicle - 5.0).abs() < 0.01);
        assert!((settings.mission_per_farm - 4.0).abs() < 0.01);
        // Unchanged
        assert!((settings.allow_clear_add - 1.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }
}
