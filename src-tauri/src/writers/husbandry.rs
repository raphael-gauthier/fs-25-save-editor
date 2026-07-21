use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{AnimalClusterChange, HusbandryChange};

/// Applies husbandry changes to placeables.xml.
///
/// Runs as a separate pass from `write_placeable_changes`, reading whatever that pass
/// wrote. Targets are found by `(placeable_index, cluster_index)` because `<animal>`
/// carries no identifier.
#[allow(dead_code)]
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
                    }
                    b"clusters" => {
                        in_clusters = !is_empty;
                        cluster_index = 0;
                    }
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
                    b"placeable" => current = None,
                    b"clusters" => in_clusters = false,
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
    use crate::models::changes::AnimalClusterChange;
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
}
