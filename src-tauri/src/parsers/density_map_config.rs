// Parser for fruitTypes.xml to build fruit type index mapping
// This maps density map pixel indices to fruit type names

use crate::error::AppError;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

/// Ground type names indexed by their value in the ground density map (bits 0-3)
pub const GROUND_TYPES: &[&str] = &[
    "NONE",               // 0
    "STUBBLE_TILLAGE",    // 1
    "CULTIVATED",         // 2
    "SEEDBED",            // 3
    "PLOWED",             // 4
    "ROLLED_SEEDBED",     // 5
    "RIDGE",              // 6
    "SOWN",               // 7
    "DIRECT_SOWN",        // 8
    "PLANTED",            // 9
    "RIDGE_SOWN",         // 10
    "ROLLER_LINES",       // 11
    "HARVEST_READY",      // 12
    "HARVEST_READY_OTHER",// 13
    "GRASS",              // 14
    "GRASS_CUT",          // 15
];

/// Parse fruitTypes.xml to build index → fruit name mapping.
/// Index 0 = no fruit, index 1+ = fruit types in order.
///
/// Fruit names are derived from the foliage filename:
///   `$data/foliage/wheat/wheat.xml` → `WHEAT`
///   `maps/foliage/silageMaize/silageMaize.xml` → `SILAGEMAIZE`
pub fn parse_fruit_types_xml(data: &[u8]) -> Result<Vec<String>, AppError> {
    let text = String::from_utf8_lossy(data);
    // Remove BOM if present
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut fruit_types = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"fruitType" => {
                if let Some(filename) = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .find(|a| a.key.as_ref() == b"filename")
                {
                    let filename_str = String::from_utf8_lossy(&filename.value);
                    let name = extract_fruit_name(&filename_str);
                    fruit_types.push(name);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::DensityMapError {
                    message: format!("Failed to parse fruitTypes.xml: {}", e),
                });
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(fruit_types)
}

/// Extract fruit type name from a foliage filename path.
/// Uses the file stem (without extension), uppercased.
///
/// Examples:
///   "$data/foliage/wheat/wheat.xml" → "WHEAT"
///   "maps/foliage/silageMaize/silageMaize.xml" → "SILAGEMAIZE"
///   "maps/foliage/meadow/fieldGrass.xml" → "FIELDGRASS"
fn extract_fruit_name(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("UNKNOWN")
        .to_uppercase()
}

/// Parse a map's own XML file (e.g. mapUS.xml) for additional fruitType entries.
/// These are appended after the shared maps_fruitTypes.xml types.
pub fn parse_map_xml_fruit_types(data: &[u8]) -> Result<Vec<String>, AppError> {
    let text = String::from_utf8_lossy(data);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);

    let mut reader = Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut fruit_types = Vec::new();
    let mut in_fruit_types = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"fruitTypes" => {
                in_fruit_types = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"fruitTypes" => {
                in_fruit_types = false;
            }
            Ok(Event::Empty(ref e))
                if in_fruit_types && e.name().as_ref() == b"fruitType" =>
            {
                if let Some(filename) = e
                    .attributes()
                    .filter_map(|a| a.ok())
                    .find(|a| a.key.as_ref() == b"filename")
                {
                    let filename_str = String::from_utf8_lossy(&filename.value);
                    let name = extract_fruit_name(&filename_str);
                    fruit_types.push(name);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break, // Don't fail on map XML parse errors
            _ => {}
        }
        buf.clear();
    }

    Ok(fruit_types)
}

/// Parse the FS25 game log to extract fruit type registration order.
/// Returns the complete ordered list of fruit types as registered by the game engine.
/// This captures DLC-added fruit types that aren't in any XML config file.
pub fn parse_game_log_fruit_types(log_path: &Path) -> Result<Vec<String>, AppError> {
    let data = std::fs::read_to_string(log_path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read game log: {}", e),
    })?;

    let mut fruit_types = Vec::new();
    let marker = "Loaded fruit type '";
    for line in data.lines() {
        if let Some(start) = line.find(marker) {
            let rest = &line[start + marker.len()..];
            if let Some(end) = rest.find('\'') {
                let name = rest[..end].to_uppercase();
                if !fruit_types.contains(&name) {
                    fruit_types.push(name);
                }
            }
        }
    }

    Ok(fruit_types)
}

/// Known FS25 fruit types that may be registered beyond XML + game log sources.
/// These are base game vegetables that the engine may register through a code path
/// not captured in the log or config XML files.
pub const KNOWN_EXTRA_FRUIT_TYPES: &[&str] = &["GREENBEAN", "PEA", "SPINACH"];

/// Read fruitTypes.xml from a filesystem path
pub fn parse_fruit_types_from_file(path: &Path) -> Result<Vec<String>, AppError> {
    let data = std::fs::read(path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read fruitTypes.xml: {}", e),
    })?;
    parse_fruit_types_xml(&data)
}

/// Read fruitTypes.xml from inside a zip archive
pub fn parse_fruit_types_from_zip(
    zip_path: &Path,
    inner_path: &str,
) -> Result<Vec<String>, AppError> {
    let file = std::fs::File::open(zip_path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to open mod zip {}: {}", zip_path.display(), e),
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read mod zip: {}", e),
    })?;

    let mut entry = archive.by_name(inner_path).map_err(|e| AppError::DensityMapError {
        message: format!("fruitTypes.xml not found in mod zip: {}", e),
    })?;

    let mut data = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read fruitTypes.xml from zip: {}", e),
    })?;

    parse_fruit_types_xml(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_fruit_name() {
        assert_eq!(extract_fruit_name("$data/foliage/wheat/wheat.xml"), "WHEAT");
        assert_eq!(
            extract_fruit_name("maps/foliage/silageMaize/silageMaize.xml"),
            "SILAGEMAIZE"
        );
        assert_eq!(
            extract_fruit_name("$data/foliage/oilseedRadish/oilseedRadish.xml"),
            "OILSEEDRADISH"
        );
        assert_eq!(
            extract_fruit_name("maps/foliage/meadow/fieldGrass.xml"),
            "FIELDGRASS"
        );
    }

    #[test]
    fn test_parse_fruit_types_xml() {
        let xml = br#"<?xml version="1.0" encoding="utf-8" standalone="no" ?>
<map>
    <fruitTypes>
        <fruitType filename="$data/foliage/wheat/wheat.xml"/>
        <fruitType filename="$data/foliage/barley/barley.xml"/>
        <fruitType filename="$data/foliage/canola/canola.xml"/>
    </fruitTypes>
</map>"#;
        let result = parse_fruit_types_xml(xml).unwrap();
        assert_eq!(result, vec!["WHEAT", "BARLEY", "CANOLA"]);
    }
}
