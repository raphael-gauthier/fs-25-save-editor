use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::CollectibleChange;

pub fn write_collectible_changes(
    path: &Path,
    changes: &[CollectibleChange],
) -> Result<(), AppError> {
    let xml_path = path.join("collectibles.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<u32, &CollectibleChange> = changes
        .iter()
        .map(|c| (c.index, c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "collectible" {
                    let index = attr_u32(e, "index");
                    if let Some(change) = change_map.get(&index) {
                        let elem = patch_collectible(e, change);
                        write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                    } else {
                        write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                    }
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

fn patch_collectible(e: &BytesStart, change: &CollectibleChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("collectible");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "isCollected" => {
                elem.push_attribute(("isCollected", if change.collected { "true" } else { "false" }));
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
    use crate::parsers::collectible::parse_collectibles;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wc_{}", name));
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
    fn test_write_collectible_toggle() {
        let save = setup_fixture("toggle");
        // Collectible index 3 is not collected (index 0-based: 3 = 4th item)
        let before = parse_collectibles(&save).unwrap();
        let c3 = before.iter().find(|c| c.index == 3).unwrap();
        assert!(!c3.collected);

        let changes = vec![CollectibleChange {
            index: 3,
            collected: true,
        }];
        write_collectible_changes(&save, &changes).unwrap();
        let after = parse_collectibles(&save).unwrap();
        let c3_after = after.iter().find(|c| c.index == 3).unwrap();
        assert!(c3_after.collected);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_collectible_roundtrip() {
        let save = setup_fixture("roundtrip_c");
        let before = parse_collectibles(&save).unwrap();

        // Toggle a few
        let changes = vec![
            CollectibleChange { index: 0, collected: false },  // was true
            CollectibleChange { index: 3, collected: true },   // was false
        ];
        write_collectible_changes(&save, &changes).unwrap();
        let after = parse_collectibles(&save).unwrap();

        assert_eq!(after.len(), before.len());
        assert!(!after.iter().find(|c| c.index == 0).unwrap().collected);
        assert!(after.iter().find(|c| c.index == 3).unwrap().collected);

        // Untouched items preserved
        let c1_before = before.iter().find(|c| c.index == 1).unwrap();
        let c1_after = after.iter().find(|c| c.index == 1).unwrap();
        assert_eq!(c1_before.collected, c1_after.collected);

        let _ = std::fs::remove_dir_all(&save);
    }
}
