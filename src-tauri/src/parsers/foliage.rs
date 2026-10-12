// Map foliage definitions: the order of foliage types in a map's fruit layer
// (which is the type index stored in densityMap_fruits.gdm) and the growth
// states of each fruit type, read from its foliage XML.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;

use crate::parsers::attrs::attr_str;

/// One growth state of a fruit type, from a `<foliageState>`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FruitState {
    /// Foliage state name, e.g. "greenSmall" or "harvestReady".
    pub name: String,
    pub is_harvest_ready: bool,
    /// Ready for haulm topping before harvest (potatoes, beets)
    pub is_preparable: bool,
    pub is_withered: bool,
    pub is_cut: bool,
}

/// Growth information of one fruit type, from its foliage XML.
#[derive(Debug, Clone, PartialEq)]
pub struct FruitGrowth {
    /// Uppercased fruit type name (`<fruitType name>`), e.g. "WHEAT".
    pub name: String,
    /// `shownOnMap`: false for background fruits such as meadow.
    pub shown_on_map: bool,
    /// Growth states of the first foliage layer. State `n` (as stored in the
    /// density map) is `states[n - 1]`, as FruitTypeDesc.lua numbers them.
    pub states: Vec<FruitState>,
    /// Ground type sowing leaves (`seeding#defaultSowingGroundType`, SOWN by default).
    pub sowing_ground: String,
    /// The one state whose `groundType` the field switches to, and that ground type.
    pub ground_change: Option<(u8, String)>,
}

impl FruitGrowth {
    /// First growth state flagged `isHarvestReady`.
    pub fn harvest_ready_state(&self) -> Option<u8> {
        self.states
            .iter()
            .position(|s| s.is_harvest_ready)
            .map(|i| (i + 1) as u8)
    }

    /// Ground type a field shows with this fruit in `state`: the sowing ground
    /// until the ground change state, the changed ground from then on. None
    /// for state 0 (no plant).
    pub fn ground_for_state(&self, state: u8) -> Option<&str> {
        if state == 0 {
            return None;
        }
        match &self.ground_change {
            Some((from, ground)) if state >= *from => Some(ground),
            _ => Some(&self.sowing_ground),
        }
    }
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

/// Parses a fruit type's foliage XML. Returns None when it declares no fruit
/// type. The `<fruitType>` and `<foliageLayer>` elements come in either order.
pub fn parse_fruit_growth(xml: &str) -> Option<FruitGrowth> {
    let mut reader = Reader::from_str(xml);
    let mut name: Option<String> = None;
    let mut shown_on_map = true;
    let mut sowing_ground: Option<String> = None;
    let mut states: Vec<FruitState> = Vec::new();
    let mut ground_change: Option<(u8, String)> = None;
    let mut in_layer = false;
    let mut layers_seen = 0u32;

    loop {
        let (e, is_empty) = match reader.read_event() {
            Ok(Event::Start(e)) => (e, false),
            Ok(Event::Empty(e)) => (e, true),
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == "foliageLayer" {
                    in_layer = false;
                }
                continue;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => continue,
        };

        match e.name().as_ref() {
            "fruitType" if name.is_none() => {
                name = Some(attr_str(&e, "name").to_uppercase());
                shown_on_map = attr_str(&e, "shownOnMap") != "false";
            }
            "seeding" if sowing_ground.is_none() => {
                let ground = attr_str(&e, "defaultSowingGroundType");
                if !ground.is_empty() {
                    sowing_ground = Some(ground);
                }
            }
            "foliageLayer" => {
                layers_seen += 1;
                in_layer = !is_empty && layers_seen == 1;
            }
            // Growth states are those of the first foliage layer only
            "foliageState" if in_layer => {
                states.push(FruitState {
                    name: attr_str(&e, "name"),
                    is_harvest_ready: attr_str(&e, "isHarvestReady") == "true",
                    is_preparable: attr_str(&e, "isPreparable") == "true",
                    is_withered: attr_str(&e, "isWithered") == "true",
                    is_cut: attr_str(&e, "isCut") == "true",
                });
                // The game keeps the last declared change (FruitTypeDesc.lua)
                let ground = attr_str(&e, "groundType");
                if !ground.is_empty() {
                    ground_change = Some((states.len() as u8, ground));
                }
            }
            _ => {}
        }
    }

    let name = name.filter(|n| !n.is_empty())?;
    Some(FruitGrowth {
        name,
        shown_on_map,
        states,
        sowing_ground: sowing_ground.unwrap_or_else(|| "SOWN".to_string()),
        ground_change,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shape of data/maps/mapUS/mapUS.i3d: decoBush, weed and stone have
    // layers of their own without type index channels.
    const I3D: &str = r#"<?xml version="1.0" encoding="iso-8859-1"?>
<i3D name="mapUS">
  <Scene>
    <TerrainTransformGroup name="terrain">
      <Layers>
          <FoliageMultiLayer densityMapId="639" numChannels="4" numTypeIndexChannels="0">
            <FoliageType name="decoBush" foliageXmlId="581"/>
          </FoliageMultiLayer>
          <FoliageMultiLayer densityMapId="640" numChannels="11" numTypeIndexChannels="6" compressionChannels="6">
            <FoliageType name="decoFoliage" foliageXmlId="582"/>
            <FoliageType name="meadow" foliageXmlId="584"/>
            <FoliageType name="wheat" foliageXmlId="588"/>
            <FoliageType name="sugarBeet" foliageXmlId="593"/>
          </FoliageMultiLayer>
          <FoliageMultiLayer densityMapId="641" numChannels="4" numTypeIndexChannels="0">
            <FoliageType name="weed" foliageXmlId="612"/>
          </FoliageMultiLayer>
          <FoliageMultiLayer densityMapId="642" numChannels="3" numTypeIndexChannels="0">
            <FoliageType name="stone" foliageXmlId="613"/>
          </FoliageMultiLayer>
      </Layers>
    </TerrainTransformGroup>
  </Scene>
</i3D>"#;

    #[test]
    fn test_fruit_layer_types_in_i3d_order() {
        assert_eq!(
            parse_fruit_layer_types(I3D),
            vec!["DECOFOLIAGE", "MEADOW", "WHEAT", "SUGARBEET"]
        );
    }

    // Shape of data/foliage/wheat/wheat.xml (states trimmed)
    const WHEAT: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<foliageType>
    <fruitType name="wheat" shownOnMap="true" useForFieldMissions="true">
        <harvest litersPerSqm="0.89" cutHeight="0.15" chopperType="CHOPPER_STRAW"/>
        <seeding directionSnapAngle="0" needsRolling="true" litersPerSqm="0.0308" isAvailable="true"/>
    </fruitType>
    <foliageLayer densityMapChannelOffset="0" numDensityMapChannels="4" numBlocksPerUnit="1.8">
        <foliageState name="invisible" isGrowing="true" allowsWeeding="true" allowsHoeing="true"/>
        <foliageState name="greenSmall" numBlocksPerUnit="3.2" isGrowing="true" allowsWeeding="true" allowsHoeing="true">
            <foliageShape><foliageLod/></foliageShape>
        </foliageState>
        <foliageState name="harvestReady" isGrowing="true" isHarvestReady="true" groundType="HARVEST_READY"/>
        <foliageState name="dead" isWithered="true"/>
        <foliageState name="harvested" isCut="true"/>
    </foliageLayer>
</foliageType>"#;

    #[test]
    fn test_wheat_states_are_one_based_with_flags() {
        let growth = parse_fruit_growth(WHEAT).unwrap();
        assert_eq!(growth.name, "WHEAT");
        assert!(growth.shown_on_map);
        let names: Vec<&str> = growth.states.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["invisible", "greenSmall", "harvestReady", "dead", "harvested"]);
        assert_eq!(growth.harvest_ready_state(), Some(3));
        assert!(growth.states[3].is_withered && growth.states[4].is_cut);
    }

    #[test]
    fn test_wheat_ground_is_sown_then_harvest_ready() {
        let growth = parse_fruit_growth(WHEAT).unwrap();
        assert_eq!(growth.ground_for_state(0), None);
        assert_eq!(growth.ground_for_state(2), Some("SOWN"));
        assert_eq!(growth.ground_for_state(3), Some("HARVEST_READY"));
        // The ground stays changed once harvested
        assert_eq!(growth.ground_for_state(5), Some("HARVEST_READY"));
    }

    // Shape of data/foliage/potato/potato.xml (states trimmed): harvest ready
    // only once the haulm is cut, a haulm layer with states of its own, and
    // no ground change.
    const POTATO: &str = r#"<foliageType>
    <fruitType name="potato" shownOnMap="true" useForFieldMissions="true">
        <seeding directionSnapAngle="90" needsRolling="false" litersPerSqm="0.3733" isAvailable="true" defaultSowingGroundType="RIDGE_SOWN"/>
    </fruitType>
    <foliageLayer densityMapChannelOffset="0" numDensityMapChannels="4">
        <foliageState name="invisible" isGrowing="true" allowsHoeing="true"/>
        <foliageState name="greenSmall" isGrowing="true" allowsHoeing="true"/>
        <foliageState name="harvestReady" isGrowing="true" isPreparable="true"/>
        <foliageState name="dead" isWithered="true"/>
        <foliageState name="harvested" isCut="true"/>
        <foliageState name="cutHaulm" isHarvestReady="true" isPrepared="true"/>
    </foliageLayer>
    <foliageLayer name="haulm" densityMapChannelOffset="4" numDensityMapChannels="1">
        <foliageState distanceMapLayer="2"/>
    </foliageLayer>
</foliageType>"#;

    #[test]
    fn test_potato_ignores_the_haulm_layer_and_sows_ridges() {
        let growth = parse_fruit_growth(POTATO).unwrap();
        assert_eq!(growth.states.len(), 6);
        assert_eq!(growth.harvest_ready_state(), Some(6));
        assert!(growth.states[2].is_preparable && !growth.states[5].is_preparable);
        assert_eq!(growth.ground_change, None);
        assert_eq!(growth.ground_for_state(6), Some("RIDGE_SOWN"));
    }

    // Shape of data/foliage/grass/grass.xml: the layer comes before the
    // fruit type, and the ground change is on an early state.
    const GRASS: &str = r#"<foliageType>
    <foliageLayer densityMapChannelOffset="0" numDensityMapChannels="4">
        <foliageState name="invisible" isGrowing="true" />
        <foliageState name="greenSmall" isGrowing="true" regrowthStart="true" groundType="GRASS" groundTypeMask="GRASS GRASS_CUT SOWN DIRECT_SOWN ROLLER_LINES"/>
        <foliageState name="greenMiddle" isGrowing="true" isHarvestReady="true" yieldScale="0.5"/>
        <foliageState name="harvestReady" isGrowing="true" isHarvestReady="true"/>
        <foliageState name="cut" isCut="true" isDestructedByDisaster="true" isMulched="true"/>
        <foliageState name="cutRolled" isRolledCut="true"/>
    </foliageLayer>
    <fruitType name="grass" shownOnMap="true" useForFieldMissions="false">
        <seeding directionSnapAngle="0" litersPerSqm="0.012" isAvailable="true" plantsWeed="false"/>
    </fruitType>
</foliageType>"#;

    #[test]
    fn test_grass_layer_before_fruit_type() {
        let growth = parse_fruit_growth(GRASS).unwrap();
        assert_eq!(growth.name, "GRASS");
        assert_eq!(growth.harvest_ready_state(), Some(3));
        assert_eq!(growth.ground_for_state(1), Some("SOWN"));
        assert_eq!(growth.ground_for_state(4), Some("GRASS"));
    }

    #[test]
    fn test_fruit_growth_hidden_fruit() {
        let xml = r#"<foliageType><fruitType name="meadow" shownOnMap="false"/>
            <foliageLayer><foliageState name="a"/><foliageState name="b" isHarvestReady="true"/></foliageLayer></foliageType>"#;
        let growth = parse_fruit_growth(xml).unwrap();
        assert!(!growth.shown_on_map);
        assert_eq!(growth.harvest_ready_state(), Some(2));
    }

    #[test]
    fn test_fruit_growth_without_fruit_type() {
        assert_eq!(parse_fruit_growth("<foliageType><foliageLayer/></foliageType>"), None);
    }
}
