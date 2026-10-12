use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::collectible::Collectible;

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
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
                let tag = e.name().as_ref().to_string();
                if tag == "collectible" {
                    collectibles.push(Collectible {
                        index: attr_u32(e, "index"),
                        collected: attr_str(e, "collected") == "true",
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

    fn reference_save() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames")
    }

    #[test]
    fn test_parse_collectibles_reference_save() {
        let collectibles = parse_collectibles(&reference_save()).unwrap();
        assert_eq!(collectibles.len(), 25);
        assert_eq!(collectibles[0].index, 20);
        assert!(collectibles.iter().all(|c| !c.collected));
    }

    #[test]
    fn test_parse_collectibles_reads_collected_attribute() {
        let dir = std::env::temp_dir().join("fs25_test_collected_attr");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(
            dir.join("collectibles.xml"),
            "<collectibles isComplete=\"false\">
    <collectible index=\"20\" collected=\"true\"/>
    <collectible index=\"5\" collected=\"false\"/>
</collectibles>
",
        )
        .unwrap();
        let collectibles = parse_collectibles(&dir).unwrap();
        assert!(collectibles[0].collected);
        assert!(!collectibles[1].collected);
        let _ = std::fs::remove_dir_all(&dir);
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
