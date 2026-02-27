use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::mission::{Mission, MissionStatus};

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

fn attr_u32_opt(e: &quick_xml::events::BytesStart, key: &str) -> Option<u32> {
    let s = attr_str(e, key);
    if s.is_empty() { None } else { s.parse().ok() }
}

fn attr_f64_opt(e: &quick_xml::events::BytesStart, key: &str) -> Option<f64> {
    let s = attr_str(e, key);
    if s.is_empty() { None } else { s.parse().ok() }
}

fn attr_str_opt(e: &quick_xml::events::BytesStart, key: &str) -> Option<String> {
    let s = attr_str(e, key);
    if s.is_empty() { None } else { Some(s) }
}

pub fn parse_missions(path: &Path) -> Result<Vec<Mission>, AppError> {
    let xml_path = path.join("missions.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut missions: Vec<Mission> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "mission" {
                    let unique_id = attr_str(e, "id");
                    if unique_id.is_empty() {
                        continue;
                    }
                    missions.push(Mission {
                        unique_id,
                        mission_type: attr_str(e, "type"),
                        status: MissionStatus::from_str(&attr_str(e, "status")),
                        reward: attr_f64(e, "reward"),
                        reimbursement: attr_f64(e, "reimbursement"),
                        completion: attr_f64(e, "completion"),
                        field_id: attr_u32_opt(e, "fieldId"),
                        fruit_type: attr_str_opt(e, "fruitType"),
                        expected_liters: attr_f64_opt(e, "expectedLiters"),
                        deposited_liters: attr_f64_opt(e, "depositedLiters"),
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

    Ok(missions)
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
    fn test_parse_missions_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let missions = parse_missions(&path).unwrap();
        assert_eq!(missions.len(), 3);

        let harvest = missions.iter().find(|m| m.unique_id == "1").unwrap();
        assert_eq!(harvest.mission_type, "harvest");
        assert_eq!(harvest.status, MissionStatus::Running);
        assert!((harvest.reward - 12000.0).abs() < 0.01);
        assert!((harvest.completion - 0.45).abs() < 0.01);
        assert_eq!(harvest.field_id, Some(12));
        assert_eq!(harvest.fruit_type.as_deref(), Some("WHEAT"));
    }

    #[test]
    fn test_parse_missions_statuses() {
        let path = fixtures_path().join("savegame_complete");
        let missions = parse_missions(&path).unwrap();

        let created = missions.iter().find(|m| m.status == MissionStatus::Created);
        assert!(created.is_some());

        let completed = missions.iter().find(|m| m.status == MissionStatus::Completed);
        assert!(completed.is_some());
    }

    #[test]
    fn test_parse_missions_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_missions");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_missions(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
