use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Farm {
    pub farm_id: u8,
    pub name: String,
    pub color: u8,
    pub loan: f64,
    pub money: f64,
    pub players: Vec<FarmPlayer>,
    pub statistics: FarmStatistics,
    pub daily_finances: Vec<DailyFinance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmPlayer {
    pub unique_user_id: String,
    pub farm_manager: bool,
    pub last_nickname: String,
    pub time_last_connected: String,
    pub buy_vehicle: bool,
    pub sell_vehicle: bool,
    pub buy_placeable: bool,
    pub sell_placeable: bool,
    pub manage_contracts: bool,
    pub trade_animals: bool,
    pub create_fields: bool,
    pub landscaping: bool,
    pub hire_assistant: bool,
    pub reset_vehicle: bool,
    pub manage_productions: bool,
    pub cut_trees: bool,
    pub manage_rights: bool,
    pub transfer_money: bool,
    pub update_farm: bool,
    pub manage_contracting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FarmStatistics {
    // Distances
    pub traveled_distance: f64,
    pub tractor_distance: f64,
    pub car_distance: f64,
    pub truck_distance: f64,
    pub horse_distance: f64,
    // Consumption
    pub fuel_usage: f64,
    pub seed_usage: f64,
    pub spray_usage: f64,
    // Hectares
    pub worked_hectares: f64,
    pub cultivated_hectares: f64,
    pub sown_hectares: f64,
    pub sprayed_hectares: f64,
    pub threshed_hectares: f64,
    pub plowed_hectares: f64,
    pub harvested_grapes: f64,
    pub harvested_olives: f64,
    // Time spent (minutes)
    pub worked_time: f64,
    pub cultivated_time: f64,
    pub sown_time: f64,
    pub sprayed_time: f64,
    pub threshed_time: f64,
    pub plowed_time: f64,
    // Counts
    pub bale_count: u32,
    pub wrapped_bales: u32,
    pub sold_cotton_bales: u32,
    pub mission_count: u32,
    pub repair_vehicle_count: u32,
    pub repaint_vehicle_count: u32,
    // Animals
    pub breed_cows_count: u32,
    pub breed_sheep_count: u32,
    pub breed_pigs_count: u32,
    pub breed_chicken_count: u32,
    pub breed_horses_count: u32,
    pub breed_goats_count: u32,
    pub breed_water_buffalo_count: u32,
    pub pet_dog_count: u32,
    pub horse_jump_count: u32,
    // Trees & wood
    pub planted_tree_count: u32,
    pub cut_tree_count: u32,
    pub wood_tons_sold: f64,
    // Finance (game internal - not reliable totals)
    pub revenue: f64,
    pub expenses: f64,
    // Play time
    pub play_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyFinance {
    pub day: u32,
    pub new_vehicles_cost: f64,
    pub sold_vehicles: f64,
    pub new_animals_cost: f64,
    pub sold_animals: f64,
    pub construction_cost: f64,
    pub sold_buildings: f64,
    pub field_purchase: f64,
    pub sold_fields: f64,
    pub vehicle_running_cost: f64,
    pub vehicle_leasing_cost: f64,
    pub property_maintenance: f64,
    pub property_income: f64,
    pub production_costs: f64,
    pub sold_products: f64,
    pub harvest_income: f64,
    pub mission_income: f64,
    pub wage_payment: f64,
    pub loan_interest: f64,
    pub other_income: f64,
    pub other_expenses: f64,
}
