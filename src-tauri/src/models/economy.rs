use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Economy {
    pub great_demands: Vec<GreatDemand>,
    pub fill_types: Vec<FillTypePrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreatDemand {
    pub index: usize,
    pub unique_id: String,
    pub fill_type_name: String,
    pub demand_multiplier: f64,
    pub demand_start_day: u32,
    pub demand_start_hour: u32,
    pub demand_duration: u32,
    pub is_running: bool,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillTypePrice {
    pub fill_type: String,
    pub total_amount: Option<u64>,
    pub price_history: Vec<PeriodPrice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodPrice {
    pub period: String,
    pub price: u32,
}
