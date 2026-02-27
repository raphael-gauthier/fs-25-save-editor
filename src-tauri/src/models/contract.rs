use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSettings {
    pub lease_vehicle: f64,
    pub mission_per_farm: f64,
    pub allow_clear_add: f64,
}
