// Map configuration parsers for the density map code: fruit type lists, map
// configs of map mods and the extra fruit types recorded in the game log.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::parsers::attrs::attr_str;

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
/// Filenames of the `<fruitType filename>` entries inside `<fruitTypes>`
/// (maps_fruitTypes.xml, a map XML or a mod's fruitTypes.xml).
pub fn parse_fruit_type_filenames(data: &[u8]) -> Vec<String> {
    parse_map_config(data).fruit_type_filenames
}

/// What the density code needs from a map XML (`configFilename` of a map mod).
#[derive(Debug, Default, PartialEq)]
pub struct MapConfig {
    /// `<map width>`: the terrain size in meters.
    pub width: Option<f32>,
    /// `<farmlands filename>`: the farmland definitions.
    pub farmlands_filename: Option<String>,
    /// `<fillTypes filename>`: the map's own fill types and categories.
    pub fill_types_filename: Option<String>,
    /// `<filename>`: the map i3d.
    pub i3d_filename: Option<String>,
    /// `<fruitTypes filename>`: an external fruit types file.
    pub fruit_types_filename: Option<String>,
    /// Inline `<fruitTypes><fruitType filename>` entries.
    pub fruit_type_filenames: Vec<String>,
}

pub fn parse_map_config(data: &[u8]) -> MapConfig {
    let text = String::from_utf8_lossy(data);
    let mut reader = Reader::from_str(text.strip_prefix('\u{feff}').unwrap_or(&text));
    let mut config = MapConfig::default();
    let mut depth = 0usize;
    let mut in_fruit_types = false;
    let mut in_filename = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                match e.name().as_ref() {
                    "map" if depth == 1 => config.width = attr_str(e, "width").parse().ok(),
                    "fruitTypes" => in_fruit_types = true,
                    "filename" if depth == 2 => in_filename = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                "farmlands" => {
                    let file = attr_str(e, "filename");
                    if !file.is_empty() {
                        config.farmlands_filename = Some(file);
                    }
                }
                "fillTypes" => {
                    let file = attr_str(e, "filename");
                    if !file.is_empty() {
                        config.fill_types_filename = Some(file);
                    }
                }
                "fruitTypes" => {
                    let file = attr_str(e, "filename");
                    if !file.is_empty() {
                        config.fruit_types_filename = Some(file);
                    }
                }
                "fruitType" if in_fruit_types => {
                    let file = attr_str(e, "filename");
                    if !file.is_empty() {
                        config.fruit_type_filenames.push(file);
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref t)) if in_filename => {
                let text = t.trim().to_string();
                if !text.is_empty() {
                    config.i3d_filename = Some(text);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == "fruitTypes" {
                    in_fruit_types = false;
                }
                in_filename = false;
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    config
}

/// `configFilename` of the first `<map>` in a mod's modDesc.xml.
pub fn parse_mod_desc_map_config(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    let mut reader = Reader::from_str(text.strip_prefix('\u{feff}').unwrap_or(&text));
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.name().as_ref() == "map" => {
                let file = attr_str(e, "configFilename");
                return (!file.is_empty()).then_some(file);
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

/// Foliage XML paths of the fruit types the game adds after a map's own
/// fruit layer ("Loading additional fruit type 'x' from 'path'" in log.txt).
pub fn parse_additional_fruit_types(log: &str) -> Vec<String> {
    const MARKER: &str = "Loading additional fruit type '";
    const FROM: &str = "' from '";
    let mut files: Vec<String> = Vec::new();
    for line in log.lines() {
        let Some(start) = line.find(MARKER) else { continue };
        let rest = &line[start + MARKER.len()..];
        let Some(from) = rest.find(FROM) else { continue };
        let path = &rest[from + FROM.len()..];
        let Some(end) = path.find('\'') else { continue };
        let path = path[..end].to_string();
        if !files.contains(&path) {
            files.push(path);
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fruit_type_filenames_from_maps_fruit_types() {
        // Shape of data/maps/maps_fruitTypes.xml
        let xml = br#"<?xml version="1.0" encoding="utf-8" standalone="no" ?>
<map>
    <fruitTypes>
        <fruitType filename="$data/foliage/wheat/wheat.xml"/>
        <fruitType filename="$data/foliage/barley/barley.xml"/>
    </fruitTypes>
    <fruitTypeCategories>
        <fruitTypeCategory name="GRAINHEADER">WHEAT BARLEY</fruitTypeCategory>
    </fruitTypeCategories>
</map>"#;
        assert_eq!(
            parse_fruit_type_filenames(xml),
            vec!["$data/foliage/wheat/wheat.xml", "$data/foliage/barley/barley.xml"]
        );
    }

    #[test]
    fn test_map_config_of_a_mod_map() {
        // Shape of FS25_Pallegney.zip maps/maps.xml
        let xml = br#"<map width="2048" height="2048" imageFilename="maps/overview_Pallegney.dds">
    <filename>maps/Pallegney.i3d</filename>
    <farmlands filename="maps/config/farmlands.xml" />
    <fillTypes filename="maps/config/fillTypes.xml" />
    <fruitTypes filename="maps/config/fruitTypes.xml" />
    <fruitTypes>
        <fruitType filename="maps/foliage/extra/extra.xml"/>
    </fruitTypes>
</map>"#;
        let config = parse_map_config(xml);
        assert_eq!(config.width, Some(2048.0));
        assert_eq!(config.farmlands_filename.as_deref(), Some("maps/config/farmlands.xml"));
        assert_eq!(config.fill_types_filename.as_deref(), Some("maps/config/fillTypes.xml"));
        assert_eq!(config.i3d_filename.as_deref(), Some("maps/Pallegney.i3d"));
        assert_eq!(config.fruit_types_filename.as_deref(), Some("maps/config/fruitTypes.xml"));
        assert_eq!(config.fruit_type_filenames, vec!["maps/foliage/extra/extra.xml"]);
    }

    #[test]
    fn test_mod_desc_map_config_filename() {
        let xml = b"<modDesc><maps>
<map id=\"FS25_Pallegney\" 
        className=\"Mission00\" 
        configFilename=\"maps/maps.xml\" 
        defaultVehiclesXMLFilename=\"maps/config/vehicles.xml\"></map></maps></modDesc>";
        assert_eq!(parse_mod_desc_map_config(xml).as_deref(), Some("maps/maps.xml"));
    }

    #[test]
    fn test_additional_fruit_types_from_game_log() {
        let log = "2026-10-10 18:25:31.007   Info: Loaded fruit type 'ONION' from 'data/foliage/onion/onion.xml'
2026-10-10 18:25:43.680 Loading additional fruit type 'onion' from 'data/foliage/onion/onion.xml'
2026-10-10 18:27:43.680 Loading additional fruit type 'onion' from 'data/foliage/onion/onion.xml'
";
        assert_eq!(parse_additional_fruit_types(log), vec!["data/foliage/onion/onion.xml"]);
    }
}
