use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::collectible::Collectible;

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

pub fn parse_collectibles(path: &Path) -> Result<Vec<Collectible>, AppError> {
    let xml_path = path.join("collectibles.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut collectibles: Vec<Collectible> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "collectible" {
                    collectibles.push(Collectible {
                        index: attr_u32(e, "index"),
                        collected: attr_str(e, "isCollected") == "true",
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

    Ok(collectibles)
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
    fn test_parse_collectibles_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let collectibles = parse_collectibles(&path).unwrap();
        assert_eq!(collectibles.len(), 25);

        let collected_count = collectibles.iter().filter(|c| c.collected).count();
        assert_eq!(collected_count, 12);

        assert!(collectibles[0].collected);
        assert!(!collectibles[3].collected);
    }

    #[test]
    fn test_parse_collectibles_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_collectibles");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_collectibles(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
