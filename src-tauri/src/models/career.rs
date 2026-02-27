use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavegameSummary {
    pub path: String,
    pub name: String,
    pub map_title: String,
    pub money: f64,
    pub play_time: f64,
    pub save_date: String,
    pub economic_difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CareerSavegame {
    pub savegame_name: String,
    pub creation_date: String,
    pub map_id: String,
    pub map_title: String,
    pub save_date: String,
    pub economic_difficulty: String,
    pub money: f64,
    pub play_time: f64,
    pub growth_mode: u8,
    pub planned_days_per_period: u8,
    pub plowing_required: bool,
    pub stones_enabled: bool,
    pub weeds_enabled: bool,
    pub lime_required: bool,
    pub snow_enabled: bool,
    pub fuel_usage: u8,
    pub traffic_enabled: bool,
}
