use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub id: u32,
    pub planned_fruit: String,
    pub fruit_type: String,
    pub growth_state: u8,
    pub last_growth_state: u8,
    pub weed_state: u8,
    pub stone_level: u8,
    pub spray_level: u8,
    pub spray_type: String,
    pub lime_level: u8,
    pub plow_level: u8,
    pub roller_level: u8,
    pub stubble_shred_level: u8,
    pub water_level: u8,
    pub ground_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Farmland {
    pub id: u32,
    pub farm_id: u8,
}

/// What buying or selling a farmland involves, from the map's definitions.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmlandInfo {
    pub id: u32,
    pub area_ha: f64,
    /// Price the game charges to buy it and pays back when selling it
    pub price: f64,
    pub npc_name: Option<String>,
    /// Owned by the farm on a new savegame with starting farmlands
    pub default_farm_property: bool,
    /// Listed on the in-game farmland screen
    pub show_on_farmlands_screen: bool,
    /// Placeables a farm built on it (map buildings follow its owner on their own)
    pub built_placeables: u32,
}
