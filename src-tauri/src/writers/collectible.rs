use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::writers::replace_xml_file;
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
                let tag = e.name().as_ref().to_string();
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

    replace_xml_file(&xml_path, &content, &writer.into_inner())?;

    Ok(())
}

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
        .unwrap_or_default()
}

fn attr_u32(e: &BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn patch_collectible(e: &BytesStart, change: &CollectibleChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("collectible");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "collected" => {
                elem.push_attribute(("collected", if change.collected { "true" } else { "false" }));
            }
            _ => {
                elem.push_attribute(attr);
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

    fn reference_copy(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames")
            .join("collectibles.xml");
        let dst = std::env::temp_dir().join(format!("fs25_test_wc_ref_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        std::fs::copy(src, dst.join("collectibles.xml")).unwrap();
        dst
    }

    #[test]
    fn test_write_collectible_on_reference_save() {
        let save = reference_copy("toggle");
        let changes = vec![CollectibleChange { index: 5, collected: true }];
        write_collectible_changes(&save, &changes).unwrap();

        let after = parse_collectibles(&save).unwrap();
        assert!(after.iter().find(|c| c.index == 5).unwrap().collected);
        assert!(!after.iter().find(|c| c.index == 20).unwrap().collected);

        let xml = std::fs::read_to_string(save.join("collectibles.xml")).unwrap();
        assert!(xml.contains(r#"<collectible index="5" collected="true"/>"#));
        assert!(!xml.contains("isCollected"));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_collectible_roundtrip() {
        let save = reference_copy("roundtrip");
        let before = parse_collectibles(&save).unwrap();

        let changes = vec![
            CollectibleChange { index: 20, collected: true },
            CollectibleChange { index: 7, collected: true },
        ];
        write_collectible_changes(&save, &changes).unwrap();
        let after = parse_collectibles(&save).unwrap();

        assert_eq!(after.len(), before.len());
        let order: Vec<u32> = after.iter().map(|c| c.index).collect();
        assert_eq!(order, before.iter().map(|c| c.index).collect::<Vec<_>>());
        assert_eq!(after.iter().filter(|c| c.collected).count(), 2);

        let changes = vec![CollectibleChange { index: 20, collected: false }];
        write_collectible_changes(&save, &changes).unwrap();
        let after = parse_collectibles(&save).unwrap();
        assert!(!after.iter().find(|c| c.index == 20).unwrap().collected);
        assert!(after.iter().find(|c| c.index == 7).unwrap().collected);

        let _ = std::fs::remove_dir_all(&save);
    }
}
