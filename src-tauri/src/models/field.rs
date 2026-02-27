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
