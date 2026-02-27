use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{FarmlandChange, FieldChange};

/// Applies field changes to fields.xml.
/// Fields are identified by their `id` attribute.
pub fn write_field_changes(path: &Path, changes: &[FieldChange]) -> Result<(), AppError> {
    let xml_path = path.join("fields.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<u32, &FieldChange> =
        changes.iter().map(|c| (c.id, c)).collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "field" {
                    let id = attr_u32(e, "id");
                    if let Some(change) = change_map.get(&id) {
                        let elem = patch_field(e, change);
                        write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                    } else {
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Empty(e.clone().into_owned()),
                        )?;
                    }
                } else {
                    write_event(
                        &mut writer,
                        &xml_path,
                        Event::Empty(e.clone().into_owned()),
                    )?;
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

/// Applies farmland changes to farmland.xml.
pub fn write_farmland_changes(path: &Path, changes: &[FarmlandChange]) -> Result<(), AppError> {
    let xml_path = path.join("farmland.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<u32, &FarmlandChange> =
        changes.iter().map(|c| (c.id, c)).collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "farmland" {
                    let id = attr_u32(e, "id");
                    if let Some(change) = change_map.get(&id) {
                        let elem = patch_farmland(e, change);
                        write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                    } else {
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Empty(e.clone().into_owned()),
                        )?;
                    }
                } else {
                    write_event(
                        &mut writer,
                        &xml_path,
                        Event::Empty(e.clone().into_owned()),
                    )?;
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

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_u32(e: &BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn patch_field(e: &BytesStart, change: &FieldChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("field");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "fruitType" if change.fruit_type.is_some() => {
                elem.push_attribute(("fruitType", change.fruit_type.as_ref().unwrap().as_str()));
            }
            "plannedFruit" if change.planned_fruit.is_some() => {
                elem.push_attribute((
                    "plannedFruit",
                    change.planned_fruit.as_ref().unwrap().as_str(),
                ));
            }
            "growthState" if change.growth_state.is_some() => {
                elem.push_attribute((
                    "growthState",
                    change.growth_state.unwrap().to_string().as_str(),
                ));
            }
            "groundType" if change.ground_type.is_some() => {
                elem.push_attribute((
                    "groundType",
                    change.ground_type.as_ref().unwrap().as_str(),
                ));
            }
            "weedState" if change.weed_state.is_some() => {
                elem.push_attribute((
                    "weedState",
                    change.weed_state.unwrap().to_string().as_str(),
                ));
            }
            "stoneLevel" if change.stone_level.is_some() => {
                elem.push_attribute((
                    "stoneLevel",
                    change.stone_level.unwrap().to_string().as_str(),
                ));
            }
            "sprayLevel" if change.spray_level.is_some() => {
                elem.push_attribute((
                    "sprayLevel",
                    change.spray_level.unwrap().to_string().as_str(),
                ));
            }
            "sprayType" if change.spray_type.is_some() => {
                elem.push_attribute(("sprayType", change.spray_type.as_ref().unwrap().as_str()));
            }
            "limeLevel" if change.lime_level.is_some() => {
                elem.push_attribute((
                    "limeLevel",
                    change.lime_level.unwrap().to_string().as_str(),
                ));
            }
            "plowLevel" if change.plow_level.is_some() => {
                elem.push_attribute((
                    "plowLevel",
                    change.plow_level.unwrap().to_string().as_str(),
                ));
            }
            "rollerLevel" if change.roller_level.is_some() => {
                elem.push_attribute((
                    "rollerLevel",
                    change.roller_level.unwrap().to_string().as_str(),
                ));
            }
            "stubbleShredLevel" if change.stubble_shred_level.is_some() => {
                elem.push_attribute((
                    "stubbleShredLevel",
                    change.stubble_shred_level.unwrap().to_string().as_str(),
                ));
            }
            "waterLevel" if change.water_level.is_some() => {
                elem.push_attribute((
                    "waterLevel",
                    change.water_level.unwrap().to_string().as_str(),
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

fn patch_farmland(e: &BytesStart, change: &FarmlandChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("farmland");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "farmId" => {
                elem.push_attribute(("farmId", change.farm_id.to_string().as_str()));
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
    writer
        .write_event(event)
        .map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::field::{parse_farmlands, parse_fields};

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wf_{}", name));
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
    fn test_write_field_growth_state() {
        let save = setup_fixture("field_growth");
        let changes = vec![FieldChange {
            id: 1,
            fruit_type: None,
            planned_fruit: None,
            growth_state: Some(5),
            ground_type: None,
            weed_state: None,
            stone_level: None,
            spray_level: None,
            spray_type: None,
            lime_level: None,
            plow_level: None,
            roller_level: None,
            stubble_shred_level: None,
            water_level: None,
        }];
        write_field_changes(&save, &changes).unwrap();
        let fields = parse_fields(&save).unwrap();
        let f = fields.iter().find(|f| f.id == 1).unwrap();
        assert_eq!(f.growth_state, 5);
        // Other fields preserved
        assert_eq!(f.fruit_type, "WHEAT");
        assert_eq!(f.lime_level, 3);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_field_fruit_type() {
        let save = setup_fixture("field_fruit");
        let changes = vec![FieldChange {
            id: 2,
            fruit_type: Some("CORN".to_string()),
            planned_fruit: Some("CORN".to_string()),
            growth_state: None,
            ground_type: None,
            weed_state: None,
            stone_level: None,
            spray_level: None,
            spray_type: None,
            lime_level: None,
            plow_level: None,
            roller_level: None,
            stubble_shred_level: None,
            water_level: None,
        }];
        write_field_changes(&save, &changes).unwrap();
        let fields = parse_fields(&save).unwrap();
        let f = fields.iter().find(|f| f.id == 2).unwrap();
        assert_eq!(f.fruit_type, "CORN");
        assert_eq!(f.planned_fruit, "CORN");
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_farmland_owner() {
        let save = setup_fixture("farmland_owner");
        let changes = vec![FarmlandChange { id: 3, farm_id: 1 }];
        write_farmland_changes(&save, &changes).unwrap();
        let farmlands = parse_farmlands(&save).unwrap();
        let fl = farmlands.iter().find(|f| f.id == 3).unwrap();
        assert_eq!(fl.farm_id, 1);
        // Others unchanged
        let fl5 = farmlands.iter().find(|f| f.id == 5).unwrap();
        assert_eq!(fl5.farm_id, 0);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_field_roundtrip() {
        let save = setup_fixture("field_roundtrip");
        let changes = vec![FieldChange {
            id: 3,
            fruit_type: Some("WHEAT".to_string()),
            planned_fruit: None,
            growth_state: Some(10),
            ground_type: Some("HARVEST_READY".to_string()),
            weed_state: Some(0),
            stone_level: Some(0),
            spray_level: Some(2),
            spray_type: None,
            lime_level: Some(3),
            plow_level: None,
            roller_level: None,
            stubble_shred_level: None,
            water_level: None,
        }];
        write_field_changes(&save, &changes).unwrap();
        let fields = parse_fields(&save).unwrap();
        let f = fields.iter().find(|f| f.id == 3).unwrap();
        assert_eq!(f.fruit_type, "WHEAT");
        assert_eq!(f.growth_state, 10);
        assert_eq!(f.ground_type, "HARVEST_READY");
        assert_eq!(f.weed_state, 0);
        assert_eq!(f.stone_level, 0);
        assert_eq!(f.spray_level, 2);
        assert_eq!(f.lime_level, 3);

        // Field 1 should be untouched
        let f1 = fields.iter().find(|f| f.id == 1).unwrap();
        assert_eq!(f1.growth_state, 10);
        assert_eq!(f1.fruit_type, "WHEAT");

        let _ = std::fs::remove_dir_all(&save);
    }
}
