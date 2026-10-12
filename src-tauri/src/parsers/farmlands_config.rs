// A map's farmland definitions (config/farmlands.xml): the base price per
// hectare and each farmland's price scale or fixed price.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::parsers::attrs::attr_str;

/// One `<farmland>` of a map's farmlands config.
#[derive(Debug, Clone, PartialEq)]
pub struct FarmlandDef {
    pub id: u32,
    /// `priceScale`, applied to the area price (Farmland.lua defaults it to 1)
    pub price_scale: f64,
    /// `price`: a fixed price that replaces the area price
    pub fixed_price: Option<f64>,
    /// Owned by the player's farm on a new savegame with starting farmlands
    pub default_farm_property: bool,
    /// Listed on the in-game farmland screen (`showOnFarmlandsScreen`)
    pub show_on_farmlands_screen: bool,
    pub npc_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FarmlandsConfig {
    /// `<farmlands pricePerHa>`, 60000 when missing (FarmlandManager.lua)
    pub price_per_ha: f64,
    pub farmlands: Vec<FarmlandDef>,
}

impl Default for FarmlandsConfig {
    fn default() -> Self {
        FarmlandsConfig { price_per_ha: 60000.0, farmlands: Vec::new() }
    }
}

pub fn parse_farmlands_config(xml: &str) -> FarmlandsConfig {
    let mut reader = Reader::from_str(xml.strip_prefix('\u{feff}').unwrap_or(xml));
    let mut config = FarmlandsConfig::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                "farmlands" => {
                    if let Ok(price) = attr_str(e, "pricePerHa").parse() {
                        config.price_per_ha = price;
                    }
                }
                "farmland" => {
                    let Ok(id) = attr_str(e, "id").parse() else { continue };
                    let npc_name = attr_str(e, "npcName");
                    config.farmlands.push(FarmlandDef {
                        id,
                        price_scale: attr_str(e, "priceScale").parse().unwrap_or(1.0),
                        fixed_price: attr_str(e, "price").parse().ok(),
                        default_farm_property: attr_str(e, "defaultFarmProperty") == "true",
                        show_on_farmlands_screen: attr_str(e, "showOnFarmlandsScreen") != "false",
                        npc_name: (!npc_name.is_empty()).then_some(npc_name),
                    });
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape of data/maps/mapUS/config/farmlands.xml
    const MAP_US: &str = "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\" ?>
<map xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xsi:noNamespaceSchemaLocation=\"../../../../shared/xml/schema/farmlands.xsd\">
    <farmlands infoLayer=\"farmlands\" pricePerHa=\"60000\">
        <farmland id=\"1\" priceScale=\"1\" npcName=\"GRANDPA\" defaultFarmProperty=\"true\"/>
        <farmland id=\"6\" priceScale=\"1.25\" npcName=\"FORESTER\" />
        <farmland id=\"92\" showOnFarmlandsScreen=\"false\" />
    </farmlands>
</map>";

    #[test]
    fn test_farmlands_config_of_map_us() {
        let config = parse_farmlands_config(MAP_US);
        assert_eq!(config.price_per_ha, 60000.0);
        assert_eq!(config.farmlands.len(), 3);
        assert_eq!(
            config.farmlands[0],
            FarmlandDef {
                id: 1,
                price_scale: 1.0,
                fixed_price: None,
                default_farm_property: true,
                show_on_farmlands_screen: true,
                npc_name: Some("GRANDPA".into()),
            }
        );
        assert_eq!(config.farmlands[1].price_scale, 1.25);
        assert!(!config.farmlands[1].default_farm_property);
        // Missing attributes take Farmland.lua's defaults
        assert_eq!(config.farmlands[2].price_scale, 1.0);
        assert!(!config.farmlands[2].show_on_farmlands_screen);
        assert_eq!(config.farmlands[2].npc_name, None);
    }

    #[test]
    fn test_fixed_price_and_default_price_per_ha() {
        let config = parse_farmlands_config(
            r#"<map><farmlands densityMapFilename="x.png"><farmland id="3" price="12000"/></farmlands></map>"#,
        );
        assert_eq!(config.price_per_ha, 60000.0);
        assert_eq!(config.farmlands[0].fixed_price, Some(12000.0));
    }
}
