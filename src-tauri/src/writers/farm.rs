use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;

/// Modifies money and/or loan in farms.xml for the specified farm.
/// Uses patch strategy: reads, modifies only the target attributes, rewrites atomically.
pub fn write_farm_finances(
    path: &Path,
    farm_id: u8,
    money: Option<f64>,
    loan: Option<f64>,
) -> Result<(), AppError> {
    let xml_path = path.join("farms.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "farm" {
                    let current_id: u8 = e
                        .attributes()
                        .flatten()
                        .find(|a| a.key.as_ref() == b"farmId")
                        .map(|a| {
                            String::from_utf8_lossy(&a.value)
                                .parse()
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);

                    if current_id == farm_id {
                        let mut elem = BytesStart::new("farm");
                        for attr in e.attributes().flatten() {
                            let key =
                                String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            match key.as_str() {
                                "money" if money.is_some() => {
                                    elem.push_attribute((
                                        "money",
                                        format!("{:.6}", money.unwrap()).as_str(),
                                    ));
                                }
                                "loan" if loan.is_some() => {
                                    elem.push_attribute((
                                        "loan",
                                        format!("{:.6}", loan.unwrap()).as_str(),
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
                        writer
                            .write_event(Event::Start(elem))
                            .map_err(|e| AppError::XmlParseError {
                                file: xml_path.display().to_string(),
                                message: e.to_string(),
                            })?;
                    } else {
                        writer
                            .write_event(Event::Start(e.clone().into_owned()))
                            .map_err(|e| AppError::XmlParseError {
                                file: xml_path.display().to_string(),
                                message: e.to_string(),
                            })?;
                    }
                } else {
                    writer
                        .write_event(Event::Start(e.clone().into_owned()))
                        .map_err(|e| AppError::XmlParseError {
                            file: xml_path.display().to_string(),
                            message: e.to_string(),
                        })?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                writer
                    .write_event(event.into_owned())
                    .map_err(|e| AppError::XmlParseError {
                        file: xml_path.display().to_string(),
                        message: e.to_string(),
                    })?;
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
    use crate::parsers::farm::parse_farms;

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
    fn test_write_farm_money() {
        let save = setup_fixture("money");
        write_farm_finances(&save, 1, Some(777777.0), None).unwrap();
        let farms = parse_farms(&save).unwrap();
        assert!((farms[0].money - 777777.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_farm_loan() {
        let save = setup_fixture("loan");
        write_farm_finances(&save, 1, None, Some(12345.0)).unwrap();
        let farms = parse_farms(&save).unwrap();
        assert!((farms[0].loan - 12345.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_farm_preserves_stats() {
        let save = setup_fixture("preserve");
        let before = parse_farms(&save).unwrap();
        write_farm_finances(&save, 1, Some(1.0), Some(2.0)).unwrap();
        let after = parse_farms(&save).unwrap();

        assert_eq!(after[0].farm_id, before[0].farm_id);
        assert_eq!(after[0].name, before[0].name);
        assert!((after[0].statistics.revenue - before[0].statistics.revenue).abs() < 0.01);
        assert_eq!(after[0].players.len(), before[0].players.len());
        let _ = std::fs::remove_dir_all(&save);
    }
}
