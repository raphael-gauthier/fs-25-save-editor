use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVehicle {
    pub xml_filename: String,
    pub name: String,
    pub brand: String,
    pub category: String,
    pub price: u32,
    pub source: VehicleSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VehicleSource {
    BaseGame,
    Mod(String),
}
