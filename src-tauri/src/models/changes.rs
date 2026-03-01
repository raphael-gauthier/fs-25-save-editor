use serde::{Deserialize, Serialize};

use super::common::LocalizedMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavegameChanges {
    pub finance: Option<FinanceChanges>,
    pub vehicles: Option<Vec<VehicleChange>>,
    pub sales: Option<Vec<SaleChange>>,
    pub sale_additions: Option<Vec<SaleAddition>>,
    pub fields: Option<Vec<FieldChange>>,
    pub farmlands: Option<Vec<FarmlandChange>>,
    pub placeables: Option<Vec<PlaceableChange>>,
    pub missions: Option<Vec<MissionChange>>,
    pub collectibles: Option<Vec<CollectibleChange>>,
    pub contract_settings: Option<ContractSettingsChange>,
    pub environment: Option<EnvironmentChanges>,
    pub economy: Option<EconomyChanges>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceChanges {
    pub money: Option<f64>,
    pub loan: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleChange {
    pub unique_id: String,
    pub delete: bool,
    pub age: Option<f64>,
    pub price: Option<f64>,
    pub farm_id: Option<u8>,
    pub property_state: Option<String>,
    pub operating_time: Option<f64>,
    pub damage: Option<f64>,
    pub wear: Option<f64>,
    pub fill_units: Option<Vec<FillUnitChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillUnitChange {
    pub index: u32,
    pub fill_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleChange {
    pub index: usize,
    pub delete: bool,
    pub price: Option<u32>,
    pub damage: Option<f64>,
    pub wear: Option<f64>,
    pub age: Option<u32>,
    pub operating_time: Option<f64>,
    pub time_left: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleAddition {
    pub xml_filename: String,
    pub price: u32,
    pub damage: f64,
    pub wear: f64,
    pub age: u32,
    pub operating_time: f64,
    pub time_left: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldChange {
    pub id: u32,
    pub fruit_type: Option<String>,
    pub planned_fruit: Option<String>,
    pub growth_state: Option<u8>,
    pub ground_type: Option<String>,
    pub weed_state: Option<u8>,
    pub stone_level: Option<u8>,
    pub spray_level: Option<u8>,
    pub spray_type: Option<String>,
    pub lime_level: Option<u8>,
    pub plow_level: Option<u8>,
    pub roller_level: Option<u8>,
    pub stubble_shred_level: Option<u8>,
    pub water_level: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmlandChange {
    pub id: u32,
    pub farm_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentChanges {
    pub day_time: Option<f64>,
    pub current_day: Option<u32>,
    pub snow_height: Option<f64>,
    pub ground_wetness: Option<f64>,
    pub weather_forecast: Option<Vec<crate::models::environment::WeatherEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceableChange {
    pub index: usize,
    pub farm_id: Option<u8>,
    pub price: Option<f64>,
    pub complete_construction: bool,
    pub production_inputs: Option<Vec<ProductionStockChange>>,
    pub production_outputs: Option<Vec<ProductionStockChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStockChange {
    pub fill_type: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionChange {
    pub unique_id: String,
    pub reward: Option<f64>,
    pub completion: Option<f64>,
    pub status: Option<String>,
    pub reimbursement: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectibleChange {
    pub index: u32,
    pub collected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSettingsChange {
    pub lease_vehicle: Option<f64>,
    pub mission_per_farm: Option<f64>,
    pub allow_clear_add: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreatDemandChange {
    pub index: usize,
    pub fill_type_name: Option<String>,
    pub demand_multiplier: Option<f64>,
    pub demand_start_day: Option<u32>,
    pub demand_start_hour: Option<u32>,
    pub demand_duration: Option<u32>,
    pub is_running: Option<bool>,
    pub is_valid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GreatDemandAddition {
    pub unique_id: String,
    pub fill_type_name: String,
    pub demand_multiplier: f64,
    pub demand_start_day: u32,
    pub demand_start_hour: u32,
    pub demand_duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomyChanges {
    pub great_demand_changes: Option<Vec<GreatDemandChange>>,
    pub great_demand_additions: Option<Vec<GreatDemandAddition>>,
    pub great_demand_deletions: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub files_modified: Vec<String>,
    pub errors: Vec<LocalizedMessage>,
}
