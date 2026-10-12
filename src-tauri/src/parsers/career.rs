use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::parsers::text::resolve_ref;
use crate::models::career::{CareerSavegame, SavegameSummary};

/// Parse careerSavegame.xml and extract only the fields needed for the summary.
/// Uses quick-xml event-based Reader for selective lightweight parsing.
pub fn parse_career_summary(path: &Path) -> Result<SavegameSummary, AppError> {
    let xml_path = path.join("careerSavegame.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);

    let mut savegame_name = String::new();
    let mut map_title = String::new();
    let mut save_date = String::new();
    let mut economic_difficulty = String::from("NORMAL");
    let mut money: f64 = 0.0;
    let mut play_time: f64 = 0.0;

    let mut current_tag = String::new();
    let mut text = String::new();
    let mut in_settings = false;
    let mut in_statistics = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                text.clear();
                let tag_name = e.name().as_ref().to_string();
                match tag_name.as_str() {
                    "settings" => in_settings = true,
                    "statistics" => {
                        in_statistics = true;
                        // Also check attributes (format: <statistics money="..." />)
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref().to_string();
                            let val = attr.value.to_string();
                            match key.as_str() {
                                "money" => money = val.parse().unwrap_or(0.0),
                                "playTime" => play_time = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                if in_settings {
                    current_tag = tag_name;
                } else if in_statistics {
                    current_tag = tag_name;
                }
            }
            Ok(Event::Text(ref e)) => text.push_str(e),
            Ok(Event::GeneralRef(ref r)) => text.push_str(&resolve_ref(r)),
            Ok(Event::End(ref e)) => {
                let text = std::mem::take(&mut text);
                // An empty element yields no text: keep the field's default.
                if !text.is_empty() {
                    if in_settings {
                        match current_tag.as_str() {
                            "savegameName" => savegame_name = text,
                            "mapTitle" => map_title = text,
                            "saveDateFormatted" => save_date = text,
                            "economicDifficulty" => {
                                economic_difficulty = match text.trim() {
                                    "1" => "EASY".to_string(),
                                    "2" => "NORMAL".to_string(),
                                    "3" => "HARD".to_string(),
                                    other => other.to_string(),
                                };
                            }
                            _ => {}
                        }
                    } else if in_statistics {
                        match current_tag.as_str() {
                            "money" => money = text.trim().parse().unwrap_or(0.0),
                            "playTime" => play_time = text.trim().parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
                }
                let tag_name = e.name().as_ref().to_string();
                if tag_name == "settings" {
                    in_settings = false;
                } else if tag_name == "statistics" {
                    in_statistics = false;
                }
                current_tag.clear();
            }
            Ok(Event::Empty(ref e)) => {
                let tag_name = e.name().as_ref().to_string();
                // Self-closing format: <statistics money="..." playTime="..." />
                if tag_name == "statistics" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref().to_string();
                        let val = attr.value.to_string();
                        match key.as_str() {
                            "money" => money = val.parse().unwrap_or(0.0),
                            "playTime" => play_time = val.parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
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

    Ok(SavegameSummary {
        path: path.display().to_string(),
        name: savegame_name,
        map_title,
        money,
        play_time,
        save_date,
        economic_difficulty,
    })
}

/// Parse careerSavegame.xml with all CareerSavegame fields.
pub fn parse_career(path: &Path) -> Result<CareerSavegame, AppError> {
    let xml_path = path.join("careerSavegame.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);

    let mut savegame_name = String::new();
    let mut creation_date = String::new();
    let mut map_id = String::new();
    let mut map_title = String::new();
    let mut save_date = String::new();
    let mut economic_difficulty = String::from("NORMAL");
    let mut money: f64 = 0.0;
    let mut play_time: f64 = 0.0;
    let mut growth_mode: u8 = 1;
    let mut planned_days_per_period: u8 = 1;
    let mut plowing_required = false;
    let mut stones_enabled = false;
    let mut weeds_enabled = false;
    let mut lime_required = false;
    let mut snow_enabled = false;
    let mut fuel_usage: u8 = 1;
    let mut traffic_enabled = true;

    let mut current_tag = String::new();
    let mut text = String::new();
    let mut in_settings = false;
    let mut in_statistics = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                text.clear();
                let tag_name = e.name().as_ref().to_string();
                match tag_name.as_str() {
                    "settings" => in_settings = true,
                    "statistics" => {
                        in_statistics = true;
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref().to_string();
                            let val = attr.value.to_string();
                            match key.as_str() {
                                "money" => money = val.parse().unwrap_or(0.0),
                                "playTime" => play_time = val.parse().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                if in_settings {
                    current_tag = tag_name;
                } else if in_statistics {
                    current_tag = tag_name;
                }
            }
            Ok(Event::Text(ref e)) => text.push_str(e),
            Ok(Event::GeneralRef(ref r)) => text.push_str(&resolve_ref(r)),
            Ok(Event::End(ref e)) => {
                let text = std::mem::take(&mut text);
                // An empty element yields no text: keep the field's default.
                if !text.is_empty() {
                    if in_settings {
                        match current_tag.as_str() {
                            "savegameName" => savegame_name = text,
                            "creationDate" => creation_date = text,
                            "mapId" => map_id = text,
                            "mapTitle" => map_title = text,
                            "saveDateFormatted" => save_date = text,
                            "economicDifficulty" => {
                                economic_difficulty = match text.trim() {
                                    "1" => "EASY".to_string(),
                                    "2" => "NORMAL".to_string(),
                                    "3" => "HARD".to_string(),
                                    other => other.to_string(),
                                };
                            }
                            "growthMode" => growth_mode = text.trim().parse().unwrap_or(1),
                            "plannedDaysPerPeriod" => {
                                planned_days_per_period = text.trim().parse().unwrap_or(1)
                            }
                            "plowingRequiredEnabled" => {
                                plowing_required = text.trim() == "true"
                            }
                            "stonesEnabled" => stones_enabled = text.trim() == "true",
                            "weedsEnabled" => weeds_enabled = text.trim() == "true",
                            "limeRequired" => lime_required = text.trim() == "true",
                            "snowEnabled" | "isSnowEnabled" => snow_enabled = text.trim() == "true",
                            "fuelUsage" => fuel_usage = text.trim().parse().unwrap_or(1),
                            "trafficEnabled" => traffic_enabled = text.trim() == "true",
                            _ => {}
                        }
                    } else if in_statistics {
                        match current_tag.as_str() {
                            "money" => money = text.trim().parse().unwrap_or(0.0),
                            "playTime" => play_time = text.trim().parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
                }
                let tag_name = e.name().as_ref().to_string();
                if tag_name == "settings" {
                    in_settings = false;
                } else if tag_name == "statistics" {
                    in_statistics = false;
                }
                current_tag.clear();
            }
            Ok(Event::Empty(ref e)) => {
                let tag_name = e.name().as_ref().to_string();
                if tag_name == "statistics" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref().to_string();
                        let val = attr.value.to_string();
                        match key.as_str() {
                            "money" => money = val.parse().unwrap_or(0.0),
                            "playTime" => play_time = val.parse().unwrap_or(0.0),
                            _ => {}
                        }
                    }
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

    Ok(CareerSavegame {
        savegame_name,
        creation_date,
        map_id,
        map_title,
        save_date,
        economic_difficulty,
        money,
        play_time,
        growth_mode,
        planned_days_per_period,
        plowing_required,
        stones_enabled,
        weeds_enabled,
        lime_required,
        snow_enabled,
        fuel_usage,
        traffic_enabled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixtures_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("test_saves")
    }

    #[test]
    fn test_parse_career_summary_nominal() {
        let path = fixtures_path().join("savegame1");
        let summary = parse_career_summary(&path).unwrap();
        assert_eq!(summary.name, "Ma partie");
        assert_eq!(summary.map_title, "Riverbend Springs");
        assert!((summary.money - 1_000_000.0).abs() < 0.01);
        assert!((summary.play_time - 12345.678).abs() < 0.01);
        assert_eq!(summary.save_date, "2025-01-15");
        assert_eq!(summary.economic_difficulty, "NORMAL");
    }

    #[test]
    fn test_parse_career_summary_minimal() {
        let path = fixtures_path().join("savegame2");
        let summary = parse_career_summary(&path).unwrap();
        assert_eq!(summary.name, "Partie 2");
        assert_eq!(summary.map_title, "Elm Creek");
        assert_eq!(summary.economic_difficulty, "EASY");
    }

    #[test]
    fn test_parse_career_summary_invalid_xml() {
        let dir = std::env::temp_dir().join("fs25_test_invalid_xml");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("careerSavegame.xml"), "<broken><xml").unwrap();

        let result = parse_career_summary(&dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_career_summary_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_missing_file");
        let _ = fs::create_dir_all(&dir);

        let result = parse_career_summary(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));

        let _ = fs::remove_dir_all(&dir);
    }

    /// The reference save with only its name changed to one the game writes
    /// escaped: `&` is stored as `&amp;`.
    fn reference_save_named(dir_name: &str, escaped_name: &str) -> std::path::PathBuf {
        let reference = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames")
            .join("careerSavegame.xml");
        let content = fs::read_to_string(reference).unwrap().replace(
            "<savegameName>Ma partie</savegameName>",
            &format!("<savegameName>{}</savegameName>", escaped_name),
        );
        assert!(content.contains(escaped_name));
        let dir = std::env::temp_dir().join(dir_name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("careerSavegame.xml"), content).unwrap();
        dir
    }

    #[test]
    fn test_parse_career_decodes_entities_in_savegame_name() {
        let dir = reference_save_named("fs25_test_career_entities", "Ferme &amp; Fils &#233;t&#xE9;");

        let summary = parse_career_summary(&dir).unwrap();
        let career = parse_career(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(summary.name, "Ferme & Fils été");
        assert_eq!(career.savegame_name, "Ferme & Fils été");
    }
}
