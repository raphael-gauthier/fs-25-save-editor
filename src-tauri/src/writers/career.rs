use std::path::Path;

use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;

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
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "statistics" {
                    in_statistics = true;
                    // Pass through with attributes (may have attrs in some formats)
                    let mut elem = BytesStart::new("statistics");
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        if key == "money" {
                            elem.push_attribute(("money", format!("{:.6}", money).as_str()));
                        } else {
                            elem.push_attribute((key.as_str(), String::from_utf8_lossy(&attr.value).as_ref()));
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
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "money" && in_statistics {
                    in_money_tag = false;
                } else if tag == "statistics" {
                    in_statistics = false;
                }
                writer.write_event(Event::End(e.clone())).map_err(write_err)?;
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "statistics" {
                    // Self-closing format: <statistics money="..." />
                    let mut elem = BytesStart::new("statistics");
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        if key == "money" {
                            elem.push_attribute(("money", format!("{:.6}", money).as_str()));
                        } else {
                            elem.push_attribute((key.as_str(), String::from_utf8_lossy(&attr.value).as_ref()));
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

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::career::parse_career;

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
}
