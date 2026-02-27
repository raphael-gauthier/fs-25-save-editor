use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
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
    let mut in_construction = false;
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
                    "constructionPlaceable" if in_placeable => {
                        in_construction = true;
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
                    "input" if in_production_point => {
                        in_production_input = true;
                        write_event(
                            &mut writer,
                            &xml_path,
                            Event::Start(e.clone().into_owned()),
                        )?;
                    }
                    "output" if in_production_point => {
                        in_production_output = true;
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
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if let Some(change) = current_change {
                    match tag.as_str() {
                        "material" if in_construction && change.complete_construction => {
                            let elem = patch_material_complete(e);
                            write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                            continue;
                        }
                        "storage" if in_production_input => {
                            if let Some(ref input_changes) = change.production_inputs {
                                let fill_type = attr_str(e, "fillType");
                                if let Some(sc) = input_changes.iter().find(|s| s.fill_type == fill_type) {
                                    let elem = patch_storage(e, sc);
                                    write_event(&mut writer, &xml_path, Event::Empty(elem))?;
                                    continue;
                                }
                            }
                        }
                        "storage" if in_production_output => {
                            if let Some(ref output_changes) = change.production_outputs {
                                let fill_type = attr_str(e, "fillType");
                                if let Some(sc) = output_changes.iter().find(|s| s.fill_type == fill_type) {
                                    let elem = patch_storage(e, sc);
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
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag.as_str() {
                    "placeable" if in_placeable => {
                        in_placeable = false;
                        current_change = None;
                    }
                    "constructionPlaceable" => in_construction = false,
                    "productionPoint" => {
                        in_production_point = false;
                        in_production_input = false;
                        in_production_output = false;
                    }
                    "input" if in_production_point => in_production_input = false,
                    "output" if in_production_point => in_production_output = false,
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

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn patch_placeable_start(e: &BytesStart, change: &PlaceableChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("placeable");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
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
                    String::from_utf8_lossy(&attr.value).as_ref(),
                ));
            }
        }
    }
    elem
}

fn patch_material_complete(e: &BytesStart) -> BytesStart<'static> {
    let mut elem = BytesStart::new("material");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "amountRemaining" => {
                elem.push_attribute(("amountRemaining", "0"));
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

fn patch_storage(e: &BytesStart, change: &ProductionStockChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("storage");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
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
                    String::from_utf8_lossy(&attr.value).as_ref(),
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
            production_inputs: None,
            production_outputs: None,
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
            production_inputs: None,
            production_outputs: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[uc_index];
        // After completing, all remaining should be 0
        assert!(!p.is_under_construction);
        for step in &p.construction_steps {
            for mat in &step.materials {
                assert!(mat.amount_remaining <= 0.0);
            }
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_placeable_production_stock() {
        let save = setup_fixture("production");
        let before = parse_placeables(&save).unwrap();
        let prod = before
            .iter()
            .find(|p| !p.production_inputs.is_empty())
            .unwrap();
        let prod_index = prod.index;
        let fill_type = prod.production_inputs[0].fill_type.clone();

        let changes = vec![PlaceableChange {
            index: prod_index,
            farm_id: None,
            price: None,
            complete_construction: false,
            production_inputs: Some(vec![ProductionStockChange {
                fill_type: fill_type.clone(),
                amount: 5000.0,
            }]),
            production_outputs: None,
        }];
        write_placeable_changes(&save, &changes).unwrap();
        let after = parse_placeables(&save).unwrap();
        let p = &after[prod_index];
        let input = p
            .production_inputs
            .iter()
            .find(|s| s.fill_type == fill_type)
            .unwrap();
        assert!((input.amount - 5000.0).abs() < 0.01);
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
            production_inputs: None,
            production_outputs: None,
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
