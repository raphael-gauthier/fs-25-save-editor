// Storage definitions of placeables (their XML in the game or a mod): the
// capacities and accepted fill types Storage.lua and PlaceableObjectStorage.lua
// load, the fill type categories they refer to, and which XML a map's
// pre-placed placeables come from.

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::parsers::attrs::attr_str;

/// One `<storage>` of a placeable XML.
#[derive(Debug, Clone, PartialEq)]
pub struct StorageDef {
    /// `capacity`, 100000 when missing (Storage.lua)
    pub capacity: f64,
    /// `<capacity fillType capacity>` overrides per fill type
    pub capacities: Vec<(String, f64)>,
    /// `fillTypeCategories`, uppercased
    pub fill_type_categories: Vec<String>,
    /// `fillTypes`, uppercased
    pub fill_types: Vec<String>,
    /// `supportsMultipleFillTypes`: when false the storage holds one fill
    /// type at a time; when true they share `capacity`
    pub supports_multiple_fill_types: bool,
}

/// Storages a placeable XML defines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StorageDefs {
    /// `<silo><storages><storage>`, in order: the save's `<storage index>` is 1-based
    pub silo_storages: Vec<StorageDef>,
    /// `<siloExtension><storage>`
    pub extension: Option<StorageDef>,
    /// `<objectStorage capacity>`, 250 when missing; None without an object storage
    pub object_storage_capacity: Option<u32>,
}

/// Uppercased names of a space-separated list attribute.
fn names(list: &str) -> Vec<String> {
    list.split_whitespace().map(|n| n.to_uppercase()).collect()
}

pub fn parse_storage_defs(xml: &str) -> StorageDefs {
    let mut reader = Reader::from_str(xml.strip_prefix('\u{feff}').unwrap_or(xml));
    let mut defs = StorageDefs::default();
    let mut in_silo_storages = false;
    let mut in_extension = false;
    let mut open: Option<StorageDef> = None;

    loop {
        let (e, is_empty) = match reader.read_event() {
            Ok(Event::Start(e)) => (e, false),
            Ok(Event::Empty(e)) => (e, true),
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    "storage" => {
                        if let Some(def) = open.take() {
                            if in_silo_storages {
                                defs.silo_storages.push(def);
                            } else if in_extension {
                                defs.extension = Some(def);
                            }
                        }
                    }
                    "storages" => in_silo_storages = false,
                    "siloExtension" => in_extension = false,
                    _ => {}
                }
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };

        match e.name().as_ref() {
            "storages" => in_silo_storages = !is_empty,
            "siloExtension" => in_extension = !is_empty,
            "storage" if in_silo_storages || in_extension => {
                let def = StorageDef {
                    capacity: attr_str(&e, "capacity").parse().unwrap_or(100_000.0),
                    capacities: Vec::new(),
                    fill_type_categories: names(&attr_str(&e, "fillTypeCategories")),
                    fill_types: names(&attr_str(&e, "fillTypes")),
                    supports_multiple_fill_types: attr_str(&e, "supportsMultipleFillTypes") != "false",
                };
                if !is_empty {
                    open = Some(def);
                } else if in_silo_storages {
                    defs.silo_storages.push(def);
                } else {
                    defs.extension = Some(def);
                }
            }
            "capacity" => {
                if let Some(ref mut def) = open {
                    let capacity = attr_str(&e, "capacity").parse().unwrap_or(100_000.0);
                    def.capacities.push((attr_str(&e, "fillType").to_uppercase(), capacity));
                }
            }
            "objectStorage" => {
                defs.object_storage_capacity = Some(attr_str(&e, "capacity").parse().unwrap_or(250));
            }
            _ => {}
        }
    }

    defs
}

/// `<fillTypeCategory name>` → its fill types, uppercased, from a fill types XML.
pub fn parse_fill_type_categories(xml: &str) -> HashMap<String, Vec<String>> {
    let mut reader = Reader::from_str(xml.strip_prefix('\u{feff}').unwrap_or(xml));
    let mut categories = HashMap::new();
    let mut open: Option<(String, String)> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == "fillTypeCategory" => {
                open = Some((attr_str(e, "name").to_uppercase(), String::new()));
            }
            Ok(Event::Text(ref t)) => {
                if let Some((_, ref mut text)) = open {
                    text.push_str(t);
                    text.push(' ');
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == "fillTypeCategory" => {
                if let Some((name, text)) = open.take() {
                    categories.insert(name, names(&text));
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    categories
}

/// `uniqueId` → `xmlFilename` of the pre-placed placeables of a map i3d.
pub fn parse_preplaced_files(i3d: &str) -> HashMap<String, String> {
    let mut reader = Reader::from_str(i3d);
    let mut files = HashMap::new();
    let mut unique_id: Option<String> = None;
    let mut xml_filename: Option<String> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == "UserAttribute" => {
                unique_id = None;
                xml_filename = None;
            }
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) if e.name().as_ref() == "Attribute" => {
                match attr_str(e, "name").as_str() {
                    "uniqueId" => unique_id = Some(attr_str(e, "value")),
                    "xmlFilename" => xml_filename = Some(attr_str(e, "value")),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == "UserAttribute" => {
                if let (Some(id), Some(file)) = (unique_id.take(), xml_filename.take()) {
                    files.insert(id, file);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage(capacity: f64, categories: &[&str], fill_types: &[&str]) -> StorageDef {
        StorageDef {
            capacity,
            capacities: Vec::new(),
            fill_type_categories: categories.iter().map(|s| s.to_string()).collect(),
            fill_types: fill_types.iter().map(|s| s.to_string()).collect(),
            supports_multiple_fill_types: true,
        }
    }

    #[test]
    fn test_silo_storages_of_farm_silo() {
        // Shape of data/placeables/mapUS/farmSilo/farmSilo.xml (trimmed)
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="no"?>
<placeable type="silo" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <silo>
        <loadingStation node="loadingTrigger" supportsExtension="true" storageRadius="50">
            <loadingTrigger triggerNode="loadingTrigger" fillTypeCategories="farmSilo"/>
        </loadingStation>
        <storages>
            <storage node="storage" fillTypeCategories="farmSilo" capacity="200000" isExtension="true"/>
        </storages>
    </silo>
</placeable>"#;
        let defs = parse_storage_defs(xml);
        assert_eq!(defs.silo_storages, vec![storage(200000.0, &["FARMSILO"], &[])]);
        assert_eq!(defs.extension, None);
        assert_eq!(defs.object_storage_capacity, None);
    }

    #[test]
    fn test_extension_storage() {
        // Shape of data/placeables/unia/farma/farma250/farma250.xml
        let xml = r#"<placeable type="siloExtension">
    <siloExtension>
        <storage node="storage" fillTypeCategories="farmSilo" capacity="250000" isExtension="true"/>
    </siloExtension>
</placeable>"#;
        assert_eq!(parse_storage_defs(xml).extension, Some(storage(250000.0, &["FARMSILO"], &[])));
    }

    #[test]
    fn test_single_fill_type_storage_with_fill_type_capacities_and_defaults() {
        // Shape of a husbandry-style storage (cowBarn.xml), placed in a silo
        let xml = r#"<placeable><silo><storages>
            <storage node="storage" fillTypes="SEEDS mineral_feed" capacity="27300" supportsMultipleFillTypes="false" isExtension="false"/>
            <storage node="storage2">
                <capacity fillType="DIESEL" capacity="5000"/>
            </storage>
        </storages></silo></placeable>"#;
        let defs = parse_storage_defs(xml);
        let mut first = storage(27300.0, &[], &["SEEDS", "MINERAL_FEED"]);
        first.supports_multiple_fill_types = false;
        let mut second = storage(100000.0, &[], &[]);
        second.capacities = vec![("DIESEL".into(), 5000.0)];
        assert_eq!(defs.silo_storages, vec![first, second]);
    }

    #[test]
    fn test_object_storage_capacity() {
        // Shape of data/placeables/mapUS/objectStorage/objectStorage.xml
        let xml = r#"<placeable type="objectStorage">
    <objectStorage supportsBales="true" supportsPallets="true" maxLength="8.5" maxWidth="6" maxHeight="3.5">
    </objectStorage>
</placeable>"#;
        assert_eq!(parse_storage_defs(xml).object_storage_capacity, Some(250));
        let sized = r#"<placeable><objectStorage capacity="120"/></placeable>"#;
        assert_eq!(parse_storage_defs(sized).object_storage_capacity, Some(120));
    }

    #[test]
    fn test_fill_type_categories() {
        // Shape of data/maps/maps_fillTypes.xml and FS25_Pallegney's fillTypes.xml
        let xml = r#"<map>
    <fillTypes><fillType name="WHEAT"/></fillTypes>
    <fillTypeCategories>
        <fillTypeCategory name="FARMSILO">WHEAT BARLEY
            OAT</fillTypeCategory>
        <fillTypeCategory name="LIQUID" >MILK water</fillTypeCategory>
    </fillTypeCategories>
</map>"#;
        let categories = parse_fill_type_categories(xml);
        assert_eq!(categories["FARMSILO"], vec!["WHEAT", "BARLEY", "OAT"]);
        assert_eq!(categories["LIQUID"], vec!["MILK", "WATER"]);
    }

    #[test]
    fn test_preplaced_files_of_map_i3d() {
        // Shape of the UserAttribute section of data/maps/mapUS/mapUS.i3d
        let i3d = r#"<i3D><UserAttributes>
    <UserAttribute nodeId="36966">
      <Attribute name="aiSplines" type="string" value="2"/>
      <Attribute name="onCreate" type="scriptCallback" value="Placeable.onCreate"/>
      <Attribute name="uniqueId" type="string" value="preplaced_farmSilo_3f0a7c05db981d5d352333d1d157eb05"/>
      <Attribute name="xmlFilename" type="string" value="$data/placeables/mapUS/farmSilo/farmSilo.xml"/>
    </UserAttribute>
    <UserAttribute nodeId="13284">
      <Attribute name="polygonIndex" type="string" value="0"/>
    </UserAttribute>
  </UserAttributes></i3D>"#;
        let files = parse_preplaced_files(i3d);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files["preplaced_farmSilo_3f0a7c05db981d5d352333d1d157eb05"],
            "$data/placeables/mapUS/farmSilo/farmSilo.xml"
        );
    }
}
