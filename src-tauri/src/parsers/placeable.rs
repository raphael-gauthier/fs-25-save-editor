use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::common::Position;
use crate::models::placeable::{
    display_name_from_unique_id, placeable_display_name, ConstructionMaterial, ConstructionState,
    Placeable, ProductionLine, ProductionStock,
};
use crate::parsers::attrs::{attr_f64, attr_str, attr_u8};

fn attr_bool(e: &quick_xml::events::BytesStart, key: &str) -> bool {
    attr_str(e, key) == "true"
}

/// Parses a `position="x y z"` attribute. Returns `None` when absent or malformed.
fn parse_position(e: &quick_xml::events::BytesStart) -> Option<Position> {
    let raw = attr_str(e, "position");
    if raw.is_empty() {
        return None;
    }
    let coords: Vec<f64> = raw
        .split_whitespace()
        .filter_map(|c| c.parse::<f64>().ok())
        .collect();
    match coords[..] {
        [x, y, z] => Some(Position { x, y, z }),
        _ => None,
    }
}

pub fn parse_placeables(path: &Path) -> Result<Vec<Placeable>, AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut placeables: Vec<Placeable> = Vec::new();

    let mut placeable_index: usize = 0;
    let mut current: Option<Placeable> = None;
    // `<storage>` and `<node>` are shared by silos, husbandries and production points,
    // so stock collection is gated on being inside a `<productionPoint>`.
    let mut in_constructible = false;
    let mut in_production_point = false;
    let mut in_production_storage = false;

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;

        let (elem, is_empty) = match &event {
            Event::Start(e) => (Some(e), false),
            Event::Empty(e) => (Some(e), true),
            Event::Eof => break,
            Event::End(e) => {
                match e.name().as_ref() {
                    b"placeable" => {
                        if let Some(p) = current.take() {
                            placeables.push(p);
                        }
                        in_constructible = false;
                        in_production_point = false;
                        in_production_storage = false;
                    }
                    b"constructible" => in_constructible = false,
                    b"productionPoint" => {
                        in_production_point = false;
                        in_production_storage = false;
                    }
                    b"storage" if in_production_point => in_production_storage = false,
                    _ => {}
                }
                continue;
            }
            _ => continue,
        };

        let Some(e) = elem else { continue };
        let tag = e.name().as_ref().to_vec();

        match tag.as_slice() {
            b"placeable" => {
                if let Some(p) = current.take() {
                    placeables.push(p);
                }
                in_constructible = false;
                in_production_point = false;
                in_production_storage = false;

                let filename = attr_str(e, "filename");
                let unique_id = attr_str(e, "uniqueId");
                let display_name = if filename.is_empty() {
                    display_name_from_unique_id(&unique_id)
                } else {
                    placeable_display_name(&filename)
                };

                let placeable = Placeable {
                    index: placeable_index,
                    unique_id,
                    filename,
                    display_name,
                    farm_id: attr_u8(e, "farmId"),
                    price: attr_f64(e, "price"),
                    age: attr_f64(e, "age"),
                    position: parse_position(e),
                    is_pre_placed: attr_bool(e, "isPreplaced"),
                    is_under_construction: false,
                    construction: None,
                    productions: Vec::new(),
                    production_storage: Vec::new(),
                };

                placeable_index += 1;

                if is_empty {
                    placeables.push(placeable);
                } else {
                    current = Some(placeable);
                }
            }
            b"constructible" if current.is_some() => in_constructible = !is_empty,
            b"state" if in_constructible => {
                if let Some(ref mut p) = current {
                    p.construction = Some(ConstructionState {
                        state_name: attr_str(e, "name"),
                        inputs: Vec::new(),
                    });
                }
            }
            b"input" if in_constructible => {
                if let Some(ref mut p) = current {
                    if let Some(ref mut c) = p.construction {
                        c.inputs.push(ConstructionMaterial {
                            fill_type: attr_str(e, "fillType"),
                            amount_remaining: attr_f64(e, "remainingAmount"),
                        });
                    }
                }
            }
            b"productionPoint" if current.is_some() => in_production_point = !is_empty,
            b"production" if in_production_point => {
                if let Some(ref mut p) = current {
                    p.productions.push(ProductionLine {
                        id: attr_str(e, "id"),
                        is_enabled: attr_bool(e, "isEnabled"),
                    });
                }
            }
            b"storage" if in_production_point => in_production_storage = !is_empty,
            b"node" if in_production_storage => {
                if let Some(ref mut p) = current {
                    p.production_storage.push(ProductionStock {
                        fill_type: attr_str(e, "fillType"),
                        amount: attr_f64(e, "fillLevel"),
                    });
                }
            }
            _ => {}
        }
    }

    if let Some(p) = current.take() {
        placeables.push(p);
    }

    // A finished building keeps its <constructible> block in state "OLD" with no
    // inputs left, so only outstanding deliveries mark a real construction site.
    for p in &mut placeables {
        p.is_under_construction = p
            .construction
            .as_ref()
            .is_some_and(|c| c.inputs.iter().any(|m| m.amount_remaining > 0.0));
    }

    Ok(placeables)
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
    fn test_parse_placeables_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();
        assert_eq!(placeables.len(), 8);

        // First placeable: silo (completed building)
        let silo = &placeables[0];
        assert!(silo.display_name.contains("Silo"));
        assert_eq!(silo.farm_id, 1);
        assert!(!silo.is_under_construction);
        assert!(!silo.is_pre_placed);
    }

    #[test]
    fn test_parse_placeables_position_from_attribute() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        // Position is a space-separated attribute on <placeable>, not a <sentTranslation> child.
        let pos = placeables[0].position.as_ref().unwrap();
        assert!((pos.x - 200.5).abs() < 0.001);
        assert!((pos.y - 0.0).abs() < 0.001);
        assert!((pos.z - (-100.3)).abs() < 0.001);
    }

    #[test]
    fn test_parse_placeables_under_construction() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let under_construction: Vec<&Placeable> =
            placeables.iter().filter(|p| p.is_under_construction).collect();
        assert_eq!(under_construction.len(), 1);

        let barn = under_construction[0];
        let construction = barn.construction.as_ref().unwrap();
        assert_eq!(construction.state_name, "SETUP");
        assert_eq!(construction.inputs.len(), 2);
        assert_eq!(construction.inputs[0].fill_type, "WOODBEAM");
        assert!((construction.inputs[0].amount_remaining - 6000.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_placeables_finished_constructible_is_not_under_construction() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        // The water tower carries <constructible><state name="OLD"/> with no inputs left.
        let tower = placeables
            .iter()
            .find(|p| p.display_name.contains("Water Tower"))
            .unwrap();
        assert!(!tower.is_under_construction);
        assert_eq!(tower.construction.as_ref().unwrap().state_name, "OLD");
        assert!(tower.construction.as_ref().unwrap().inputs.is_empty());
    }

    #[test]
    fn test_parse_placeables_production_lines() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let mill = placeables
            .iter()
            .find(|p| !p.productions.is_empty())
            .unwrap();
        assert_eq!(mill.productions.len(), 2);
        assert_eq!(mill.productions[0].id, "flour");
        assert!(mill.productions[0].is_enabled);
        assert_eq!(mill.productions[1].id, "bread");
        assert!(!mill.productions[1].is_enabled);
    }

    #[test]
    fn test_parse_placeables_production_storage() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let mill = placeables
            .iter()
            .find(|p| !p.production_storage.is_empty())
            .unwrap();
        assert_eq!(mill.production_storage.len(), 2);
        assert_eq!(mill.production_storage[0].fill_type, "WHEAT");
        assert!((mill.production_storage[0].amount - 2500.0).abs() < 0.01);
        assert_eq!(mill.production_storage[1].fill_type, "FLOUR");
    }

    #[test]
    fn test_parse_placeables_silo_storage_is_not_production_storage() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        // The silo's <silo><storage><node> must not be mistaken for production stock.
        assert!(placeables[0].production_storage.is_empty());
        assert!(placeables[0].productions.is_empty());
    }

    #[test]
    fn test_parse_placeables_pre_placed_uses_attribute_not_farm_id() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let pre_placed: Vec<&Placeable> =
            placeables.iter().filter(|p| p.is_pre_placed).collect();
        assert_eq!(pre_placed.len(), 3);

        // A pre-placed building bought by a farm keeps farmId != 0.
        let farmhouse = pre_placed.iter().find(|p| p.farm_id == 1).unwrap();
        assert!(farmhouse.is_pre_placed);
    }

    #[test]
    fn test_parse_placeables_display_name_from_unique_id() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        // Pre-placed entries have no filename: the name comes from uniqueId,
        // stripped of the "preplaced_" prefix and the trailing hash.
        let farmhouse = placeables
            .iter()
            .find(|p| p.unique_id.starts_with("preplaced_farmHouse05"))
            .unwrap();
        assert!(farmhouse.filename.is_empty());
        assert_eq!(farmhouse.display_name, "Farm House 05");
    }

    #[test]
    fn test_parse_placeables_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_placeables");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_placeables(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
