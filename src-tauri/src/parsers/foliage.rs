// Map foliage definitions: the order of foliage types in a map's fruit layer
// (which is the type index stored in densityMap_fruits.gdm) and the growth
// states of each fruit type, read from its foliage XML.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::parsers::attrs::attr_str;

/// Growth information of one fruit type, from its foliage XML.
#[derive(Debug, Clone, PartialEq)]
pub struct FruitGrowth {
    /// Uppercased fruit type name (`<fruitType name>`), e.g. "WHEAT".
    pub name: String,
    /// `shownOnMap`: false for background fruits such as meadow.
    pub shown_on_map: bool,
    /// First growth state flagged `isHarvestReady` (states are numbered from 1
    /// in file order, as FruitTypeDesc.lua does).
    pub harvest_ready_state: Option<u8>,
    /// Ground type the field shows once the fruit is harvest ready: the
    /// `groundType` of the last state up to `harvest_ready_state` declaring one.
    pub harvest_ready_ground: Option<String>,
}

/// Returns the uppercased names of the foliage types in the map's fruit
/// layer, the `FoliageMultiLayer` with type index channels, in i3d order.
/// densityMap_fruits.gdm stores the type at position `i` as index `i + 1`,
/// index 0 being bare ground.
pub fn parse_fruit_layer_types(i3d: &str) -> Vec<String> {
    let mut reader = Reader::from_str(i3d);
    let mut names = Vec::new();
    let mut in_fruit_layer = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                "FoliageMultiLayer" => {
                    let type_channels = attr_str(e, "numTypeIndexChannels").parse().unwrap_or(0u32);
                    in_fruit_layer = type_channels > 0;
                }
                "FoliageType" if in_fruit_layer => {
                    names.push(attr_str(e, "name").to_uppercase());
                }
                _ => {}
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == "FoliageMultiLayer" => {
                if in_fruit_layer {
                    break;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    names
}

/// Parses a fruit type's foliage XML. Returns None when it declares no fruit type.
pub fn parse_fruit_growth(xml: &str) -> Option<FruitGrowth> {
    let mut reader = Reader::from_str(xml);
    let mut growth: Option<FruitGrowth> = None;
    let mut layer_depth = 0u32;
    let mut layers_seen = 0u32;
    let mut state = 0u8;
    let mut last_ground: Option<String> = None;

    loop {
        let (e, is_empty) = match reader.read_event() {
            Ok(Event::Start(e)) => (e, false),
            Ok(Event::Empty(e)) => (e, true),
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == "foliageLayer" && layer_depth > 0 {
                    layer_depth -= 1;
                }
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };

        match e.name().as_ref() {
            "fruitType" if growth.is_none() => {
                growth = Some(FruitGrowth {
                    name: attr_str(&e, "name").to_uppercase(),
                    shown_on_map: attr_str(&e, "shownOnMap") != "false",
                    harvest_ready_state: None,
                    harvest_ready_ground: None,
                });
            }
            "foliageLayer" => {
                layers_seen += 1;
                if !is_empty {
                    layer_depth += 1;
                }
            }
            // Growth states are those of the first foliage layer only
            "foliageState" if layer_depth > 0 && layers_seen == 1 => {
                state = state.saturating_add(1);
                let ground = attr_str(&e, "groundType");
                if let Some(ref mut g) = growth {
                    if g.harvest_ready_state.is_none() {
                        if !ground.is_empty() {
                            last_ground = Some(ground);
                        }
                        if attr_str(&e, "isHarvestReady") == "true" {
                            g.harvest_ready_state = Some(state);
                            g.harvest_ready_ground = last_ground.clone();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    growth.filter(|g| !g.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape of data/maps/mapUS/mapUS.i3d: the weed and stone layers carry no
    // type index channels, the fruit layer does.
    const I3D: &str = r#"<?xml version="1.0" encoding="iso-8859-1"?>
<i3D name="mapUS">
  <Scene>
    <TerrainTransformGroup name="terrain">
      <Layers>
        <FoliageMultiLayer densityMapId="639" numChannels="4" numTypeIndexChannels="0">
          <FoliageType name="weed" foliageXmlId="1"/>
        </FoliageMultiLayer>
        <FoliageMultiLayer densityMapId="640" numChannels="11" numTypeIndexChannels="6" compressionChannels="6">
          <FoliageType name="decoBush" foliageXmlId="2"/>
          <FoliageType name="meadow" foliageXmlId="3">
            <FoliageSubType name="x"/>
          </FoliageType>
          <FoliageType name="wheat" foliageXmlId="4"/>
          <FoliageType name="sugarBeet" foliageXmlId="5"/>
        </FoliageMultiLayer>
        <FoliageMultiLayer densityMapId="642" numChannels="3" numTypeIndexChannels="0">
          <FoliageType name="stone" foliageXmlId="6"/>
        </FoliageMultiLayer>
      </Layers>
    </TerrainTransformGroup>
  </Scene>
</i3D>"#;

    #[test]
    fn test_fruit_layer_types_in_i3d_order() {
        assert_eq!(
            parse_fruit_layer_types(I3D),
            vec!["DECOBUSH", "MEADOW", "WHEAT", "SUGARBEET"]
        );
    }

    // Shape of data/foliage/wheat/wheat.xml
    const WHEAT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<foliageType>
    <fruitType name="wheat" shownOnMap="true" useForFieldMissions="true">
        <harvest litersPerSqm="0.89"/>
    </fruitType>
    <foliageLayer densityMapChannelOffset="0" numDensityMapChannels="4">
        <foliageState name="invisible" isGrowing="true"/>
        <foliageState name="greenSmall" isGrowing="true">
            <foliageShape><foliageLod/></foliageShape>
        </foliageState>
        <foliageState name="greenMiddle" isGrowing="true"/>
        <foliageState name="harvestReady" isHarvestReady="true" groundType="HARVEST_READY"/>
        <foliageState name="dead" isWithered="true"/>
        <foliageState name="harvested" isCut="true"/>
    </foliageLayer>
</foliageType>"#;

    #[test]
    fn test_fruit_growth_harvest_ready_state_is_one_based() {
        let growth = parse_fruit_growth(WHEAT).unwrap();
        assert_eq!(growth.name, "WHEAT");
        assert!(growth.shown_on_map);
        assert_eq!(growth.harvest_ready_state, Some(4));
        assert_eq!(growth.harvest_ready_ground.as_deref(), Some("HARVEST_READY"));
    }

    // Shape of data/foliage/potato/potato.xml: harvest ready only once the
    // haulm is cut, and the haulm layer has states of its own.
    const POTATO: &str = r#"<foliageType>
    <fruitType name="potato">
    </fruitType>
    <foliageLayer densityMapChannelOffset="0" numDensityMapChannels="4">
        <foliageState name="invisible" isGrowing="true" allowsHoeing="true"/>
        <foliageState name="greenSmall" groundType="RIDGE_SOWN" isGrowing="true"/>
        <foliageState name="harvestReady" isPreparable="true"/>
        <foliageState name="dead" isWithered="true"/>
        <foliageState name="harvested" isCut="true"/>
        <foliageState name="cutHaulm" isHarvestReady="true" isPrepared="true"/>
    </foliageLayer>
    <foliageLayer name="haulm" densityMapChannelOffset="4" numDensityMapChannels="1">
        <foliageState distanceMapLayer="2"/>
    </foliageLayer>
</foliageType>"#;

    #[test]
    fn test_fruit_growth_ignores_later_layers_and_uses_last_ground_type() {
        let growth = parse_fruit_growth(POTATO).unwrap();
        assert_eq!(growth.name, "POTATO");
        assert!(growth.shown_on_map);
        assert_eq!(growth.harvest_ready_state, Some(6));
        assert_eq!(growth.harvest_ready_ground.as_deref(), Some("RIDGE_SOWN"));
    }

    #[test]
    fn test_fruit_growth_hidden_fruit() {
        let xml = r#"<foliageType><fruitType name="meadow" shownOnMap="false"/>
            <foliageLayer><foliageState name="a"/><foliageState name="b" isHarvestReady="true"/></foliageLayer></foliageType>"#;
        let growth = parse_fruit_growth(xml).unwrap();
        assert!(!growth.shown_on_map);
        assert_eq!(growth.harvest_ready_state, Some(2));
        assert_eq!(growth.harvest_ready_ground, None);
    }

    #[test]
    fn test_fruit_growth_without_fruit_type() {
        assert_eq!(parse_fruit_growth("<foliageType><foliageLayer/></foliageType>"), None);
    }
}
