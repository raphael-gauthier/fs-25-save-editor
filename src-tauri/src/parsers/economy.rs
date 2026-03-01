use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::economy::{Economy, FillTypePrice, GreatDemand, PeriodPrice};

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

fn attr_u64_opt(e: &quick_xml::events::BytesStart, key: &str) -> Option<u64> {
    let s = attr_str(e, key);
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

fn attr_bool(e: &quick_xml::events::BytesStart, key: &str) -> bool {
    attr_str(e, key) == "true"
}

pub fn parse_economy(path: &Path) -> Result<Economy, AppError> {
    let xml_path = path.join("economy.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut great_demands: Vec<GreatDemand> = Vec::new();
    let mut fill_types: Vec<FillTypePrice> = Vec::new();

    let mut in_great_demands = false;
    let mut in_fill_types = false;
    let mut demand_index: usize = 0;

    // State for current fillType being parsed
    let mut current_fill_type: Option<FillTypePrice> = None;
    let mut current_period_name: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "greatDemands" => {
                        in_great_demands = true;
                        demand_index = 0;
                    }
                    "greatDemand" if in_great_demands => {
                        let unique_id = attr_str(e, "uniqueId");
                        if !unique_id.is_empty() {
                            great_demands.push(GreatDemand {
                                index: demand_index,
                                unique_id,
                                fill_type_name: attr_str(e, "fillTypeName"),
                                demand_multiplier: attr_f64(e, "demandMultiplier"),
                                demand_start_day: attr_u32(e, "demandStartDay"),
                                demand_start_hour: attr_u32(e, "demandStartHour"),
                                demand_duration: attr_u32(e, "demandDuration"),
                                is_running: attr_bool(e, "isRunning"),
                                is_valid: attr_bool(e, "isValid"),
                            });
                        }
                        demand_index += 1;
                    }
                    "fillTypes" => {
                        in_fill_types = true;
                    }
                    "fillType" if in_fill_types => {
                        let ft = attr_str(e, "fillType");
                        let total = attr_u64_opt(e, "totalAmount");
                        current_fill_type = Some(FillTypePrice {
                            fill_type: ft,
                            total_amount: total,
                            price_history: Vec::new(),
                        });
                    }
                    "period" if current_fill_type.is_some() => {
                        current_period_name = Some(attr_str(e, "period"));
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "greatDemand" if in_great_demands => {
                        let unique_id = attr_str(e, "uniqueId");
                        if !unique_id.is_empty() {
                            great_demands.push(GreatDemand {
                                index: demand_index,
                                unique_id,
                                fill_type_name: attr_str(e, "fillTypeName"),
                                demand_multiplier: attr_f64(e, "demandMultiplier"),
                                demand_start_day: attr_u32(e, "demandStartDay"),
                                demand_start_hour: attr_u32(e, "demandStartHour"),
                                demand_duration: attr_u32(e, "demandDuration"),
                                is_running: attr_bool(e, "isRunning"),
                                is_valid: attr_bool(e, "isValid"),
                            });
                        }
                        // Empty slots (no attributes) are just skipped for parsing
                        demand_index += 1;
                    }
                    "fillType" if in_fill_types => {
                        // Self-closing fillType (e.g., <fillType fillType="UNKNOWN"/>)
                        // Skip: no history data
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref period_name) = current_period_name {
                    if let Some(ref mut ft) = current_fill_type {
                        let text = e.unescape().unwrap_or_default().to_string();
                        if let Ok(price) = text.trim().parse::<u32>() {
                            ft.price_history.push(PeriodPrice {
                                period: period_name.clone(),
                                price,
                            });
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "greatDemands" => {
                        in_great_demands = false;
                    }
                    "fillTypes" => {
                        in_fill_types = false;
                    }
                    "fillType" => {
                        if let Some(ft) = current_fill_type.take() {
                            if !ft.price_history.is_empty() || ft.total_amount.is_some() {
                                fill_types.push(ft);
                            }
                        }
                    }
                    "period" => {
                        current_period_name = None;
                    }
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

    Ok(Economy {
        great_demands,
        fill_types,
    })
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
    fn test_parse_economy_great_demands() {
        let path = fixtures_path().join("savegame_complete");
        let economy = parse_economy(&path).unwrap();

        assert_eq!(economy.great_demands.len(), 2);

        let seeds = &economy.great_demands[0];
        assert_eq!(seeds.index, 0);
        assert_eq!(seeds.unique_id, "sellingStationTrain01");
        assert_eq!(seeds.fill_type_name, "SEEDS");
        assert!((seeds.demand_multiplier - 1.1).abs() < 0.001);
        assert_eq!(seeds.demand_start_day, 55);
        assert_eq!(seeds.demand_start_hour, 12);
        assert_eq!(seeds.demand_duration, 18);
        assert!(!seeds.is_running);
        assert!(seeds.is_valid);

        let flour = &economy.great_demands[1];
        assert_eq!(flour.index, 2); // index 1 is empty slot
        assert_eq!(flour.unique_id, "sellingStationGrain02");
        assert_eq!(flour.fill_type_name, "FLOUR");
        assert!((flour.demand_multiplier - 1.5).abs() < 0.001);
        assert!(flour.is_running);
    }

    #[test]
    fn test_parse_economy_fill_types() {
        let path = fixtures_path().join("savegame_complete");
        let economy = parse_economy(&path).unwrap();

        // UNKNOWN has no history/totalAmount, so it should be filtered out
        assert_eq!(economy.fill_types.len(), 2);

        let wheat = &economy.fill_types[0];
        assert_eq!(wheat.fill_type, "WHEAT");
        assert_eq!(wheat.total_amount, Some(44762));
        assert_eq!(wheat.price_history.len(), 12);
        assert_eq!(wheat.price_history[0].period, "EARLY_SPRING");
        assert_eq!(wheat.price_history[0].price, 349);
        assert_eq!(wheat.price_history[5].period, "LATE_SUMMER");
        assert_eq!(wheat.price_history[5].price, 370);

        let barley = &economy.fill_types[1];
        assert_eq!(barley.fill_type, "BARLEY");
        assert_eq!(barley.total_amount, Some(12500));
        assert_eq!(barley.price_history.len(), 12);
    }

    #[test]
    fn test_parse_economy_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_economy");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_economy(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
