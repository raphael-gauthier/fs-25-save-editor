use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::husbandry::{AnimalCluster, Husbandry};
use crate::models::placeable::{display_name_from_unique_id, placeable_display_name};

// Not yet wired into a command — these exist ahead of the feature that will call them.
// Remove the allows once a Tauri command invokes `parse_husbandries`.
#[allow(dead_code)]
fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

#[allow(dead_code)]
fn attr_f64(e: &quick_xml::events::BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

#[allow(dead_code)]
fn attr_u8(e: &quick_xml::events::BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

#[allow(dead_code)]
fn attr_u32(e: &quick_xml::events::BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

/// Extracts every animal husbandry from placeables.xml.
///
/// This is a second pass over the same file `parse_placeables` reads. Husbandries keep
/// their own change payload so that editing animals can never collide with editing
/// buildings, and `placeable_index` is what ties the two together.
#[allow(dead_code)]
pub fn parse_husbandries(path: &Path) -> Result<Vec<Husbandry>, AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut husbandries: Vec<Husbandry> = Vec::new();

    // Counts EVERY <placeable> so indices stay aligned with parse_placeables.
    let mut placeable_index: usize = 0;
    let mut current: Option<Husbandry> = None;
    let mut is_husbandry = false;
    let mut in_clusters = false;

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;

        let (e, is_empty) = match &event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                match e.name().as_ref() {
                    b"placeable" => {
                        if is_husbandry {
                            if let Some(h) = current.take() {
                                husbandries.push(h);
                            }
                        }
                        current = None;
                        is_husbandry = false;
                        in_clusters = false;
                    }
                    b"clusters" => in_clusters = false,
                    _ => {}
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };

        match e.name().as_ref() {
            b"placeable" => {
                // Guards a malformed file: flush a previous husbandry whose
                // </placeable> never arrived before this one started.
                if is_husbandry {
                    if let Some(h) = current.take() {
                        husbandries.push(h);
                    }
                }
                is_husbandry = false;
                in_clusters = false;

                let filename = attr_str(e, "filename");
                let unique_id = attr_str(e, "uniqueId");
                let display_name = if filename.is_empty() {
                    display_name_from_unique_id(&unique_id)
                } else {
                    placeable_display_name(&filename)
                };

                current = Some(Husbandry {
                    placeable_index,
                    display_name,
                    farm_id: attr_u8(e, "farmId"),
                    production_factor: 0.0,
                    clusters: Vec::new(),
                    food: Vec::new(),
                    water: Vec::new(),
                    meadow: Vec::new(),
                });
                placeable_index += 1;

                if is_empty {
                    current = None;
                }
            }
            b"husbandry" => {
                is_husbandry = true;
                if let Some(ref mut h) = current {
                    h.production_factor = attr_f64(e, "productionFactor");
                }
            }
            b"husbandryAnimals" => is_husbandry = true,
            b"clusters" => in_clusters = !is_empty,
            b"animal" if in_clusters => {
                if let Some(ref mut h) = current {
                    h.clusters.push(AnimalCluster {
                        index: h.clusters.len(),
                        sub_type: attr_str(e, "subType"),
                        num_animals: attr_u32(e, "numAnimals"),
                        age: attr_u32(e, "age"),
                        health: attr_f64(e, "health"),
                        reproduction: attr_f64(e, "reproduction"),
                    });
                }
            }
            // Food, water and pasture are added in Task 5.
            _ => {}
        }
    }

    if is_husbandry {
        if let Some(h) = current.take() {
            husbandries.push(h);
        }
    }

    Ok(husbandries)
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
    fn test_parse_husbandries_finds_only_husbandry_placeables() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();
        assert_eq!(husbandries.len(), 3);
    }

    #[test]
    fn test_parse_husbandries_single_cluster() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Chicken Barn Big"))
            .unwrap();
        assert_eq!(barn.farm_id, 1);
        assert_eq!(barn.clusters.len(), 1);

        let cluster = &barn.clusters[0];
        assert_eq!(cluster.index, 0);
        assert_eq!(cluster.sub_type, "CHICKEN");
        assert_eq!(cluster.num_animals, 360);
        assert_eq!(cluster.age, 48);
        assert!((cluster.health - 100.0).abs() < 0.01);
        assert!((cluster.reproduction - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_multiple_clusters_keep_document_order() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Sheep Barn Small"))
            .unwrap();
        assert_eq!(barn.clusters.len(), 2);
        assert_eq!(barn.clusters[0].index, 0);
        assert_eq!(barn.clusters[0].num_animals, 20);
        assert_eq!(barn.clusters[1].index, 1);
        assert_eq!(barn.clusters[1].num_animals, 71);
        assert!((barn.clusters[1].health - 90.0).abs() < 0.01);
        assert!((barn.production_factor - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_empty_preplaced_is_kept_and_named() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        // No <husbandryAnimals> at all, and no filename to derive a name from.
        let coop = husbandries
            .iter()
            .find(|h| h.display_name == "Chicken Coop 02")
            .unwrap();
        assert!(coop.clusters.is_empty());
        assert!((coop.production_factor - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_placeable_index_matches_placeable_parser() {
        // The writer finds its target by this index, so it must agree with the
        // numbering parse_placeables assigns — counting every <placeable>, not
        // just the ones holding animals.
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();
        let placeables = crate::parsers::placeable::parse_placeables(&path).unwrap();

        assert!(
            !husbandries.is_empty(),
            "no husbandries parsed — the index alignment check would pass vacuously"
        );
        for h in &husbandries {
            let p = placeables
                .iter()
                .find(|p| p.index == h.placeable_index)
                .unwrap();
            assert_eq!(p.display_name, h.display_name);
        }
    }

    #[test]
    fn test_parse_husbandries_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_husbandry");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_husbandries(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
