use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::husbandry::{AnimalCluster, FillStock, Husbandry, MeadowStock};
use crate::models::placeable::{display_name_from_unique_id, placeable_display_name};
use crate::parsers::attrs::{attr_f64, attr_str, attr_u32, attr_u8};

/// Extracts every animal husbandry from placeables.xml.
///
/// This is a second pass over the same file `parse_placeables` reads. Husbandries keep
/// their own change payload so that editing animals can never collide with editing
/// buildings, and `placeable_index` is what ties the two together.
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
    // <storage>/<node> are also used by silos and production points, so water
    // collection is gated on being inside <husbandry>.
    let mut in_husbandry = false;
    let mut in_husbandry_storage = false;
    let mut in_food = false;
    let mut in_meadow = false;

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
                    "placeable" => {
                        if is_husbandry {
                            if let Some(h) = current.take() {
                                husbandries.push(h);
                            }
                        }
                        current = None;
                        is_husbandry = false;
                        in_clusters = false;
                        in_husbandry = false;
                        in_husbandry_storage = false;
                        in_food = false;
                        in_meadow = false;
                    }
                    "husbandry" => {
                        in_husbandry = false;
                        in_husbandry_storage = false;
                    }
                    "storage" if in_husbandry => in_husbandry_storage = false,
                    "clusters" => in_clusters = false,
                    "husbandryFood" => in_food = false,
                    "husbandryMeadow" => in_meadow = false,
                    _ => {}
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };

        match e.name().as_ref() {
            "placeable" => {
                // Guards a malformed file: flush a previous husbandry whose
                // </placeable> never arrived before this one started.
                if is_husbandry {
                    if let Some(h) = current.take() {
                        husbandries.push(h);
                    }
                }
                is_husbandry = false;
                in_clusters = false;
                in_husbandry = false;
                in_husbandry_storage = false;
                in_food = false;
                in_meadow = false;

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
            "husbandry" => {
                is_husbandry = true;
                in_husbandry = !is_empty;
                if let Some(ref mut h) = current {
                    h.production_factor = attr_f64(e, "productionFactor");
                }
            }
            "husbandryAnimals" => is_husbandry = true,
            "clusters" => in_clusters = !is_empty,
            "animal" if in_clusters => {
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
            "husbandryFood" => {
                is_husbandry = true;
                in_food = !is_empty;
            }
            "fillLevel" if in_food => {
                if let Some(ref mut h) = current {
                    h.food.push(FillStock {
                        fill_type: attr_str(e, "fillType"),
                        amount: attr_f64(e, "fillLevel"),
                    });
                }
            }
            "husbandryMeadow" => {
                is_husbandry = true;
                in_meadow = !is_empty;
            }
            "fillType" if in_meadow => {
                if let Some(ref mut h) = current {
                    h.meadow.push(MeadowStock {
                        fill_type: attr_str(e, "name"),
                        amount: attr_f64(e, "fillLevel"),
                        capacity: attr_f64(e, "capacity"),
                    });
                }
            }
            "storage" if in_husbandry => in_husbandry_storage = !is_empty,
            "node" if in_husbandry_storage => {
                if let Some(ref mut h) = current {
                    h.water.push(FillStock {
                        fill_type: attr_str(e, "fillType"),
                        amount: attr_f64(e, "fillLevel"),
                    });
                }
            }
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
    fn test_parse_husbandries_food_stock() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Chicken Barn Big"))
            .unwrap();
        assert_eq!(barn.food.len(), 1);
        assert_eq!(barn.food[0].fill_type, "WHEAT");
        assert!((barn.food[0].amount - 5400.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_water_and_meadow() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Sheep Barn Small"))
            .unwrap();

        assert_eq!(barn.water.len(), 1);
        assert_eq!(barn.water[0].fill_type, "WATER");
        assert!((barn.water[0].amount - 1500.0).abs() < 0.01);

        // The meadow is the only stock the savegame gives a capacity for.
        assert_eq!(barn.meadow.len(), 1);
        assert_eq!(barn.meadow[0].fill_type, "GRASS_WINDROW");
        assert!((barn.meadow[0].amount - 21689.705078).abs() < 0.01);
        assert!((barn.meadow[0].capacity - 25693.414062).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_ignores_silo_and_production_storage() {
        // Fixture index 0 is a silo with <silo><storage><node fillType="WHEAT">, and
        // index 3 is a production point with its own <storage><node>. Neither is a
        // husbandry, so neither may leak into the results.
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        assert!(husbandries.iter().all(|h| h.placeable_index >= 6));
        for h in &husbandries {
            assert!(
                h.water.iter().all(|w| w.fill_type == "WATER"),
                "silo or production stock leaked into husbandry water"
            );
        }

        // Chicken Barn Big has a self-closing <storage farmId="1"/> with no <node>
        // children, so its water must be empty. This pins the !is_empty handling.
        let coop = husbandries
            .iter()
            .find(|h| h.display_name.contains("Chicken Barn Big"))
            .unwrap();
        assert!(
            coop.water.is_empty(),
            "a self-closing <storage/> must not produce water entries"
        );

        // Values that exist only in the silo (placeable 0) and the production
        // point (placeable 2). If scoping breaks, they surface here by value.
        let foreign = [20657.392578_f64, 4855.552734, 2500.0, 1200.0];
        for h in &husbandries {
            for stock in h.food.iter().chain(h.water.iter()) {
                for f in foreign {
                    assert!(
                        (stock.amount - f).abs() > 0.001,
                        "{} leaked a foreign stock value {} as {}",
                        h.display_name,
                        f,
                        stock.fill_type
                    );
                }
            }
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
