use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::field::{Farmland, Field};

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_u32(e: &quick_xml::events::BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn attr_u8(e: &quick_xml::events::BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

/// Parse fields.xml and return the list of fields.
pub fn parse_fields(path: &Path) -> Result<Vec<Field>, AppError> {
    let xml_path = path.join("fields.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut fields: Vec<Field> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "field" {
                    fields.push(Field {
                        id: attr_u32(e, "id"),
                        planned_fruit: attr_str(e, "plannedFruit"),
                        fruit_type: attr_str(e, "fruitType"),
                        growth_state: attr_u8(e, "growthState"),
                        last_growth_state: attr_u8(e, "lastGrowthState"),
                        weed_state: attr_u8(e, "weedState"),
                        stone_level: attr_u8(e, "stoneLevel"),
                        spray_level: attr_u8(e, "sprayLevel"),
                        spray_type: attr_str(e, "sprayType"),
                        lime_level: attr_u8(e, "limeLevel"),
                        plow_level: attr_u8(e, "plowLevel"),
                        roller_level: attr_u8(e, "rollerLevel"),
                        stubble_shred_level: attr_u8(e, "stubbleShredLevel"),
                        water_level: attr_u8(e, "waterLevel"),
                        ground_type: attr_str(e, "groundType"),
                    });
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

    Ok(fields)
}

/// Parse farmland.xml and return the list of farmlands.
pub fn parse_farmlands(path: &Path) -> Result<Vec<Farmland>, AppError> {
    let xml_path = path.join("farmland.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut farmlands: Vec<Farmland> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "farmland" {
                    farmlands.push(Farmland {
                        id: attr_u32(e, "id"),
                        farm_id: attr_u8(e, "farmId"),
                    });
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

    Ok(farmlands)
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
    fn test_parse_fields_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let fields = parse_fields(&path).unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].id, 1);
        assert_eq!(fields[0].fruit_type, "WHEAT");
        assert_eq!(fields[0].growth_state, 10);
        assert_eq!(fields[0].ground_type, "HARVEST_READY");
        assert_eq!(fields[0].lime_level, 3);
        assert_eq!(fields[2].weed_state, 5);
        assert_eq!(fields[2].stone_level, 2);
    }

    #[test]
    fn test_parse_farmlands_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let farmlands = parse_farmlands(&path).unwrap();
        assert_eq!(farmlands.len(), 5);
        assert_eq!(farmlands[0].id, 1);
        assert_eq!(farmlands[0].farm_id, 1);
        assert_eq!(farmlands[2].id, 3);
        assert_eq!(farmlands[2].farm_id, 0);
    }

    #[test]
    fn test_parse_fields_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_fields");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_fields(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_farmlands_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_farmlands");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_farmlands(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
