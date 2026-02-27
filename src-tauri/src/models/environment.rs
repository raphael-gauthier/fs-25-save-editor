use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub day_time: f64,
    pub current_day: u32,
    pub current_monotonic_day: u32,
    pub days_per_period: u8,
    pub weather_forecast: Vec<WeatherEvent>,
    pub snow_height: f64,
    pub ground_wetness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherEvent {
    pub type_name: String,
    pub season: String,
    pub variation_index: u8,
    pub start_day: u32,
    pub start_day_time: u64,
    pub duration: u64,
}
