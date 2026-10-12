use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::writers::replace_xml_file;
use crate::models::changes::{PlaceableChange, ProductionStockChange};

/// Applies a list of placeable changes to placeables.xml.
/// Patch strategy: reads original XML, modifies only targeted attributes/elements, rewrites atomically.
pub fn write_placeable_changes(
    path: &Path,
    changes: &[PlaceableChange],
) -> Result<(), AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<usize, &PlaceableChange> = changes
        .iter()
        .map(|c| (c.index, c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    // State tracking
    let mut placeable_index: usize = 0;
    let mut in_placeable = false;
    let mut current_change: Option<&PlaceableChange> = None;
    // `<storage>`/`<node>` are shared by silos, husbandries and production points,
    // so stock patching is gated on being inside a `<productionPoint>`.
    let mut in_constructible = false;
    let mut in_production_point = false;
    let mut in_production_storage = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = e.name().as_ref().to_string();

                match tag.as_str() {
                    "placeable" if !in_placeable => {
                        in_placeable = true;
                        current_change = change_map.get(&placeable_index).copied();
                        placeable_index += 1;

                        if let Some(change) = current_change {
                            let elem = patch_placeable_start(e, change);
                            write_event(&mut writer, &xml_path, Event::Start(elem))?;
                        } else {
                            write_event(
                                &mut writer,
                                &xml_path,
                                Event::Start(e.clone().into_owned()),
                            )?;
                        }
                    }
                    "constructible" if in_placeable => {
                        in_constructible = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    "productionPoint" if in_placeable => {
                        in_production_point = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    "storage" if in_production_point => {
                        in_production_storage = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    _ => {
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = e.name().as_ref().to_string();

                // A self-closing <placeable .../> (pre-placed buildings, ~37% of a real
                // save) never opens a Start/End pair, so it never sets `in_placeable`.
                // It must still be counted here — otherwise placeable_index drifts out
                // of sync with the parser and every later edit lands on the wrong
                // building. It has no children, so only its own attributes are patched.
                if tag == "placeable" {
                    let change = change_map.get(&placeable_index).copied();
                    placeable_index += 1;

                    match change {
                        Some(change) => {
                            let elem = patch_placeable_start(e, change);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                        }
                        None => {
                            write_event(
                                &mut writer,
                                &xml_path,
                                Event::Empty(e.clone().into_owned()),
                            )?;
                        }
                    }
                    continue;
                }

                if let Some(change) = current_change {
                    match tag.as_str() {
                        "input" if in_constructible && change.complete_construction => {
                            let elem = patch_construction_input_complete(e);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                            continue;
                        }
                        "node" if in_production_storage => {
                            if let Some(ref stock_changes) = change.production_storage {
                                let fill_type = attr_str(e, "fillType");
                                if let Some(sc) =
                                    stock_changes.iter().find(|s| s.fill_type == fill_type)
                                {
                                    let elem = patch_storage_node(e, sc);
                                    write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                write_event(
                    &mut writer,
                    &xml_path,
                    Event::Empty(e.clone().into_owned()),
                )?;
            }
            Ok(Event::End(ref e)) => {
                let tag = e.name().as_ref().to_string();

                match tag.as_str() {
                    "placeable" if in_placeable => {
                        in_placeable = false;
                        current_change = None;
                        in_constructible = false;
                        in_production_point = false;
                        in_production_storage = false;
                    }
                    "constructible" => in_constructible = false,
                    "productionPoint" => {
                        in_production_point = false;
                        in_production_storage = false;
                    }
                    "storage" if in_production_point => in_production_storage = false,
                    _ => {}
                }

                write_event(
                    &mut writer,
                    &xml_path,
                    Event::End(e.clone().into_owned()),
                )?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                write_event(&mut writer, &xml_path, event.into_owned())?;
            }
            Err(e) => {
                return Err(AppError::XmlParseError {
                    file: xml_path.display().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    replace_xml_file(&xml_path, &content, &writer.into_inner())?;

    Ok(())
}

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .map(|a| a.value.to_string())
        .unwrap_or_default()
}

fn patch_placeable_start(e: &BytesStart, change: &PlaceableChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("placeable");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "farmId" if change.farm_id.is_some() => {
                elem.push_attribute((
                    "farmId",
                    change.farm_id.unwrap().to_string().as_str(),
                ));
            }
            "price" if change.price.is_some() => {
                elem.push_attribute((
                    "price",
                    format!("{:.6}", change.price.unwrap()).as_str(),
                ));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    attr.value.as_ref(),
                ));
            }
        }
    }
    elem
}

/// Marks a construction material as fully delivered by zeroing its outstanding amount.
fn patch_construction_input_complete(e: &BytesStart) -> BytesStart<'static> {
    let mut elem = BytesStart::new("input");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "remainingAmount" => {
                elem.push_attribute(("remainingAmount", "0.000000"));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    attr.value.as_ref(),
                ));
            }
        }
    }
    elem
}

fn patch_storage_node(e: &BytesStart, change: &ProductionStockChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("node");
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref().to_string();
        match key.as_str() {
            "fillLevel" => {
                elem.push_attribute((
                    "fillLevel",
                    format!("{:.6}", change.amount).as_str(),
                ));
            }
            _ => {
                elem.push_attribute((
                    key.as_str(),
                    attr.value.as_ref(),
                ));
            }
        }
    }
    elem
}

fn write_event(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer
        .write_event(event)
        .map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::placeable::parse_placeables;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wp_{}", name));
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
    fn test_write_placeable_owner() {
        let save = setup_fixture("owner");
        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: Some(2),
            price: None,
            complete_construction: false,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let placeables = parse_placeables(&save).unwrap();
        assert_eq!(placeables[0].farm_id, 2);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_complete_construction() {
        let save = setup_fixture("complete");
        // Find which index is under construction
        let before = parse_placeables(&save).unwrap();
        let uc = before.iter().find(|p| p.is_under_construction).unwrap();
        let uc_index = uc.index;

        let changes = vec![PlaceableChange {
            index: uc_index,
            farm_id: None,
            price: None,
            complete_construction: true,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[uc_index];
        // Every <input remainingAmount> is zeroed, so nothing is left to deliver.
        assert!(!p.is_under_construction);
        let construction = p.construction.as_ref().unwrap();
        for mat in &construction.inputs {
            assert!(mat.amount_remaining <= 0.0);
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_production_stock() {
        let save = setup_fixture("production");
        let before = parse_placeables(&save).unwrap();
        let prod = before
            .iter()
            .find(|p| !p.production_storage.is_empty())
            .unwrap();
        let prod_index = prod.index;
        let fill_type = prod.production_storage[0].fill_type.clone();

        let changes = vec![PlaceableChange {
            index: prod_index,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: fill_type.clone(),
                amount: 5000.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[prod_index];
        let stock = p
            .production_storage
            .iter()
            .find(|s| s.fill_type == fill_type)
            .unwrap();
        assert!((stock.amount - 5000.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_leaves_silo_storage_untouched() {
        let save = setup_fixture("silo_untouched");
        // Fixture precondition (not a result to verify): the silo at index 0 has
        // <silo><storage><node fillType="WHEAT"/><node fillType="SUNFLOWER"/></storage></silo>,
        // which shares the <node> shape used by production points. Editing a production
        // point must not touch it. The SUNFLOWER node is checked after the write below.

        let changes = vec![PlaceableChange {
            index: 3,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 42.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        // Production point updated...
        let stock = after[3]
            .production_storage
            .iter()
            .find(|s| s.fill_type == "WHEAT")
            .unwrap();
        assert!((stock.amount - 42.0).abs() < 0.01);
        // ...while the silo's own nodes kept their original levels.
        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""));
        assert!(raw.contains("fillType=\"SUNFLOWER\" fillLevel=\"4855.552734\""));
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_production_storage_change_does_not_patch_non_production_node() {
        // Exercises the `in_production_storage` gate directly. Index 0 is the silo:
        // its <node> elements sit under <silo><storage>, never inside a
        // <productionPoint>. A production_storage change aimed at the SILO'S OWN
        // index sees `current_change = Some(..)` while walking those nodes, so
        // whether they get patched depends entirely on the `if in_production_storage`
        // guard on the "node" arm — not on index scoping. (A change aimed at the
        // production point's own index, as in the test above, would never reach the
        // silo regardless of the guard, because `current_change` stays None while
        // walking the silo's subtree — that would be a toothless test.)
        let save = setup_fixture("silo_gate");

        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_storage: Some(vec![ProductionStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 42.0,
            }]),
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""),
            "silo's WHEAT node was patched by a production_storage change not scoped to a productionPoint"
        );
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_targets_correct_index_past_a_self_closing_placeable() {
        // 37% of placeables in a real save are self-closing. The parser counts them;
        // if the writer does not, every edit after the first one lands on the wrong
        // building. Fixture index 1 is self-closing, so index 2 is only reachable
        // if both sides agree.
        let save = setup_fixture("self_closing_index");
        let before = parse_placeables(&save).unwrap();

        let target = 2usize;
        let target_name = before[target].display_name.clone();

        let changes = vec![PlaceableChange {
            index: target,
            farm_id: Some(7),
            price: None,
            complete_construction: false,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after[target].display_name, target_name, "index drifted");
        assert_eq!(after[target].farm_id, 7, "the edit did not reach its target");
        for (i, (a, b)) in after.iter().zip(before.iter()).enumerate() {
            if i != target {
                assert_eq!(a.farm_id, b.farm_id, "placeable {} was collaterally modified", i);
            }
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_can_edit_a_self_closing_placeable() {
        // The self-closing pre-placed entries are exactly the ones a player claims
        // after buying farmland, so they must be writable.
        let save = setup_fixture("self_closing_edit");
        let changes = vec![PlaceableChange {
            index: 1,
            farm_id: Some(3),
            price: None,
            complete_construction: false,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();

        let after = parse_placeables(&save).unwrap();
        assert_eq!(after[1].farm_id, 3);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_roundtrip() {
        let save = setup_fixture("roundtrip_p");
        let before = parse_placeables(&save).unwrap();

        // Modify first placeable's price
        let changes = vec![PlaceableChange {
            index: 0,
            farm_id: None,
            price: Some(999999.0),
            complete_construction: false,
            production_storage: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();

        assert_eq!(after.len(), before.len());
        assert!((after[0].price - 999999.0).abs() < 0.01);

        // Other placeables should be untouched
        for i in 1..before.len() {
            assert!((before[i].price - after[i].price).abs() < 0.01);
            assert_eq!(before[i].farm_id, after[i].farm_id);
        }

        let _ = std::fs::remove_dir_all(&save);
    }
}
