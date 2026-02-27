use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{VehicleChange, FillUnitChange};

/// Applies a list of vehicle changes to vehicles.xml.
/// Patch strategy: reads original XML, modifies only targeted attributes, rewrites atomically.
pub fn write_vehicle_changes(
    path: &Path,
    changes: &[VehicleChange],
) -> Result<(), AppError> {
    let xml_path = path.join("vehicles.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    // Build a lookup map for quick access
    let change_map: std::collections::HashMap<&str, &VehicleChange> = changes
        .iter()
        .map(|c| (c.unique_id.as_str(), c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    // State tracking
    let mut current_vehicle_id: Option<String> = None;
    let mut skip_until_vehicle_end: bool = false;
    let mut skip_depth: u32 = 0;
    let mut in_fill_unit = false;
    let mut current_fill_changes: Option<&Vec<FillUnitChange>> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if skip_until_vehicle_end {
                    skip_depth += 1;
                    continue;
                }

                match tag.as_str() {
                    "vehicle" => {
                        let id = attr_str(e, "id");
                        if let Some(change) = change_map.get(id.as_str()) {
                            if change.delete {
                                skip_until_vehicle_end = true;
                                skip_depth = 1;
                                current_vehicle_id = Some(id);
                                continue;
                            }
                            // Modify vehicle attributes
                            let elem = patch_vehicle_start(e, change);
                            current_vehicle_id = Some(id);
                            current_fill_changes = change.fill_units.as_ref();
                            write_event(&mut writer, &xml_path, Event::Start(elem))?;
                        } else {
                            current_vehicle_id = None;
                            current_fill_changes = None;
                            write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        }
                    }
                    "fillUnit" if current_vehicle_id.is_some() => {
                        in_fill_unit = true;
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                    _ => {
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if skip_until_vehicle_end {
                    continue;
                }

                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if tag == "unit" && in_fill_unit {
                    if let Some(fill_changes) = current_fill_changes {
                        let unit_index: u32 = attr_str(e, "index").parse().unwrap_or(u32::MAX);
                        if let Some(fc) = fill_changes.iter().find(|f| f.index == unit_index) {
                            let elem = patch_fill_unit(e, fc);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                            continue;
                        }
                    }
                }

                write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if skip_until_vehicle_end {
                    if tag == "vehicle" {
                        skip_depth -= 1;
                        if skip_depth == 0 {
                            skip_until_vehicle_end = false;
                            current_vehicle_id = None;
                        }
                    } else {
                        skip_depth -= 1;
                    }
                    continue;
                }

                match tag.as_str() {
                    "vehicle" => {
                        current_vehicle_id = None;
                        current_fill_changes = None;
                        write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                    "fillUnit" => {
                        in_fill_unit = false;
                        write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                    _ => {
                        write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if !skip_until_vehicle_end {
                    write_event(&mut writer, &xml_path, event.into_owned())?;
                }
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

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn property_state_to_u8(state: &str) -> &str {
    match state {
        "Owned" => "1",
        "Rented" => "2",
        "None" => "0",
        _ => "0",
    }
}

fn patch_vehicle_start(e: &BytesStart, change: &VehicleChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("vehicle");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "age" if change.age.is_some() => {
                elem.push_attribute(("age", format!("{:.6}", change.age.unwrap()).as_str()));
            }
            "price" if change.price.is_some() => {
                elem.push_attribute(("price", format!("{:.6}", change.price.unwrap()).as_str()));
            }
            "farmId" if change.farm_id.is_some() => {
                elem.push_attribute(("farmId", change.farm_id.unwrap().to_string().as_str()));
            }
            "propertyState" if change.property_state.is_some() => {
                elem.push_attribute((
                    "propertyState",
                    property_state_to_u8(change.property_state.as_ref().unwrap()),
                ));
            }
            "operatingTime" if change.operating_time.is_some() => {
                elem.push_attribute((
                    "operatingTime",
                    format!("{:.6}", change.operating_time.unwrap() * 60.0).as_str(),
                ));
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

fn patch_fill_unit(e: &BytesStart, change: &FillUnitChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("unit");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "fillLevel" => {
                elem.push_attribute((
                    "fillLevel",
                    format!("{:.6}", change.fill_level).as_str(),
                ));
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
    use crate::parsers::vehicle::parse_vehicles;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wv_{}", name));
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
    fn test_write_vehicle_price() {
        let save = setup_fixture("price");
        let changes = vec![VehicleChange {
            unique_id: "1".to_string(),
            delete: false,
            age: None,
            price: Some(999999.0),
            farm_id: None,
            property_state: None,
            operating_time: None,
            fill_units: None,
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let vehicles = parse_vehicles(&save).unwrap();
        let v = vehicles.iter().find(|v| v.unique_id == "1").unwrap();
        assert!((v.price - 999999.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_age() {
        let save = setup_fixture("age");
        let changes = vec![VehicleChange {
            unique_id: "1".to_string(),
            delete: false,
            age: Some(0.0),
            price: None,
            farm_id: None,
            property_state: None,
            operating_time: Some(0.0), // 0 hours
            fill_units: None,
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let vehicles = parse_vehicles(&save).unwrap();
        let v = vehicles.iter().find(|v| v.unique_id == "1").unwrap();
        assert!((v.age - 0.0).abs() < 0.01);
        assert!((v.operating_time - 0.0).abs() < 0.01); // 0 hours
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_fill_level() {
        let save = setup_fixture("fill");
        let changes = vec![VehicleChange {
            unique_id: "1".to_string(),
            delete: false,
            age: None,
            price: None,
            farm_id: None,
            property_state: None,
            operating_time: None,
            fill_units: Some(vec![FillUnitChange {
                index: 0,
                fill_level: 500.0,
            }]),
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let vehicles = parse_vehicles(&save).unwrap();
        let v = vehicles.iter().find(|v| v.unique_id == "1").unwrap();
        let unit = v.fill_units.iter().find(|u| u.index == 0).unwrap();
        assert!((unit.fill_level - 500.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_delete() {
        let save = setup_fixture("delete");
        let changes = vec![VehicleChange {
            unique_id: "2".to_string(),
            delete: true,
            age: None,
            price: None,
            farm_id: None,
            property_state: None,
            operating_time: None,
            fill_units: None,
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let vehicles = parse_vehicles(&save).unwrap();
        assert_eq!(vehicles.len(), 2);
        assert!(vehicles.iter().all(|v| v.unique_id != "2"));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_preserves_unknown() {
        let save = setup_fixture("preserve");
        let before = parse_vehicles(&save).unwrap();
        let changes = vec![VehicleChange {
            unique_id: "1".to_string(),
            delete: false,
            age: None,
            price: Some(1.0),
            farm_id: None,
            property_state: None,
            operating_time: None,
            fill_units: None,
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let after = parse_vehicles(&save).unwrap();

        // Vehicle 2 and 3 should be untouched
        let before_v2 = before.iter().find(|v| v.unique_id == "2").unwrap();
        let after_v2 = after.iter().find(|v| v.unique_id == "2").unwrap();
        assert!((before_v2.price - after_v2.price).abs() < 0.01);
        assert_eq!(before_v2.fill_units.len(), after_v2.fill_units.len());

        // Vehicle 1 should have modified price but preserved other fields
        let after_v1 = after.iter().find(|v| v.unique_id == "1").unwrap();
        assert!((after_v1.price - 1.0).abs() < 0.01);
        assert_eq!(after_v1.configurations.len(), 2);
        assert_eq!(after_v1.fill_units.len(), 2);

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_roundtrip() {
        let save = setup_fixture("roundtrip");
        let before = parse_vehicles(&save).unwrap();

        // Modify then re-read
        let changes = vec![VehicleChange {
            unique_id: "1".to_string(),
            delete: false,
            age: Some(99.0),
            price: Some(123456.0),
            farm_id: None,
            property_state: None,
            operating_time: Some(999.0), // 999 hours, writer converts to 59940 minutes in XML
            fill_units: Some(vec![
                FillUnitChange { index: 0, fill_level: 111.0 },
                FillUnitChange { index: 1, fill_level: 22.0 },
            ]),
        }];
        write_vehicle_changes(&save, &changes).unwrap();
        let after = parse_vehicles(&save).unwrap();

        let v = after.iter().find(|v| v.unique_id == "1").unwrap();
        assert!((v.age - 99.0).abs() < 0.01);
        assert!((v.price - 123456.0).abs() < 0.01);
        assert!((v.operating_time - 999.0).abs() < 0.01);
        assert!((v.fill_units[0].fill_level - 111.0).abs() < 0.01);
        assert!((v.fill_units[1].fill_level - 22.0).abs() < 0.01);

        // Untouched vehicles remain the same
        let before_v3 = before.iter().find(|v| v.unique_id == "3").unwrap();
        let after_v3 = after.iter().find(|v| v.unique_id == "3").unwrap();
        assert!((before_v3.price - after_v3.price).abs() < 0.01);

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_vehicle_multiple_changes() {
        let save = setup_fixture("multi");
        let changes = vec![
            VehicleChange {
                unique_id: "1".to_string(),
                delete: false,
                age: None,
                price: Some(100.0),
                farm_id: None,
                property_state: None,
                operating_time: None,
                fill_units: None,
            },
            VehicleChange {
                unique_id: "3".to_string(),
                delete: false,
                age: Some(0.0),
                price: None,
                farm_id: None,
                property_state: None,
                operating_time: None,
                fill_units: None,
            },
        ];
        write_vehicle_changes(&save, &changes).unwrap();
        let vehicles = parse_vehicles(&save).unwrap();

        let v1 = vehicles.iter().find(|v| v.unique_id == "1").unwrap();
        assert!((v1.price - 100.0).abs() < 0.01);

        let v3 = vehicles.iter().find(|v| v.unique_id == "3").unwrap();
        assert!((v3.age - 0.0).abs() < 0.01);

        let _ = std::fs::remove_dir_all(&save);
    }
}
