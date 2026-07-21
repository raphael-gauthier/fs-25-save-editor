use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{AnimalClusterChange, FillStockChange, HusbandryChange};

/// Applies husbandry changes to placeables.xml.
///
/// Runs as a separate pass from `write_placeable_changes`, reading whatever that pass
/// wrote. Targets are found by `(placeable_index, cluster_index)` because `<animal>`
/// carries no identifier.
pub fn write_husbandry_changes(
    path: &Path,
    changes: &[HusbandryChange],
) -> Result<(), AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<usize, &HusbandryChange> =
        changes.iter().map(|c| (c.placeable_index, c)).collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut placeable_index: usize = 0;
    let mut current: Option<&HusbandryChange> = None;
    let mut cluster_index: usize = 0;
    let mut in_clusters = false;
    // <storage>/<node> are shared with silos and production points, so patching is
    // gated on being inside <husbandry>.
    let mut in_husbandry = false;
    let mut in_husbandry_storage = false;
    let mut in_food = false;
    let mut in_meadow = false;

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;

        match event {
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(event, Event::Empty(_));
                let tag = e.name().as_ref().to_vec();

                match tag.as_slice() {
                    b"placeable" => {
                        current = change_map.get(&placeable_index).copied();
                        placeable_index += 1;
                        cluster_index = 0;
                        in_clusters = false;
                        in_husbandry = false;
                        in_husbandry_storage = false;
                        in_food = false;
                        in_meadow = false;
                    }
                    b"clusters" => {
                        in_clusters = !is_empty;
                        cluster_index = 0;
                    }
                    b"husbandry" => in_husbandry = !is_empty,
                    b"storage" if in_husbandry => in_husbandry_storage = !is_empty,
                    b"husbandryFood" => in_food = !is_empty,
                    b"husbandryMeadow" => in_meadow = !is_empty,
                    _ => {}
                }

                // Decide whether this element needs patching.
                // Stock arms are added in Task 9.
                let patched: Option<BytesStart<'static>> = match (current, tag.as_slice()) {
                    (Some(change), b"animal") if in_clusters => {
                        let this_index = cluster_index;
                        cluster_index += 1;
                        change
                            .clusters
                            .as_ref()
                            .and_then(|cs| cs.iter().find(|c| c.index == this_index))
                            .map(|c| patch_animal(e, c))
                    }
                    (Some(change), b"fillLevel") if in_food => {
                        find_stock(change.food.as_ref(), &attr_str(e, "fillType"))
                            .map(|sc| patch_amount(e, "fillLevel", "fillLevel", sc))
                    }
                    (Some(change), b"node") if in_husbandry_storage => {
                        find_stock(change.water.as_ref(), &attr_str(e, "fillType"))
                            .map(|sc| patch_amount(e, "node", "fillLevel", sc))
                    }
                    (Some(change), b"fillType") if in_meadow => {
                        find_stock(change.meadow.as_ref(), &attr_str(e, "name"))
                            .map(|sc| patch_amount(e, "fillType", "fillLevel", sc))
                    }
                    _ => {
                        if tag.as_slice() == b"animal" && in_clusters {
                            cluster_index += 1;
                        }
                        None
                    }
                };

                let out = match patched {
                    Some(elem) => elem,
                    None => e.clone().into_owned(),
                };
                let out_event = if is_empty {
                    Event::Empty(out)
                } else {
                    Event::Start(out)
                };
                write_event(&mut writer, &xml_path, out_event)?;
            }
            Event::End(ref e) => {
                match e.name().as_ref() {
                    b"placeable" => {
                        current = None;
                        in_husbandry = false;
                        in_husbandry_storage = false;
                        in_food = false;
                        in_meadow = false;
                    }
                    b"clusters" => in_clusters = false,
                    b"husbandry" => {
                        in_husbandry = false;
                        in_husbandry_storage = false;
                    }
                    b"storage" if in_husbandry => in_husbandry_storage = false,
                    b"husbandryFood" => in_food = false,
                    b"husbandryMeadow" => in_meadow = false,
                    _ => {}
                }
                write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
            }
            other => {
                write_event(&mut writer, &xml_path, other.into_owned())?;
            }
        }
    }

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

fn patch_animal(e: &BytesStart, change: &AnimalClusterChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("animal");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let original = String::from_utf8_lossy(&attr.value).to_string();
        let value = match key.as_str() {
            "numAnimals" => change.num_animals.map(|v| v.to_string()),
            "age" => change.age.map(|v| v.to_string()),
            "health" => change.health.map(|v| format!("{}", v.round() as i64)),
            "reproduction" => change.reproduction.map(|v| format!("{}", v.round() as i64)),
            _ => None,
        }
        .unwrap_or(original);
        elem.push_attribute((key.as_str(), value.as_str()));
    }
    elem
}

fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn find_stock<'a>(
    stocks: Option<&'a Vec<FillStockChange>>,
    fill_type: &str,
) -> Option<&'a FillStockChange> {
    stocks?.iter().find(|s| s.fill_type == fill_type)
}

/// Rewrites the amount attribute, preserving every other attribute — notably the
/// meadow's `capacity`, which is not editable.
fn patch_amount(
    e: &BytesStart,
    tag_name: &str,
    amount_key: &str,
    change: &FillStockChange,
) -> BytesStart<'static> {
    let mut elem = BytesStart::new(tag_name.to_string());
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        if key == amount_key {
            elem.push_attribute((amount_key, format!("{:.6}", change.amount).as_str()));
        } else {
            elem.push_attribute((
                key.as_str(),
                String::from_utf8_lossy(&attr.value).as_ref(),
            ));
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
    use crate::models::changes::{AnimalClusterChange, FillStockChange};
    use crate::parsers::husbandry::parse_husbandries;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wh_{}", name));
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

    fn husbandry_named<'a>(
        list: &'a [crate::models::husbandry::Husbandry],
        needle: &str,
    ) -> &'a crate::models::husbandry::Husbandry {
        list.iter()
            .find(|h| h.display_name.contains(needle))
            .unwrap()
    }

    #[test]
    fn test_write_cluster_patches_self_closing_animal() {
        let save = setup_fixture("cluster");
        let before = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&before, "Chicken Barn Big");

        let changes = vec![HusbandryChange {
            placeable_index: barn.placeable_index,
            clusters: Some(vec![AnimalClusterChange {
                index: 0,
                num_animals: Some(500),
                age: None,
                health: Some(75.0),
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&after, "Chicken Barn Big");
        assert_eq!(barn.clusters[0].num_animals, 500);
        assert!((barn.clusters[0].health - 75.0).abs() < 0.01);
        // Untouched attributes keep their original values.
        assert_eq!(barn.clusters[0].age, 48);
        assert_eq!(barn.clusters[0].sub_type, "CHICKEN");
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_cluster_targets_the_right_index() {
        // Editing the SECOND cluster catches off-by-one drift between parser and writer.
        let save = setup_fixture("cluster_second");
        let before = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&before, "Sheep Barn Small");

        let changes = vec![HusbandryChange {
            placeable_index: barn.placeable_index,
            clusters: Some(vec![AnimalClusterChange {
                index: 1,
                num_animals: Some(7),
                age: None,
                health: None,
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&after, "Sheep Barn Small");
        assert_eq!(barn.clusters[0].num_animals, 20, "first cluster must be untouched");
        assert_eq!(barn.clusters[1].num_animals, 7);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_husbandry_leaves_other_placeables_untouched() {
        let save = setup_fixture("others");
        let before = crate::parsers::placeable::parse_placeables(&save).unwrap();
        let barn = husbandry_named(&parse_husbandries(&save).unwrap(), "Chicken Barn Big")
            .placeable_index;

        let changes = vec![HusbandryChange {
            placeable_index: barn,
            clusters: Some(vec![AnimalClusterChange {
                index: 0,
                num_animals: Some(1),
                age: None,
                health: None,
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = crate::parsers::placeable::parse_placeables(&save).unwrap();
        assert_eq!(after.len(), before.len());
        for (a, b) in after.iter().zip(before.iter()) {
            assert_eq!(a.farm_id, b.farm_id);
            assert!((a.price - b.price).abs() < 0.01);
            assert_eq!(a.production_storage.len(), b.production_storage.len());
        }
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_food_water_and_meadow() {
        let save = setup_fixture("stocks");
        let all = parse_husbandries(&save).unwrap();
        let coop_index = husbandry_named(&all, "Chicken Barn Big").placeable_index;
        let barn_index = husbandry_named(&all, "Sheep Barn Small").placeable_index;

        let changes = vec![
            HusbandryChange {
                placeable_index: coop_index,
                clusters: None,
                food: Some(vec![FillStockChange {
                    fill_type: "WHEAT".to_string(),
                    amount: 999.0,
                }]),
                water: None,
                meadow: None,
            },
            HusbandryChange {
                placeable_index: barn_index,
                clusters: None,
                food: None,
                water: Some(vec![FillStockChange {
                    fill_type: "WATER".to_string(),
                    amount: 250.0,
                }]),
                meadow: Some(vec![FillStockChange {
                    fill_type: "GRASS_WINDROW".to_string(),
                    amount: 100.0,
                }]),
            },
        ];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let coop = husbandry_named(&after, "Chicken Barn Big");
        assert!((coop.food[0].amount - 999.0).abs() < 0.01);

        let barn = husbandry_named(&after, "Sheep Barn Small");
        assert!((barn.water[0].amount - 250.0).abs() < 0.01);
        assert!((barn.meadow[0].amount - 100.0).abs() < 0.01);
        // Capacity is not editable and must survive untouched.
        assert!((barn.meadow[0].capacity - 25693.414062).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_husbandry_does_not_touch_silo_or_production_stock() {
        // Fixture index 0 is a silo holding a WHEAT <node>; index 3 is a production point
        // holding a WHEAT <node>. Both share the tag names husbandry water uses.
        // NOTE: this passes because placeable_index scoping keeps `current` at None
        // while the silo/production placeables are processed — not because of the
        // <husbandry> tag gating. See test_write_husbandry_change_aimed_at_a_non_husbandry_is_inert
        // below for a test that actually exercises that gating.
        let save = setup_fixture("scoping");
        let all = parse_husbandries(&save).unwrap();
        let barn_index = husbandry_named(&all, "Sheep Barn Small").placeable_index;

        let changes = vec![HusbandryChange {
            placeable_index: barn_index,
            clusters: None,
            food: None,
            water: Some(vec![FillStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 1.0,
            }]),
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""),
            "silo stock was modified"
        );
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"2500.000000\""),
            "production stock was modified"
        );
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_husbandry_change_aimed_at_a_non_husbandry_is_inert() {
        // Guards the worst failure mode: if placeable_index ever drifts out of sync
        // with the parser, a husbandry edit could land on a silo. The <husbandry>
        // gating is what makes such a misfire inert instead of destructive.
        // Index 0 is the silo, holding WHEAT 20657.392578 and SUNFLOWER 4855.552734.
        let save = setup_fixture("misaimed");

        let changes = vec![HusbandryChange {
            placeable_index: 0,
            clusters: None,
            food: None,
            water: Some(vec![FillStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 1.0,
            }]),
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""),
            "a husbandry change aimed at a silo overwrote its grain"
        );
        // No SUNFLOWER assertion here: the change targets fill_type WHEAT only, so
        // find_stock's fill_type filter keeps SUNFLOWER out of reach regardless of the
        // <husbandry> gating this test exercises. Asserting on it would never fail
        // under this test's failure mode, so it would be decorative.
        let _ = std::fs::remove_dir_all(&save);
    }
}
