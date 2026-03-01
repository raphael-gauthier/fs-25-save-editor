use std::collections::{HashMap, HashSet};
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{EconomyChanges, GreatDemandAddition, GreatDemandChange};

pub fn write_economy_changes(
    path: &Path,
    changes: &EconomyChanges,
) -> Result<(), AppError> {
    let xml_path = path.join("economy.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: HashMap<usize, &GreatDemandChange> = changes
        .great_demand_changes
        .as_ref()
        .map(|v| v.iter().map(|c| (c.index, c)).collect())
        .unwrap_or_default();

    let deletions: HashSet<usize> = changes
        .great_demand_deletions
        .as_ref()
        .map(|v| v.iter().copied().collect())
        .unwrap_or_default();

    let additions: &[GreatDemandAddition] = changes
        .great_demand_additions
        .as_deref()
        .unwrap_or_default();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut demand_index: usize = 0;
    let mut in_great_demands = false;
    let mut skip_until_end_great_demand = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "greatDemands" => {
                        in_great_demands = true;
                        demand_index = 0;
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                    "greatDemand" if in_great_demands => {
                        if deletions.contains(&demand_index) {
                            // Delete: skip this element entirely, replace with empty slot
                            skip_until_end_great_demand = true;
                            let empty = BytesStart::new("greatDemand");
                            write_event(&mut writer, &xml_path, Event::Empty(empty))?;
                        } else if let Some(change) = change_map.get(&demand_index) {
                            let elem = patch_great_demand(e, change);
                            write_event(&mut writer, &xml_path, Event::Start(elem))?;
                        } else {
                            write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        }
                        demand_index += 1;
                    }
                    _ => {
                        if !skip_until_end_great_demand {
                            write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "greatDemand" && in_great_demands {
                    if deletions.contains(&demand_index) {
                        // Delete: replace with empty slot (no attributes)
                        let empty = BytesStart::new("greatDemand");
                        write_event(&mut writer, &xml_path, Event::Empty(empty))?;
                    } else if let Some(change) = change_map.get(&demand_index) {
                        // Modifying an empty slot — should not normally happen
                        let elem = patch_great_demand(e, change);
                        write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                    } else {
                        write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                    }
                    demand_index += 1;
                } else if !skip_until_end_great_demand {
                    write_event(&mut writer, &xml_path, Event::Empty(e.clone().into_owned()))?;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "greatDemand" && skip_until_end_great_demand {
                    skip_until_end_great_demand = false;
                    // Already wrote empty slot, skip closing tag
                } else if tag == "greatDemands" {
                    // Before closing, append additions
                    for addition in additions {
                        let elem = create_great_demand(addition);
                        write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                    }
                    in_great_demands = false;
                    write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                } else {
                    write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if !skip_until_end_great_demand {
                    write_event(&mut writer, &xml_path, event.into_owned())?;
                }
            }
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

fn patch_great_demand(e: &BytesStart, change: &GreatDemandChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("greatDemand");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "fillTypeName" if change.fill_type_name.is_some() => {
                elem.push_attribute(("fillTypeName", change.fill_type_name.as_ref().unwrap().as_str()));
            }
            "demandMultiplier" if change.demand_multiplier.is_some() => {
                elem.push_attribute(("demandMultiplier", format!("{:.6}", change.demand_multiplier.unwrap()).as_str()));
            }
            "demandStartDay" if change.demand_start_day.is_some() => {
                elem.push_attribute(("demandStartDay", change.demand_start_day.unwrap().to_string().as_str()));
            }
            "demandStartHour" if change.demand_start_hour.is_some() => {
                elem.push_attribute(("demandStartHour", change.demand_start_hour.unwrap().to_string().as_str()));
            }
            "demandDuration" if change.demand_duration.is_some() => {
                elem.push_attribute(("demandDuration", change.demand_duration.unwrap().to_string().as_str()));
            }
            "isRunning" if change.is_running.is_some() => {
                elem.push_attribute(("isRunning", if change.is_running.unwrap() { "true" } else { "false" }));
            }
            "isValid" if change.is_valid.is_some() => {
                elem.push_attribute(("isValid", if change.is_valid.unwrap() { "true" } else { "false" }));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    String::from_utf8_lossy(&attr.value).as_ref(),
                ));
            }
        }
    }
    elem
}

fn create_great_demand(addition: &GreatDemandAddition) -> BytesStart<'static> {
    let mut elem = BytesStart::new("greatDemand");
    elem.push_attribute(("uniqueId", addition.unique_id.as_str()));
    elem.push_attribute(("fillTypeName", addition.fill_type_name.as_str()));
    elem.push_attribute(("demandMultiplier", format!("{:.6}", addition.demand_multiplier).as_str()));
    elem.push_attribute(("demandStartDay", addition.demand_start_day.to_string().as_str()));
    elem.push_attribute(("demandStartHour", addition.demand_start_hour.to_string().as_str()));
    elem.push_attribute(("demandDuration", addition.demand_duration.to_string().as_str()));
    elem.push_attribute(("isRunning", "false"));
    elem.push_attribute(("isValid", "true"));
    elem
}

fn write_event(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer.write_event(event).map_err(|e| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::changes::{EconomyChanges, GreatDemandAddition, GreatDemandChange};
    use crate::parsers::economy::parse_economy;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_we_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
            }
        }
        dst
    }

    #[test]
    fn test_write_economy_modify_demand() {
        let save = setup_fixture("modify_demand");
        let before = parse_economy(&save).unwrap();
        assert_eq!(before.great_demands.len(), 2);

        // Modify the first demand's multiplier
        let changes = EconomyChanges {
            great_demand_changes: Some(vec![GreatDemandChange {
                index: 0,
                fill_type_name: None,
                demand_multiplier: Some(2.0),
                demand_start_day: None,
                demand_start_hour: None,
                demand_duration: None,
                is_running: None,
                is_valid: None,
            }]),
            great_demand_additions: None,
            great_demand_deletions: None,
        };
        write_economy_changes(&save, &changes).unwrap();

        let after = parse_economy(&save).unwrap();
        assert_eq!(after.great_demands.len(), 2);
        assert!((after.great_demands[0].demand_multiplier - 2.0).abs() < 0.001);
        // Other demand unchanged
        assert!((after.great_demands[1].demand_multiplier - 1.5).abs() < 0.001);

        // Fill types should be untouched
        assert_eq!(after.fill_types.len(), before.fill_types.len());

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_economy_add_demand() {
        let save = setup_fixture("add_demand");
        let before = parse_economy(&save).unwrap();

        let changes = EconomyChanges {
            great_demand_changes: None,
            great_demand_additions: Some(vec![GreatDemandAddition {
                unique_id: "sellingStationNew01".to_string(),
                fill_type_name: "WHEAT".to_string(),
                demand_multiplier: 1.25,
                demand_start_day: 10,
                demand_start_hour: 6,
                demand_duration: 48,
            }]),
            great_demand_deletions: None,
        };
        write_economy_changes(&save, &changes).unwrap();

        let after = parse_economy(&save).unwrap();
        assert_eq!(after.great_demands.len(), before.great_demands.len() + 1);
        let added = after.great_demands.last().unwrap();
        assert_eq!(added.unique_id, "sellingStationNew01");
        assert_eq!(added.fill_type_name, "WHEAT");
        assert!((added.demand_multiplier - 1.25).abs() < 0.001);

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_economy_delete_demand() {
        let save = setup_fixture("delete_demand");
        let before = parse_economy(&save).unwrap();
        assert_eq!(before.great_demands.len(), 2);

        // Delete the first demand (index 0)
        let changes = EconomyChanges {
            great_demand_changes: None,
            great_demand_additions: None,
            great_demand_deletions: Some(vec![0]),
        };
        write_economy_changes(&save, &changes).unwrap();

        let after = parse_economy(&save).unwrap();
        // First demand deleted, only second remains
        assert_eq!(after.great_demands.len(), 1);
        assert_eq!(after.great_demands[0].fill_type_name, "FLOUR");

        // Verify the XML still has the empty slots preserved
        let xml_content = std::fs::read_to_string(save.join("economy.xml")).unwrap();
        assert!(xml_content.contains("<greatDemand/>"));

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_economy_roundtrip() {
        let save = setup_fixture("roundtrip_eco");
        let before = parse_economy(&save).unwrap();

        let changes = EconomyChanges {
            great_demand_changes: Some(vec![GreatDemandChange {
                index: 2, // The FLOUR demand
                fill_type_name: Some("BARLEY".to_string()),
                demand_multiplier: Some(3.0),
                demand_start_day: Some(100),
                demand_start_hour: Some(14),
                demand_duration: Some(36),
                is_running: Some(false),
                is_valid: Some(true),
            }]),
            great_demand_additions: None,
            great_demand_deletions: None,
        };
        write_economy_changes(&save, &changes).unwrap();

        let after = parse_economy(&save).unwrap();
        assert_eq!(after.great_demands.len(), before.great_demands.len());

        let modified = after.great_demands.iter().find(|d| d.index == 2).unwrap();
        assert_eq!(modified.fill_type_name, "BARLEY");
        assert!((modified.demand_multiplier - 3.0).abs() < 0.001);
        assert_eq!(modified.demand_start_day, 100);
        assert_eq!(modified.demand_start_hour, 14);
        assert_eq!(modified.demand_duration, 36);
        assert!(!modified.is_running);

        // First demand untouched
        let first = after.great_demands.iter().find(|d| d.index == 0).unwrap();
        assert_eq!(first.fill_type_name, "SEEDS");
        assert!((first.demand_multiplier - 1.1).abs() < 0.001);

        let _ = std::fs::remove_dir_all(&save);
    }
}
