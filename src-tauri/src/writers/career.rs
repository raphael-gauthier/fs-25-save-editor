use std::path::Path;

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::career::GameSetting;
use crate::validators::game_settings::normalize_setting;
use crate::writers::replace_xml_file;

/// Sets editable game settings in careerSavegame.xml's `<settings>`. Every
/// value is checked first: nothing is written if one is refused.
pub fn write_game_settings(path: &Path, changes: &[GameSetting]) -> Result<(), AppError> {
    let xml_path = path.join("careerSavegame.xml");
    let invalid = |message: String| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message,
    };
    let values: Vec<(String, String)> = changes
        .iter()
        .map(|c| normalize_setting(&c.key, &c.value).map(|v| (c.key.clone(), v)))
        .collect::<Result<_, _>>()
        .map_err(invalid)?;

    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;
    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());
    let mut in_settings = false;
    // The new text of the setting element being rewritten
    let mut replacing: Option<&str> = None;
    let write_err = |e: std::io::Error| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = e.name().as_ref().to_string();
                if tag == "settings" {
                    in_settings = true;
                } else if in_settings {
                    replacing = values.iter().find(|(k, _)| *k == tag).map(|(_, v)| v.as_str());
                }
                writer.write_event(Event::Start(e.clone())).map_err(write_err)?;
            }
            // The old text of a replaced setting is dropped
            Ok(Event::Text(_)) | Ok(Event::GeneralRef(_)) if replacing.is_some() => {}
            Ok(Event::End(ref e)) => {
                if let Some(value) = replacing.take() {
                    writer.write_event(Event::Text(BytesText::new(value))).map_err(write_err)?;
                }
                if e.name().as_ref() == "settings" {
                    in_settings = false;
                }
                writer.write_event(Event::End(e.clone())).map_err(write_err)?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => writer.write_event(event).map_err(write_err)?,
            Err(e) => return Err(invalid(e.to_string())),
        }
    }

    replace_xml_file(&xml_path, &content, &writer.into_inner())?;
    Ok(())
}

/// Modifies the money in careerSavegame.xml.
/// Supports both formats:
///   - Self-closing: `<statistics money="..." playTime="..." />`
///   - Child elements: `<statistics><money>...</money><playTime>...</playTime></statistics>`
pub fn write_career_money(path: &Path, money: f64) -> Result<(), AppError> {
    let xml_path = path.join("careerSavegame.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut in_statistics = false;
    let mut in_money_tag = false;

    let write_err = |e: std::io::Error| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = e.name().as_ref().to_string();
                if tag == "statistics" {
                    in_statistics = true;
                    // Pass through with attributes (may have attrs in some formats)
                    let mut elem = BytesStart::new("statistics");
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref().to_string();
                        if key == "money" {
                            elem.push_attribute(("money", format!("{:.6}", money).as_str()));
                        } else {
                            elem.push_attribute(attr);
                        }
                    }
                    writer.write_event(Event::Start(elem)).map_err(write_err)?;
                } else if in_statistics && tag == "money" {
                    in_money_tag = true;
                    writer.write_event(Event::Start(e.clone())).map_err(write_err)?;
                } else {
                    writer.write_event(Event::Start(e.clone())).map_err(write_err)?;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_money_tag {
                    // Replace the money text content
                    let money_str = format!("{}", money as i64);
                    writer
                        .write_event(Event::Text(BytesText::new(&money_str)))
                        .map_err(write_err)?;
                } else {
                    writer.write_event(Event::Text(e.clone())).map_err(write_err)?;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name().as_ref().to_string();
                if tag == "money" && in_statistics {
                    in_money_tag = false;
                } else if tag == "statistics" {
                    in_statistics = false;
                }
                writer.write_event(Event::End(e.clone())).map_err(write_err)?;
            }
            Ok(Event::Empty(ref e)) => {
                let tag = e.name().as_ref().to_string();
                if tag == "statistics" {
                    // Self-closing format: <statistics money="..." />
                    let mut elem = BytesStart::new("statistics");
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref().to_string();
                        if key == "money" {
                            elem.push_attribute(("money", format!("{:.6}", money).as_str()));
                        } else {
                            elem.push_attribute(attr);
                        }
                    }
                    writer.write_event(Event::Empty(elem)).map_err(write_err)?;
                } else {
                    writer.write_event(Event::Empty(e.clone())).map_err(write_err)?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                writer.write_event(event.into_owned()).map_err(write_err)?;
            }
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    replace_xml_file(&xml_path, &content, &writer.into_inner())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::career::{parse_career, parse_game_settings};

    fn reference_copy(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames")
            .join("careerSavegame.xml");
        let dir = std::env::temp_dir().join(format!("fs25_test_wc_settings_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(src, dir.join("careerSavegame.xml")).unwrap();
        dir
    }

    fn setting(key: &str, value: &str) -> GameSetting {
        GameSetting { key: key.into(), value: value.into() }
    }

    #[test]
    fn test_write_game_settings() {
        let dir = reference_copy("write");
        let before = parse_game_settings(&dir).unwrap();
        let changes = [
            setting("savegameName", "Ferme & Fils"),
            setting("economicDifficulty", "HARD"),
            setting("stonesEnabled", "true"),
            setting("timeScale", "5"),
        ];

        write_game_settings(&dir, &changes).unwrap();

        let after = parse_game_settings(&dir).unwrap();
        let value = |key: &str| after.iter().find(|s| s.key == key).unwrap().value.clone();
        assert_eq!(value("savegameName"), "Ferme & Fils");
        assert_eq!(value("economicDifficulty"), "HARD");
        assert_eq!(value("stonesEnabled"), "true");
        assert_eq!(value("timeScale"), "5.000000");
        let unchanged = |s: &&GameSetting| !changes.iter().any(|c| c.key == s.key);
        assert_eq!(
            after.iter().filter(unchanged).collect::<Vec<_>>(),
            before.iter().filter(unchanged).collect::<Vec<_>>()
        );
        // The rest of careerSavegame.xml still reads the same
        assert_eq!(parse_career(&dir).unwrap().map_id, "MapUS");
        let xml = std::fs::read_to_string(dir.join("careerSavegame.xml")).unwrap();
        assert!(xml.contains("<savegameName>Ferme &amp; Fils</savegameName>"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_write_game_settings_refuses_any_invalid_value() {
        let dir = reference_copy("refuse");
        let original = std::fs::read(dir.join("careerSavegame.xml")).unwrap();
        let changes = [setting("stonesEnabled", "true"), setting("fuelUsage", "9")];

        assert!(write_game_settings(&dir, &changes).is_err());
        assert!(write_game_settings(&dir, &[setting("densityMapRevision", "5")]).is_err());

        assert!(std::fs::read(dir.join("careerSavegame.xml")).unwrap() == original);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wc_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        // Copy individual files
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
            }
        }
        dst
    }

    #[test]
    fn test_write_career_money() {
        let save = setup_fixture("money");
        write_career_money(&save, 999888.123456).unwrap();
        let career = parse_career(&save).unwrap();
        assert!((career.money - 999888.123456).abs() < 0.001);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_career_money_preserves_rest() {
        let save = setup_fixture("preserve");
        let before = parse_career(&save).unwrap();
        write_career_money(&save, 42.0).unwrap();
        let after = parse_career(&save).unwrap();

        assert_eq!(after.savegame_name, before.savegame_name);
        assert_eq!(after.map_title, before.map_title);
        assert!((after.play_time - before.play_time).abs() < 0.01);
        assert_eq!(after.growth_mode, before.growth_mode);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_career_money_keeps_escaped_savegame_name() {
        let save = setup_fixture("escaped_name");
        let path = save.join("careerSavegame.xml");
        let original = std::fs::read_to_string(&path).unwrap();
        let start = original.find("<savegameName>").unwrap();
        let end = original.find("</savegameName>").unwrap();
        let escaped = format!("{}<savegameName>Ferme &amp; Fils{}", &original[..start], &original[end..]);
        std::fs::write(&path, escaped).unwrap();

        write_career_money(&save, 1234.0).unwrap();
        let written = std::fs::read_to_string(&path).unwrap();
        let career = parse_career(&save).unwrap();
        let _ = std::fs::remove_dir_all(&save);

        assert!(written.contains("<savegameName>Ferme &amp; Fils</savegameName>"));
        assert_eq!(career.savegame_name, "Ferme & Fils");
    }
}
