use std::fs;
use std::io::Read;
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::models::catalog::{CatalogVehicle, VehicleSource};

/// Scan base game vehicles directory and mod zips to build a vehicle catalog.
pub fn scan_vehicle_catalog(game_path: &Path, mods_dir: &Path) -> Vec<CatalogVehicle> {
    let mut catalog = Vec::new();

    // Scan base game vehicles
    let vehicles_dir = game_path.join("data").join("vehicles");
    if vehicles_dir.exists() {
        scan_directory_recursive(&vehicles_dir, game_path, &mut catalog);
    }

    // Scan DLC/PDLC vehicles
    let pdlc_dir = game_path.join("pdlc");
    if pdlc_dir.exists() {
        if let Ok(entries) = fs::read_dir(&pdlc_dir) {
            for entry in entries.flatten() {
                let dlc_vehicles = entry.path().join("data").join("vehicles");
                if dlc_vehicles.exists() {
                    scan_directory_recursive(&dlc_vehicles, game_path, &mut catalog);
                }
            }
        }
    }

    // Scan mod zips
    if mods_dir.exists() {
        if let Ok(entries) = fs::read_dir(mods_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                    scan_mod_zip(&path, &mut catalog);
                }
            }
        }
    }

    // Sort by brand then name
    catalog.sort_by(|a, b| a.brand.cmp(&b.brand).then(a.name.cmp(&b.name)));

    catalog
}

fn scan_directory_recursive(
    dir: &Path,
    game_path: &Path,
    catalog: &mut Vec<CatalogVehicle>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory_recursive(&path, game_path, catalog);
        } else if path.extension().and_then(|e| e.to_str()) == Some("xml") {
            if let Some(vehicle) = try_parse_vehicle_xml_file(&path, game_path) {
                catalog.push(vehicle);
            }
        }
    }
}

fn try_parse_vehicle_xml_file(
    xml_path: &Path,
    game_path: &Path,
) -> Option<CatalogVehicle> {
    let content = fs::read_to_string(xml_path).ok()?;
    let store_data = parse_store_data(&content)?;

    // Build relative xml_filename like "data/vehicles/fendt/vario900/vario900.xml"
    let xml_filename = xml_path
        .strip_prefix(game_path)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");

    Some(CatalogVehicle {
        xml_filename,
        name: store_data.name,
        brand: store_data.brand,
        category: store_data.category,
        price: store_data.price,
        source: VehicleSource::BaseGame,
    })
}

fn scan_mod_zip(zip_path: &Path, catalog: &mut Vec<CatalogVehicle>) {
    let file = match fs::File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return,
    };

    let mod_name = zip_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Collect XML file names first to avoid borrow issues
    let xml_names: Vec<String> = (0..archive.len())
        .filter_map(|i| {
            let entry = archive.by_index(i).ok()?;
            let name = entry.name().to_string();
            if name.ends_with(".xml") && !name.contains("__MACOSX") {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    for name in xml_names {
        let content = match archive.by_name(&name) {
            Ok(mut entry) => {
                let mut buf = String::new();
                if entry.read_to_string(&mut buf).is_ok() {
                    buf
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        };

        if let Some(store_data) = parse_store_data(&content) {
            // For mods, the xmlFilename in sales.xml uses the absolute path format
            // but we store as $moddir$ format for consistency
            let xml_filename = format!("$moddir${}/{}", mod_name, name);

            catalog.push(CatalogVehicle {
                xml_filename,
                name: store_data.name,
                brand: store_data.brand,
                category: store_data.category,
                price: store_data.price,
                source: VehicleSource::Mod(mod_name.clone()),
            });
        }
    }
}

struct StoreData {
    name: String,
    brand: String,
    category: String,
    price: u32,
}

/// Parse `<storeData>` section from a vehicle XML and extract name, brand, category, price.
fn parse_store_data(xml_content: &str) -> Option<StoreData> {
    let mut reader = Reader::from_str(xml_content);

    let mut in_store_data = false;
    let mut current_tag = String::new();
    let mut name = None;
    let mut brand = None;
    let mut category = None;
    let mut price = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "storeData" {
                    in_store_data = true;
                } else if in_store_data {
                    current_tag = tag;
                }
            }
            Ok(Event::Text(ref text)) => {
                if in_store_data && !current_tag.is_empty() {
                    if let Ok(value) = text.unescape() {
                        let trimmed = value.trim().to_string();
                        if !trimmed.is_empty() {
                            match current_tag.as_str() {
                                "name" => name = Some(clean_localized_name(&trimmed)),
                                "brand" => brand = Some(trimmed),
                                "category" => category = Some(trimmed),
                                "price" => price = trimmed.parse::<u32>().ok(),
                                _ => {}
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "storeData" {
                    break;
                }
                if in_store_data {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // All fields required
    let name = name?;
    let brand = brand?;
    let category = category?;
    let price = price?;

    // Skip items with price 0 (internal/debug vehicles)
    if price == 0 {
        return None;
    }

    Some(StoreData {
        name,
        brand,
        category,
        price,
    })
}

/// Clean localized names: "$l10n_shopItem_frontloaderPalletFork" → "Frontloader Pallet Fork"
fn clean_localized_name(name: &str) -> String {
    if !name.starts_with("$l10n_") {
        return name.to_string();
    }

    // Strip prefix
    let raw = &name[6..]; // remove "$l10n_"

    // Remove common prefixes like "shopItem_"
    let raw = raw
        .strip_prefix("shopItem_")
        .or_else(|| raw.strip_prefix("shop_"))
        .unwrap_or(raw);

    // Convert camelCase/underscores to title case
    let mut result = String::new();
    let mut prev_was_lower = false;

    for ch in raw.chars() {
        if ch == '_' {
            result.push(' ');
            prev_was_lower = false;
            continue;
        }
        if ch.is_uppercase() && prev_was_lower {
            result.push(' ');
        }
        if result.is_empty() || result.ends_with(' ') {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
        prev_was_lower = ch.is_lowercase();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_store_data_basic() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicle type="tractor">
  <storeData>
    <name>900 Vario</name>
    <brand>FENDT</brand>
    <category>tractorsL</category>
    <price>327500</price>
    <image>$data/vehicles/fendt/vario900/store_vario900.png</image>
  </storeData>
</vehicle>"#;

        let result = parse_store_data(xml).unwrap();
        assert_eq!(result.name, "900 Vario");
        assert_eq!(result.brand, "FENDT");
        assert_eq!(result.category, "tractorsL");
        assert_eq!(result.price, 327500);
    }

    #[test]
    fn test_parse_store_data_localized_name() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicle type="implement">
  <storeData>
    <name>$l10n_shopItem_frontloaderPalletFork</name>
    <brand>ALBUTT</brand>
    <category>frontLoaderTools</category>
    <price>500</price>
  </storeData>
</vehicle>"#;

        let result = parse_store_data(xml).unwrap();
        assert_eq!(result.name, "Frontloader Pallet Fork");
    }

    #[test]
    fn test_parse_store_data_missing_fields() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicle type="tractor">
  <storeData>
    <name>Test</name>
    <brand>TEST</brand>
  </storeData>
</vehicle>"#;

        assert!(parse_store_data(xml).is_none());
    }

    #[test]
    fn test_parse_store_data_no_store_data() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicle type="tractor">
  <base>
    <filename>test.i3d</filename>
  </base>
</vehicle>"#;

        assert!(parse_store_data(xml).is_none());
    }

    #[test]
    fn test_parse_store_data_zero_price() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<vehicle type="tractor">
  <storeData>
    <name>Debug Vehicle</name>
    <brand>NONE</brand>
    <category>misc</category>
    <price>0</price>
  </storeData>
</vehicle>"#;

        assert!(parse_store_data(xml).is_none());
    }

    #[test]
    fn test_clean_localized_name() {
        assert_eq!(clean_localized_name("900 Vario"), "900 Vario");
        assert_eq!(
            clean_localized_name("$l10n_shopItem_frontloaderPalletFork"),
            "Frontloader Pallet Fork"
        );
        assert_eq!(
            clean_localized_name("$l10n_shop_myVehicle"),
            "My Vehicle"
        );
    }
}
