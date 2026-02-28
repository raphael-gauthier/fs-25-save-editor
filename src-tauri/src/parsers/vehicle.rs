use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::common::{Position, Rotation};
use crate::models::vehicle::{
    vehicle_display_name, AttachedImplement, FillUnit, Vehicle, VehicleConfiguration, PropertyState,
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

/// Parse vehicles.xml and return the list of all vehicles.
/// Uses manual event-based parsing due to the complex component-based XML structure.
pub fn parse_vehicles(path: &Path) -> Result<Vec<Vehicle>, AppError> {
    let xml_path = path.join("vehicles.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut vehicles: Vec<Vehicle> = Vec::new();

    // State tracking
    let mut in_vehicle = false;
    let mut current_vehicle: Option<VehicleBuilder> = None;
    let mut current_component_index: Option<String> = None;
    let mut in_fill_unit = false;
    let mut in_attached_implements = false;
    let mut in_configurations = false;
    let mut in_wearable = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "vehicle" if !in_vehicle => {
                        in_vehicle = true;
                        let filename = attr_str(e, "filename");
                        let display_name = vehicle_display_name(&filename);
                        current_vehicle = Some(VehicleBuilder {
                            unique_id: attr_str(e, "uniqueId"),
                            filename,
                            display_name,
                            age: attr_f64(e, "age"),
                            price: attr_f64(e, "price"),
                            farm_id: attr_u8(e, "farmId"),
                            property_state: PropertyState::from_str(
                                &attr_str(e, "propertyState"),
                            ),
                            operating_time: attr_f64(e, "operatingTime") / 60.0,
                            damage: 0.0,
                            wear: 0.0,
                            position: None,
                            rotation: None,
                            configurations: Vec::new(),
                            fill_units: Vec::new(),
                            attached_implements: Vec::new(),
                        });
                    }
                    "component" if in_vehicle => {
                        current_component_index = Some(attr_str(e, "index"));
                    }
                    "fillUnit" if in_vehicle => {
                        in_fill_unit = true;
                    }
                    "attacherJoints" if in_vehicle => {
                        in_attached_implements = true;
                    }
                    "boughtConfigurations" if in_vehicle => {
                        in_configurations = true;
                    }
                    "wearable" if in_vehicle => {
                        in_wearable = true;
                        if let Some(ref mut vb) = current_vehicle {
                            vb.damage = attr_f64(e, "damage");
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some(ref mut vb) = current_vehicle {
                    match tag.as_str() {
                        "sentTranslation" if current_component_index.as_deref() == Some("1") => {
                            let x = attr_f64(e, "x");
                            let y = attr_f64(e, "y");
                            let z = attr_f64(e, "z");
                            if x != 0.0 || y != 0.0 || z != 0.0 {
                                vb.position = Some(Position { x, y, z });
                            }
                        }
                        "sentRotation" if current_component_index.as_deref() == Some("1") => {
                            let x = attr_f64(e, "x");
                            let y = attr_f64(e, "y");
                            let z = attr_f64(e, "z");
                            vb.rotation = Some(Rotation { x, y, z });
                        }
                        "unit" if in_fill_unit => {
                            let fill_type = attr_str(e, "fillType");
                            if !fill_type.is_empty() && fill_type != "UNKNOWN" {
                                vb.fill_units.push(FillUnit {
                                    index: attr_u32(e, "index"),
                                    fill_type,
                                    fill_level: attr_f64(e, "fillLevel"),
                                    capacity: {
                                        let c = attr_f64(e, "capacity");
                                        if c > 0.0 { Some(c) } else { None }
                                    },
                                });
                            }
                        }
                        "attachedVehicle" if in_attached_implements => {
                            let uid = attr_str(e, "attachedVehicleUniqueId");
                            if !uid.is_empty() {
                                vb.attached_implements.push(AttachedImplement {
                                    joint_index: attr_u32(e, "jointIndex"),
                                    attached_vehicle_unique_id: uid,
                                    move_down: attr_str(e, "moveDown") == "true",
                                });
                            }
                        }
                        "boughtConfiguration" if in_configurations => {
                            vb.configurations.push(VehicleConfiguration {
                                name: attr_str(e, "name"),
                                id: attr_str(e, "id"),
                            });
                        }
                        "wearNode" if in_wearable => {
                            vb.wear = attr_f64(e, "amount");
                        }
                        "wearable" => {
                            // Self-closing <wearable .../> (fallback for test fixtures)
                            vb.damage = attr_f64(e, "damage");
                            vb.wear = attr_f64(e, "wear");
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "vehicle" if in_vehicle => {
                        in_vehicle = false;
                        if let Some(vb) = current_vehicle.take() {
                            vehicles.push(vb.build());
                        }
                    }
                    "component" => current_component_index = None,
                    "fillUnit" => in_fill_unit = false,
                    "attacherJoints" => in_attached_implements = false,
                    "boughtConfigurations" => in_configurations = false,
                    "wearable" => in_wearable = false,
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

    Ok(vehicles)
}

struct VehicleBuilder {
    unique_id: String,
    filename: String,
    display_name: String,
    age: f64,
    price: f64,
    farm_id: u8,
    property_state: PropertyState,
    operating_time: f64,
    damage: f64,
    wear: f64,
    position: Option<Position>,
    rotation: Option<Rotation>,
    configurations: Vec<VehicleConfiguration>,
    fill_units: Vec<FillUnit>,
    attached_implements: Vec<AttachedImplement>,
}

impl VehicleBuilder {
    fn build(self) -> Vehicle {
        Vehicle {
            unique_id: self.unique_id,
            filename: self.filename,
            display_name: self.display_name,
            age: self.age,
            price: self.price,
            farm_id: self.farm_id,
            property_state: self.property_state,
            operating_time: self.operating_time,
            damage: self.damage,
            wear: self.wear,
            position: self.position,
            rotation: self.rotation,
            configurations: self.configurations,
            fill_units: self.fill_units,
            attached_implements: self.attached_implements,
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
    fn test_parse_vehicles_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let vehicles = parse_vehicles(&path).unwrap();
        assert_eq!(vehicles.len(), 3);

        let tractor = vehicles.iter().find(|v| v.unique_id == "vehicle0001").unwrap();
        assert_eq!(tractor.display_name, "Fendt 942 Vario");
        assert!((tractor.price - 348000.0).abs() < 0.01);
        assert_eq!(tractor.property_state, PropertyState::Owned);
        assert!(tractor.position.is_some());
        assert!(!tractor.fill_units.is_empty());
        // Wearable data
        assert!((tractor.damage - 0.05).abs() < 0.001);
        assert!((tractor.wear - 0.12).abs() < 0.001);
    }

    #[test]
    fn test_parse_vehicles_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_vehicles");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_vehicles(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_vehicle_display_name() {
        assert_eq!(
            vehicle_display_name("data/vehicles/fendt/fendt942Vario/fendt942Vario.xml"),
            "Fendt 942 Vario"
        );
        assert_eq!(
            vehicle_display_name("data/vehicles/johnDeere/johnDeere8R/johnDeere8R.xml"),
            "John Deere 8 R"
        );
    }
}
