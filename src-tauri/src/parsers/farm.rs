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

/// Read the text content of the current element from the reader.
fn read_text_content(reader: &mut Reader<&[u8]>) -> String {
    let mut text = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(ref t)) => {
                text = t.unescape().unwrap_or_default().to_string();
            }
            Ok(Event::End(_)) | Ok(Event::Eof) => break,
            _ => {}
        }
    }
    text
}

/// Parse child elements of <statistics> into FarmStatistics.
fn parse_statistics_children(reader: &mut Reader<&[u8]>) -> FarmStatistics {
    let mut stats = FarmStatistics::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let text = read_text_content(reader);
                let f = text.parse::<f64>().unwrap_or(0.0);
                let u = text.parse::<u32>().unwrap_or(0);
                match tag.as_str() {
                    // Distances
                    "traveledDistance" => stats.traveled_distance = f,
                    "tractorDistance" => stats.tractor_distance = f,
                    "carDistance" => stats.car_distance = f,
                    "truckDistance" => stats.truck_distance = f,
                    "horseDistance" => stats.horse_distance = f,
                    // Consumption
                    "fuelUsage" => stats.fuel_usage = f,
                    "seedUsage" => stats.seed_usage = f,
                    "sprayUsage" => stats.spray_usage = f,
                    // Hectares
                    "workedHectares" => stats.worked_hectares = f,
                    "cultivatedHectares" => stats.cultivated_hectares = f,
                    "sownHectares" => stats.sown_hectares = f,
                    "sprayedHectares" => stats.sprayed_hectares = f,
                    "threshedHectares" => stats.threshed_hectares = f,
                    "plowedHectares" => stats.plowed_hectares = f,
                    "harvestedGrapes" => stats.harvested_grapes = f,
                    "harvestedOlives" => stats.harvested_olives = f,
                    // Time spent
                    "workedTime" => stats.worked_time = f,
                    "cultivatedTime" => stats.cultivated_time = f,
                    "sownTime" => stats.sown_time = f,
                    "sprayedTime" => stats.sprayed_time = f,
                    "threshedTime" => stats.threshed_time = f,
                    "plowedTime" => stats.plowed_time = f,
                    // Counts
                    "baleCount" => stats.bale_count = u,
                    "wrappedBales" => stats.wrapped_bales = u,
                    "soldCottonBales" => stats.sold_cotton_bales = u,
                    "missionCount" => stats.mission_count = u,
                    "repairVehicleCount" => stats.repair_vehicle_count = u,
                    "repaintVehicleCount" => stats.repaint_vehicle_count = u,
                    // Animals
                    "breedCowsCount" => stats.breed_cows_count = u,
                    "breedSheepCount" => stats.breed_sheep_count = u,
                    "breedPigsCount" => stats.breed_pigs_count = u,
                    "breedChickenCount" => stats.breed_chicken_count = u,
                    "breedHorsesCount" => stats.breed_horses_count = u,
                    "breedGoatsCount" => stats.breed_goats_count = u,
                    "breedWaterBuffaloCount" => stats.breed_water_buffalo_count = u,
                    "petDogCount" => stats.pet_dog_count = u,
                    "horseJumpCount" => stats.horse_jump_count = u,
                    // Trees & wood
                    "plantedTreeCount" => stats.planted_tree_count = u,
                    "cutTreeCount" => stats.cut_tree_count = u,
                    "woodTonsSold" => stats.wood_tons_sold = f,
                    // Finance
                    "revenue" => stats.revenue = f,
                    "expenses" => stats.expenses = f,
                    // Play time
                    "playTime" => stats.play_time = f,
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "statistics" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    stats
}

/// Parse child elements of <stats day="X"> into DailyFinance.
fn parse_daily_finance_children(reader: &mut Reader<&[u8]>, day: u32) -> DailyFinance {
    let mut df = DailyFinance::default();
    df.day = day;
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let text = read_text_content(reader);
                let f = text.parse::<f64>().unwrap_or(0.0);
                match tag.as_str() {
                    "newVehiclesCost" => df.new_vehicles_cost = f,
                    "soldVehicles" => df.sold_vehicles = f,
                    "newAnimalsCost" => df.new_animals_cost = f,
                    "soldAnimals" => df.sold_animals = f,
                    "constructionCost" => df.construction_cost = f,
                    "soldBuildings" => df.sold_buildings = f,
                    "fieldPurchase" => df.field_purchase = f,
                    "fieldSelling" | "soldFields" => df.sold_fields = f,
                    "vehicleRunningCost" => df.vehicle_running_cost = f,
                    "vehicleLeasingCost" => df.vehicle_leasing_cost = f,
                    "propertyMaintenance" => df.property_maintenance = f,
                    "propertyIncome" => df.property_income = f,
                    "productionCosts" => df.production_costs = f,
                    "soldProducts" => df.sold_products = f,
                    "harvestIncome" => df.harvest_income = f,
                    "missionIncome" => df.mission_income = f,
                    "wagePayment" => df.wage_payment = f,
                    "loanInterest" => df.loan_interest = f,
                    "other" | "otherIncome" => df.other_income = f,
                    "otherExpenses" => df.other_expenses = f,
                    _ => {} // Skip unknown fields (newHandtoolsCost, soldWood, purchaseFuel, etc.)
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "stats" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    df
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
                    "statistics" => {
                        // FS25 uses child elements: <statistics><traveledDistance>123</traveledDistance>...</statistics>
                        if let Some(ref mut farm) = current_farm {
                            farm.statistics = parse_statistics_children(&mut reader);
                        }
                    }
                    "stats" if in_finances => {
                        // FS25 uses <stats day="X"><newVehiclesCost>...</newVehiclesCost>...</stats>
                        if let Some(ref mut farm) = current_farm {
                            let day = attr_u32(e, "day");
                            farm.daily_finances.push(parse_daily_finance_children(&mut reader, day));
                        }
                    }
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
        assert!(farm.statistics.traveled_distance > 0.0);
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
