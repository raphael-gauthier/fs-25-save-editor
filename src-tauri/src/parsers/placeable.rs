use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::common::Position;
use crate::models::placeable::{
    placeable_display_name, ConstructionMaterial, ConstructionStep, Placeable, ProductionStock,
};

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_f64(e: &quick_xml::events::BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

fn attr_u8(e: &quick_xml::events::BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn attr_u32(e: &quick_xml::events::BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

pub fn parse_placeables(path: &Path) -> Result<Vec<Placeable>, AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut placeables: Vec<Placeable> = Vec::new();

    // State tracking
    let mut placeable_index: usize = 0;
    let mut in_placeable = false;
    let mut current: Option<PlaceableBuilder> = None;
    let mut current_component_index: Option<String> = None;
    let mut in_construction = false;
    let mut current_step: Option<ConstructionStep> = None;
    let mut in_production_point = false;
    let mut in_production_input = false;
    let mut in_production_output = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "placeable" if !in_placeable => {
                        in_placeable = true;
                        let filename = attr_str(e, "filename");
                        let display_name = placeable_display_name(&filename);
                        let farm_id = attr_u8(e, "farmId");
                        current = Some(PlaceableBuilder {
                            index: placeable_index,
                            filename,
                            display_name,
                            farm_id,
                            price: attr_f64(e, "price"),
                            age: attr_f64(e, "age"),
                            position: None,
                            is_pre_placed: farm_id == 0,
                            is_under_construction: false,
                            construction_steps: Vec::new(),
                            production_inputs: Vec::new(),
                            production_outputs: Vec::new(),
                        });
                        placeable_index += 1;
                    }
                    "component" if in_placeable => {
                        current_component_index = Some(attr_str(e, "index"));
                    }
                    "constructionPlaceable" if in_placeable => {
                        in_construction = true;
                        if let Some(ref mut pb) = current {
                            pb.is_under_construction = true;
                        }
                    }
                    "step" if in_construction => {
                        current_step = Some(ConstructionStep {
                            step_index: attr_u32(e, "index"),
                            materials: Vec::new(),
                        });
                    }
                    "productionPoint" if in_placeable => {
                        in_production_point = true;
                    }
                    "input" if in_production_point => {
                        in_production_input = true;
                    }
                    "output" if in_production_point => {
                        in_production_output = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some(ref mut pb) = current {
                    match tag.as_str() {
                        "sentTranslation"
                            if current_component_index.as_deref() == Some("1") =>
                        {
                            let x = attr_f64(e, "x");
                            let y = attr_f64(e, "y");
                            let z = attr_f64(e, "z");
                            if x != 0.0 || y != 0.0 || z != 0.0 {
                                pb.position = Some(Position { x, y, z });
                            }
                        }
                        "material" if current_step.is_some() => {
                            if let Some(ref mut step) = current_step {
                                let remaining = attr_f64(e, "amountRemaining");
                                let total = attr_f64(e, "amount");
                                step.materials.push(ConstructionMaterial {
                                    fill_type: attr_str(e, "fillType"),
                                    amount_remaining: remaining,
                                    amount_total: total,
                                });
                            }
                        }
                        "storage" if in_production_input => {
                            let fill_type = attr_str(e, "fillType");
                            if !fill_type.is_empty() {
                                pb.production_inputs.push(ProductionStock {
                                    fill_type,
                                    amount: attr_f64(e, "fillLevel"),
                                    capacity: attr_f64(e, "capacity"),
                                });
                            }
                        }
                        "storage" if in_production_output => {
                            let fill_type = attr_str(e, "fillType");
                            if !fill_type.is_empty() {
                                pb.production_outputs.push(ProductionStock {
                                    fill_type,
                                    amount: attr_f64(e, "fillLevel"),
                                    capacity: attr_f64(e, "capacity"),
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "placeable" if in_placeable => {
                        in_placeable = false;
                        if let Some(pb) = current.take() {
                            // Check if construction is actually complete (all remaining == 0)
                            let mut p = pb.build();
                            if p.is_under_construction {
                                let all_done = p.construction_steps.iter().all(|s| {
                                    s.materials.iter().all(|m| m.amount_remaining <= 0.0)
                                });
                                if all_done {
                                    p.is_under_construction = false;
                                }
                            }
                            placeables.push(p);
                        }
                    }
                    "component" => current_component_index = None,
                    "constructionPlaceable" => in_construction = false,
                    "step" if in_construction => {
                        if let Some(step) = current_step.take() {
                            if let Some(ref mut pb) = current {
                                pb.construction_steps.push(step);
                            }
                        }
                    }
                    "productionPoint" => {
                        in_production_point = false;
                        in_production_input = false;
                        in_production_output = false;
                    }
                    "input" if in_production_point => in_production_input = false,
                    "output" if in_production_point => in_production_output = false,
                    _ => {}
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

    Ok(placeables)
}

struct PlaceableBuilder {
    index: usize,
    filename: String,
    display_name: String,
    farm_id: u8,
    price: f64,
    age: f64,
    position: Option<Position>,
    is_pre_placed: bool,
    is_under_construction: bool,
    construction_steps: Vec<ConstructionStep>,
    production_inputs: Vec<ProductionStock>,
    production_outputs: Vec<ProductionStock>,
}

impl PlaceableBuilder {
    fn build(self) -> Placeable {
        Placeable {
            index: self.index,
            filename: self.filename,
            display_name: self.display_name,
            farm_id: self.farm_id,
            price: self.price,
            age: self.age,
            position: self.position,
            is_pre_placed: self.is_pre_placed,
            is_under_construction: self.is_under_construction,
            construction_steps: self.construction_steps,
            production_inputs: self.production_inputs,
            production_outputs: self.production_outputs,
        }
    }
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
        assert_eq!(placeables.len(), 4);

        // First placeable: silo (completed building)
        let silo = &placeables[0];
        assert!(silo.display_name.contains("Silo"));
        assert_eq!(silo.farm_id, 1);
        assert!(!silo.is_under_construction);
        assert!(!silo.is_pre_placed);
        assert!(silo.position.is_some());
    }

    #[test]
    fn test_parse_placeables_under_construction() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let under_construction: Vec<&Placeable> =
            placeables.iter().filter(|p| p.is_under_construction).collect();
        assert_eq!(under_construction.len(), 1);

        let barn = under_construction[0];
        assert!(!barn.construction_steps.is_empty());
        let step = &barn.construction_steps[0];
        assert!(!step.materials.is_empty());
        assert!(step.materials[0].amount_remaining > 0.0);
    }

    #[test]
    fn test_parse_placeables_production() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let production: Vec<&Placeable> = placeables
            .iter()
            .filter(|p| !p.production_inputs.is_empty() || !p.production_outputs.is_empty())
            .collect();
        assert_eq!(production.len(), 1);

        let mill = production[0];
        assert!(!mill.production_inputs.is_empty());
        assert!(!mill.production_outputs.is_empty());
        assert!(mill.production_inputs[0].capacity > 0.0);
    }

    #[test]
    fn test_parse_placeables_pre_placed() {
        let path = fixtures_path().join("savegame_complete");
        let placeables = parse_placeables(&path).unwrap();

        let pre_placed: Vec<&Placeable> =
            placeables.iter().filter(|p| p.is_pre_placed).collect();
        assert_eq!(pre_placed.len(), 1);
        assert_eq!(pre_placed[0].farm_id, 0);
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
