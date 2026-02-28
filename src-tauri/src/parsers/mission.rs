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

fn is_mission_tag(tag: &str) -> bool {
    tag.ends_with("Mission") && tag != "missions"
}

fn extract_mission_type(tag: &str) -> String {
    tag.strip_suffix("Mission").unwrap_or(tag).to_string()
}

pub fn parse_missions(path: &Path) -> Result<Vec<Mission>, AppError> {
    let xml_path = path.join("missions.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut missions: Vec<Mission> = Vec::new();

    // State for the mission currently being parsed
    let mut current_mission: Option<Mission> = None;
    let mut current_tag: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if is_mission_tag(&tag) {
                    let unique_id = attr_str(e, "uniqueId");
                    if unique_id.is_empty() {
                        continue;
                    }
                    current_tag = Some(tag.clone());
                    current_mission = Some(Mission {
                        unique_id,
                        mission_type: extract_mission_type(&tag),
                        status: MissionStatus::from_str(&attr_str(e, "status")),
                        reward: 0.0,
                        reimbursement: 0.0,
                        completion: 0.0,
                        field_id: None,
                        fruit_type: None,
                        expected_liters: None,
                        deposited_liters: None,
                    });
                } else if current_mission.is_some() {
                    // Child elements of a mission (non-empty, like <vehicles> with children)
                    match tag.as_str() {
                        "info" => {
                            if let Some(ref mut m) = current_mission {
                                m.reward = attr_f64(e, "reward");
                                m.reimbursement = attr_f64(e, "reimbursement");
                                m.completion = attr_f64(e, "completion");
                            }
                        }
                        "harvest" => {
                            if let Some(ref mut m) = current_mission {
                                m.fruit_type = attr_str_opt(e, "fruitType");
                                m.expected_liters = attr_f64_opt(e, "expectedLiters");
                                m.deposited_liters = attr_f64_opt(e, "depositedLiters");
                            }
                        }
                        "field" => {
                            if let Some(ref mut m) = current_mission {
                                m.field_id = attr_u32_opt(e, "id");
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if current_mission.is_some() {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag.as_str() {
                        "info" => {
                            if let Some(ref mut m) = current_mission {
                                m.reward = attr_f64(e, "reward");
                                m.reimbursement = attr_f64(e, "reimbursement");
                                m.completion = attr_f64(e, "completion");
                            }
                        }
                        "harvest" => {
                            if let Some(ref mut m) = current_mission {
                                m.fruit_type = attr_str_opt(e, "fruitType");
                                m.expected_liters = attr_f64_opt(e, "expectedLiters");
                                m.deposited_liters = attr_f64_opt(e, "depositedLiters");
                            }
                        }
                        "field" => {
                            if let Some(ref mut m) = current_mission {
                                m.field_id = attr_u32_opt(e, "id");
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if Some(&tag) == current_tag.as_ref() {
                    if let Some(mission) = current_mission.take() {
                        missions.push(mission);
                    }
                    current_tag = None;
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

        let harvest = missions.iter().find(|m| m.mission_type == "harvest").unwrap();
        assert_eq!(harvest.status, MissionStatus::Created);
        assert!((harvest.reward - 8000.0).abs() < 0.01);
        assert!((harvest.completion - 0.0).abs() < 0.01);
        assert_eq!(harvest.field_id, Some(9));
        assert_eq!(harvest.fruit_type.as_deref(), Some("WHEAT"));
        assert_eq!(harvest.expected_liters, Some(50000.0));
        assert_eq!(harvest.deposited_liters, Some(0.0));
    }

    #[test]
    fn test_parse_missions_statuses() {
        let path = fixtures_path().join("savegame_complete");
        let missions = parse_missions(&path).unwrap();

        let created = missions.iter().filter(|m| m.status == MissionStatus::Created).count();
        assert!(created >= 1);

        let running = missions.iter().find(|m| m.status == MissionStatus::Running);
        assert!(running.is_some());
    }

    #[test]
    fn test_parse_missions_types() {
        let path = fixtures_path().join("savegame_complete");
        let missions = parse_missions(&path).unwrap();

        let types: Vec<&str> = missions.iter().map(|m| m.mission_type.as_str()).collect();
        assert!(types.contains(&"harvest"));
        assert!(types.contains(&"plow"));
    }

    #[test]
    fn test_parse_missions_child_elements() {
        let path = fixtures_path().join("savegame_complete");
        let missions = parse_missions(&path).unwrap();

        // Check that info child element was parsed
        let plow = missions.iter().find(|m| m.mission_type == "plow").unwrap();
        assert!((plow.reward - 5000.0).abs() < 0.01);
        assert!((plow.reimbursement - 800.0).abs() < 0.01);
        assert_eq!(plow.field_id, Some(5));
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
