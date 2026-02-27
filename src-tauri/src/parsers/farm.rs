use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::farm::{DailyFinance, Farm, FarmPlayer, FarmStatistics};

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

fn attr_u32(e: &quick_xml::events::BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn attr_u8(e: &quick_xml::events::BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn attr_bool(e: &quick_xml::events::BytesStart, key: &str) -> bool {
    attr_str(e, key) == "true"
}

/// Parse farms.xml and return the list of farms with their players, stats and finances.
pub fn parse_farms(path: &Path) -> Result<Vec<Farm>, AppError> {
    let xml_path = path.join("farms.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut farms: Vec<Farm> = Vec::new();

    let mut current_farm: Option<Farm> = None;
    let mut in_players = false;
    let mut in_finances = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "farm" => {
                        current_farm = Some(Farm {
                            farm_id: attr_u8(e, "farmId"),
                            name: attr_str(e, "name"),
                            color: attr_u8(e, "color"),
                            loan: attr_f64(e, "loan"),
                            money: attr_f64(e, "money"),
                            players: Vec::new(),
                            statistics: FarmStatistics::default(),
                            daily_finances: Vec::new(),
                        });
                    }
                    "players" => in_players = true,
                    "finances" => in_finances = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if let Some(ref mut farm) = current_farm {
                    match tag.as_str() {
                        "player" if in_players => {
                            farm.players.push(FarmPlayer {
                                unique_user_id: attr_str(e, "uniqueUserId"),
                                farm_manager: attr_bool(e, "farmManager"),
                                last_nickname: attr_str(e, "lastNickname"),
                                time_last_connected: attr_str(e, "timeLastConnected"),
                                buy_vehicle: attr_bool(e, "buyVehicle"),
                                sell_vehicle: attr_bool(e, "sellVehicle"),
                                buy_placeable: attr_bool(e, "buyPlaceable"),
                                sell_placeable: attr_bool(e, "sellPlaceable"),
                                manage_contracts: attr_bool(e, "manageContracts"),
                                trade_animals: attr_bool(e, "tradeAnimals"),
                                create_fields: attr_bool(e, "createFields"),
                                landscaping: attr_bool(e, "landscaping"),
                                hire_assistant: attr_bool(e, "hireAssistant"),
                                reset_vehicle: attr_bool(e, "resetVehicle"),
                                manage_productions: attr_bool(e, "manageProductions"),
                                cut_trees: attr_bool(e, "cutTrees"),
                                manage_rights: attr_bool(e, "manageRights"),
                                transfer_money: attr_bool(e, "transferMoney"),
                                update_farm: attr_bool(e, "updateFarm"),
                                manage_contracting: attr_bool(e, "manageContracting"),
                            });
                        }
                        "statistics" => {
                            farm.statistics = FarmStatistics {
                                traveled_distance: attr_f64(e, "traveledDistance"),
                                fuel_usage: attr_f64(e, "fuelUsage"),
                                seed_usage: attr_f64(e, "seedUsage"),
                                spray_usage: attr_f64(e, "sprayUsage"),
                                worked_hectares: attr_f64(e, "workedHectares"),
                                cultivated_hectares: attr_f64(e, "cultivatedHectares"),
                                sown_hectares: attr_f64(e, "sownHectares"),
                                sprayed_hectares: attr_f64(e, "sprayedHectares"),
                                threshed_hectares: attr_f64(e, "threshedHectares"),
                                plowed_hectares: attr_f64(e, "plowedHectares"),
                                bale_count: attr_u32(e, "baleCount"),
                                mission_count: attr_u32(e, "missionCount"),
                                play_time: attr_f64(e, "playTime"),
                                revenue: attr_f64(e, "revenue"),
                                expenses: attr_f64(e, "expenses"),
                            };
                        }
                        "dailyFinance" if in_finances => {
                            farm.daily_finances.push(DailyFinance {
                                day: attr_u32(e, "day"),
                                new_vehicles_cost: attr_f64(e, "newVehiclesCost"),
                                sold_vehicles: attr_f64(e, "soldVehicles"),
                                new_animals_cost: attr_f64(e, "newAnimalsCost"),
                                sold_animals: attr_f64(e, "soldAnimals"),
                                construction_cost: attr_f64(e, "constructionCost"),
                                sold_buildings: attr_f64(e, "soldBuildings"),
                                field_purchase: attr_f64(e, "fieldPurchase"),
                                sold_fields: attr_f64(e, "soldFields"),
                                vehicle_running_cost: attr_f64(e, "vehicleRunningCost"),
                                vehicle_leasing_cost: attr_f64(e, "vehicleLeasingCost"),
                                property_maintenance: attr_f64(e, "propertyMaintenance"),
                                property_income: attr_f64(e, "propertyIncome"),
                                production_costs: attr_f64(e, "productionCosts"),
                                sold_products: attr_f64(e, "soldProducts"),
                                harvest_income: attr_f64(e, "harvestIncome"),
                                mission_income: attr_f64(e, "missionIncome"),
                                wage_payment: attr_f64(e, "wagePayment"),
                                loan_interest: attr_f64(e, "loanInterest"),
                                other_income: attr_f64(e, "otherIncome"),
                                other_expenses: attr_f64(e, "otherExpenses"),
                            });
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "farm" => {
                        if let Some(farm) = current_farm.take() {
                            farms.push(farm);
                        }
                    }
                    "players" => in_players = false,
                    "finances" => in_finances = false,
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

    Ok(farms)
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
    fn test_parse_farms_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let farms = parse_farms(&path).unwrap();
        assert_eq!(farms.len(), 1);
        let farm = &farms[0];
        assert_eq!(farm.farm_id, 1);
        assert_eq!(farm.name, "My Farm");
        assert!((farm.money - 1_000_000.0).abs() < 0.01);
        assert!((farm.loan - 50000.0).abs() < 0.01);
        assert_eq!(farm.players.len(), 1);
        assert!(farm.players[0].farm_manager);
        assert!(farm.statistics.revenue > 0.0);
        assert!(!farm.daily_finances.is_empty());
    }

    #[test]
    fn test_parse_farms_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_farms");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_farms(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
