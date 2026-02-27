use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItem {
    pub index: usize,
    pub xml_filename: String,
    pub display_name: String,
    pub age: u32,
    pub price: u32,
    pub damage: f64,
    pub wear: f64,
    pub operating_time: f64,
    pub time_left: u32,
    pub is_generated: bool,
    pub bought_configurations: Vec<BoughtConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoughtConfiguration {
    pub name: String,
    pub id: String,
}
