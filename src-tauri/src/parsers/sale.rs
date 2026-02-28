use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::sale::{BoughtConfiguration, SaleItem};
use crate::models::vehicle::vehicle_display_name;

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

/// Parse sales.xml and return the list of items for sale.
pub fn parse_sales(path: &Path) -> Result<Vec<SaleItem>, AppError> {
    let xml_path = path.join("sales.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut sales: Vec<SaleItem> = Vec::new();
    let mut index: usize = 0;

    let mut current_item: Option<SaleItemBuilder> = None;
    let mut in_bought_configs = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "item" => {
                        let xml_filename = attr_str(e, "xmlFilename");
                        let display_name = vehicle_display_name(&xml_filename);
                        current_item = Some(SaleItemBuilder {
                            index,
                            xml_filename,
                            display_name,
                            age: attr_u32(e, "age"),
                            price: attr_u32(e, "price"),
                            damage: attr_f64(e, "damage"),
                            wear: attr_f64(e, "wear"),
                            operating_time: attr_f64(e, "operatingTime") / 60.0,
                            time_left: attr_u32(e, "timeLeft"),
                            is_generated: attr_str(e, "isGenerated") == "true",
                            bought_configurations: Vec::new(),
                        });
                        index += 1;
                    }
                    "boughtConfigurations" => in_bought_configs = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "item" {
                    // Self-closing <item .../> (no boughtConfigurations)
                    let xml_filename = attr_str(e, "xmlFilename");
                    let display_name = vehicle_display_name(&xml_filename);
                    sales.push(SaleItem {
                        index,
                        xml_filename,
                        display_name,
                        age: attr_u32(e, "age"),
                        price: attr_u32(e, "price"),
                        damage: attr_f64(e, "damage"),
                        wear: attr_f64(e, "wear"),
                        operating_time: attr_f64(e, "operatingTime") / 60.0,
                        time_left: attr_u32(e, "timeLeft"),
                        is_generated: attr_str(e, "isGenerated") == "true",
                        bought_configurations: Vec::new(),
                    });
                    index += 1;
                } else if let Some(ref mut item) = current_item {
                    if tag == "boughtConfiguration" && in_bought_configs {
                        item.bought_configurations.push(BoughtConfiguration {
                            name: attr_str(e, "name"),
                            id: attr_str(e, "id"),
                        });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag.as_str() {
                    "item" => {
                        if let Some(item) = current_item.take() {
                            sales.push(item.build());
                        }
                    }
                    "boughtConfigurations" => in_bought_configs = false,
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

    Ok(sales)
}

struct SaleItemBuilder {
    index: usize,
    xml_filename: String,
    display_name: String,
    age: u32,
    price: u32,
    damage: f64,
    wear: f64,
    operating_time: f64,
    time_left: u32,
    is_generated: bool,
    bought_configurations: Vec<BoughtConfiguration>,
}

impl SaleItemBuilder {
    fn build(self) -> SaleItem {
        SaleItem {
            index: self.index,
            xml_filename: self.xml_filename,
            display_name: self.display_name,
            age: self.age,
            price: self.price,
            damage: self.damage,
            wear: self.wear,
            operating_time: self.operating_time,
            time_left: self.time_left,
            is_generated: self.is_generated,
            bought_configurations: self.bought_configurations,
        }
    }
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
    fn test_parse_sales_nominal() {
        let path = fixtures_path().join("savegame_complete");
        let sales = parse_sales(&path).unwrap();
        assert_eq!(sales.len(), 2);
        assert_eq!(sales[0].index, 0);
        assert_eq!(sales[1].index, 1);
        assert!(sales[0].price > 0);
    }

    #[test]
    fn test_parse_sales_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_sales");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_sales(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
