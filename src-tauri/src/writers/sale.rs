use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::SaleChange;

/// Applies sale changes to sales.xml.
/// Items are identified by their position index (0-based count of <item> elements).
pub fn write_sale_changes(
    path: &Path,
    changes: &[SaleChange],
) -> Result<(), AppError> {
    let xml_path = path.join("sales.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    // Build lookup map by index
    let change_map: std::collections::HashMap<usize, &SaleChange> = changes
        .iter()
        .map(|c| (c.index, c))
        .collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut item_index: usize = 0;
    let mut skip_until_item_end = false;
    let mut skip_depth: u32 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if skip_until_item_end {
                    skip_depth += 1;
                    continue;
                }

                if tag == "item" {
                    if let Some(change) = change_map.get(&item_index) {
                        item_index += 1;
                        if change.delete {
                            skip_until_item_end = true;
                            skip_depth = 1;
                            continue;
                        }
                        let elem = patch_item_start(e, change);
                        write_event(&mut writer, &xml_path, Event::Start(elem))?;
                    } else {
                        item_index += 1;
                        write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                    }
                } else {
                    write_event(&mut writer, &xml_path, Event::Start(e.clone().into_owned()))?;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if skip_until_item_end {
                    if tag == "item" {
                        skip_depth -= 1;
                        if skip_depth == 0 {
                            skip_until_item_end = false;
                        }
                    } else {
                        skip_depth -= 1;
                    }
                    continue;
                }

                write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if !skip_until_item_end {
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

fn patch_item_start(e: &BytesStart, change: &SaleChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("item");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        match key.as_str() {
            "price" if change.price.is_some() => {
                elem.push_attribute(("price", change.price.unwrap().to_string().as_str()));
            }
            "damage" if change.damage.is_some() => {
                elem.push_attribute(("damage", format!("{:.6}", change.damage.unwrap()).as_str()));
            }
            "wear" if change.wear.is_some() => {
                elem.push_attribute(("wear", format!("{:.6}", change.wear.unwrap()).as_str()));
            }
            "age" if change.age.is_some() => {
                elem.push_attribute(("age", change.age.unwrap().to_string().as_str()));
            }
            "operatingTime" if change.operating_time.is_some() => {
                elem.push_attribute((
                    "operatingTime",
                    format!("{:.6}", change.operating_time.unwrap() * 60.0).as_str(),
                ));
            }
            "timeLeft" if change.time_left.is_some() => {
                elem.push_attribute(("timeLeft", change.time_left.unwrap().to_string().as_str()));
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
    writer.write_event(event).map_err(|e| AppError::XmlParseError {
        file: xml_path.display().to_string(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::sale::parse_sales;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_ws_{}", name));
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
    fn test_write_sale_price() {
        let save = setup_fixture("sale_price");
        let changes = vec![SaleChange {
            index: 0,
            delete: false,
            price: Some(1234),
            damage: None,
            wear: None,
            age: None,
            operating_time: None,
            time_left: None,
        }];
        write_sale_changes(&save, &changes).unwrap();
        let sales = parse_sales(&save).unwrap();
        assert_eq!(sales[0].price, 1234);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_sale_wear_damage() {
        let save = setup_fixture("sale_wear");
        let changes = vec![SaleChange {
            index: 1,
            delete: false,
            price: None,
            damage: Some(0.0),
            wear: Some(0.0),
            age: None,
            operating_time: None,
            time_left: None,
        }];
        write_sale_changes(&save, &changes).unwrap();
        let sales = parse_sales(&save).unwrap();
        assert!((sales[1].damage - 0.0).abs() < 0.001);
        assert!((sales[1].wear - 0.0).abs() < 0.001);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_sale_delete() {
        let save = setup_fixture("sale_delete");
        let changes = vec![SaleChange {
            index: 0,
            delete: true,
            price: None,
            damage: None,
            wear: None,
            age: None,
            operating_time: None,
            time_left: None,
        }];
        write_sale_changes(&save, &changes).unwrap();
        let sales = parse_sales(&save).unwrap();
        assert_eq!(sales.len(), 1);
        // Remaining item should be what was formerly index 1
        assert_eq!(sales[0].display_name, "John Deere 6 M");
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_sale_preserves_others() {
        let save = setup_fixture("sale_preserve");
        let before = parse_sales(&save).unwrap();
        let changes = vec![SaleChange {
            index: 0,
            delete: false,
            price: Some(1),
            damage: None,
            wear: None,
            age: None,
            operating_time: None,
            time_left: None,
        }];
        write_sale_changes(&save, &changes).unwrap();
        let after = parse_sales(&save).unwrap();

        // Item 1 (index 1) should be unchanged
        assert_eq!(after[1].price, before[1].price);
        assert!((after[1].damage - before[1].damage).abs() < 0.001);
        assert!((after[1].wear - before[1].wear).abs() < 0.001);
        assert_eq!(after[1].bought_configurations.len(), before[1].bought_configurations.len());

        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_sale_roundtrip() {
        let save = setup_fixture("sale_roundtrip");
        let changes = vec![SaleChange {
            index: 0,
            delete: false,
            price: Some(99999),
            damage: Some(0.5),
            wear: Some(0.75),
            age: Some(100),
            operating_time: Some(500.0),
            time_left: Some(20),
        }];
        write_sale_changes(&save, &changes).unwrap();
        let sales = parse_sales(&save).unwrap();

        assert_eq!(sales[0].price, 99999);
        assert!((sales[0].damage - 0.5).abs() < 0.001);
        assert!((sales[0].wear - 0.75).abs() < 0.001);
        assert_eq!(sales[0].age, 100);
        assert!((sales[0].operating_time - 500.0).abs() < 0.01);
        assert_eq!(sales[0].time_left, 20);

        let _ = std::fs::remove_dir_all(&save);
    }
}
