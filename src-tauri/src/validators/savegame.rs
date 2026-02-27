use crate::models::common::LocalizedMessage;
use crate::models::SavegameData;

/// Validates cross-file consistency in a loaded savegame.
/// Returns a list of localized warning messages for any inconsistencies found.
pub fn validate_savegame(data: &SavegameData) -> Vec<LocalizedMessage> {
    let mut warnings = Vec::new();

    validate_money_consistency(data, &mut warnings);
    validate_vehicle_farms(data, &mut warnings);
    validate_attachment_references(data, &mut warnings);
    validate_field_farmland_links(data, &mut warnings);

    warnings
}

/// Check that career money matches farm 1 money.
fn validate_money_consistency(data: &SavegameData, warnings: &mut Vec<LocalizedMessage>) {
    if let Some(farm) = data.farms.iter().find(|f| f.farm_id == 1) {
        if (data.career.money - farm.money).abs() > 1.0 {
            warnings.push(
                LocalizedMessage::new("errors.validation.moneyInconsistency")
                    .with_param("careerMoney", format!("{:.2}", data.career.money))
                    .with_param("farmMoney", format!("{:.2}", farm.money)),
            );
        }
    }
}

/// Check that each vehicle's farm_id references an existing farm.
fn validate_vehicle_farms(data: &SavegameData, warnings: &mut Vec<LocalizedMessage>) {
    let farm_ids: Vec<u8> = data.farms.iter().map(|f| f.farm_id).collect();
    for vehicle in &data.vehicles {
        if vehicle.farm_id != 0 && !farm_ids.contains(&vehicle.farm_id) {
            warnings.push(
                LocalizedMessage::new("errors.validation.vehicleInvalidFarm")
                    .with_param("name", &vehicle.display_name)
                    .with_param("id", &vehicle.unique_id)
                    .with_param("farmId", vehicle.farm_id),
            );
        }
    }
}

/// Check that attached implement references point to existing vehicles.
fn validate_attachment_references(data: &SavegameData, warnings: &mut Vec<LocalizedMessage>) {
    let vehicle_ids: Vec<&str> = data.vehicles.iter().map(|v| v.unique_id.as_str()).collect();
    for vehicle in &data.vehicles {
        for implement in &vehicle.attached_implements {
            if !vehicle_ids.contains(&implement.attached_vehicle_unique_id.as_str()) {
                warnings.push(
                    LocalizedMessage::new("errors.validation.attachmentNotFound")
                        .with_param("name", &vehicle.display_name)
                        .with_param("id", &vehicle.unique_id)
                        .with_param("attachmentId", &implement.attached_vehicle_unique_id),
                );
            }
        }
    }
}

/// Check that each field has a matching farmland entry.
fn validate_field_farmland_links(data: &SavegameData, warnings: &mut Vec<LocalizedMessage>) {
    let farmland_ids: Vec<u32> = data.farmlands.iter().map(|fl| fl.id).collect();
    for field in &data.fields {
        if !farmland_ids.contains(&field.id) {
            warnings.push(
                LocalizedMessage::new("errors.validation.fieldNoFarmland")
                    .with_param("fieldId", field.id),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::career::CareerSavegame;
    use crate::models::farm::{Farm, FarmStatistics};
    use crate::models::field::{Farmland, Field};
    use crate::models::vehicle::{AttachedImplement, Vehicle, PropertyState};

    fn make_savegame_data() -> SavegameData {
        SavegameData {
            path: "/test".to_string(),
            career: CareerSavegame {
                savegame_name: "Test".to_string(),
                creation_date: "".to_string(),
                map_id: "".to_string(),
                map_title: "".to_string(),
                save_date: "".to_string(),
                economic_difficulty: "normal".to_string(),
                money: 100000.0,
                play_time: 0.0,
                growth_mode: 1,
                planned_days_per_period: 3,
                plowing_required: false,
                stones_enabled: false,
                weeds_enabled: false,
                lime_required: false,
                snow_enabled: false,
                fuel_usage: 1,
                traffic_enabled: true,
            },
            farms: vec![Farm {
                farm_id: 1,
                name: "Farm 1".to_string(),
                color: 1,
                loan: 0.0,
                money: 100000.0,
                players: vec![],
                statistics: FarmStatistics::default(),
                daily_finances: vec![],
            }],
            vehicles: vec![
                Vehicle {
                    unique_id: "1".to_string(),
                    filename: "tractor.xml".to_string(),
                    display_name: "Tractor".to_string(),
                    age: 10.0,
                    price: 50000.0,
                    farm_id: 1,
                    property_state: PropertyState::Owned,
                    operating_time: 100.0,
                    position: None,
                    rotation: None,
                    configurations: vec![],
                    fill_units: vec![],
                    attached_implements: vec![AttachedImplement {
                        joint_index: 0,
                        attached_vehicle_unique_id: "2".to_string(),
                        move_down: true,
                    }],
                },
                Vehicle {
                    unique_id: "2".to_string(),
                    filename: "trailer.xml".to_string(),
                    display_name: "Trailer".to_string(),
                    age: 5.0,
                    price: 20000.0,
                    farm_id: 1,
                    property_state: PropertyState::Owned,
                    operating_time: 50.0,
                    position: None,
                    rotation: None,
                    configurations: vec![],
                    fill_units: vec![],
                    attached_implements: vec![],
                },
            ],
            sales: vec![],
            fields: vec![Field {
                id: 1,
                planned_fruit: "WHEAT".to_string(),
                fruit_type: "WHEAT".to_string(),
                growth_state: 3,
                last_growth_state: 2,
                weed_state: 0,
                stone_level: 0,
                spray_level: 0,
                spray_type: "".to_string(),
                lime_level: 0,
                plow_level: 0,
                roller_level: 0,
                stubble_shred_level: 0,
                water_level: 0,
                ground_type: "".to_string(),
            }],
            farmlands: vec![Farmland { id: 1, farm_id: 1 }],
            placeables: vec![],
            missions: vec![],
            collectibles: vec![],
            contract_settings: None,
            environment: None,
            warnings: vec![],
        }
    }

    #[test]
    fn test_valid_savegame_no_warnings() {
        let data = make_savegame_data();
        let warnings = validate_savegame(&data);
        assert!(warnings.is_empty(), "Expected no warnings, got: {:?}", warnings);
    }

    #[test]
    fn test_money_inconsistency_warning() {
        let mut data = make_savegame_data();
        data.career.money = 200000.0; // Mismatch with farm money (100000) — diff > 1.0
        let warnings = validate_savegame(&data);
        assert!(warnings.iter().any(|w| w.code == "errors.validation.moneyInconsistency"));
    }

    #[test]
    fn test_vehicle_invalid_farm_warning() {
        let mut data = make_savegame_data();
        data.vehicles[0].farm_id = 99; // Non-existent farm
        let warnings = validate_savegame(&data);
        assert!(warnings.iter().any(|w| w.code == "errors.validation.vehicleInvalidFarm" && w.params.get("farmId").map(|v| v.as_str()) == Some("99")));
    }

    #[test]
    fn test_attachment_references_invalid_warning() {
        let mut data = make_savegame_data();
        data.vehicles[0].attached_implements[0].attached_vehicle_unique_id = "999".to_string();
        let warnings = validate_savegame(&data);
        assert!(warnings.iter().any(|w| w.code == "errors.validation.attachmentNotFound" && w.params.get("attachmentId").map(|v| v.as_str()) == Some("999")));
    }

    #[test]
    fn test_field_without_farmland_warning() {
        let mut data = make_savegame_data();
        data.farmlands.clear(); // Remove all farmlands
        let warnings = validate_savegame(&data);
        assert!(warnings.iter().any(|w| w.code == "errors.validation.fieldNoFarmland" && w.params.get("fieldId").map(|v| v.as_str()) == Some("1")));
    }
}
