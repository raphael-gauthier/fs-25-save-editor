pub mod backup;
pub mod career;
pub mod catalog;
pub mod changes;
pub mod collectible;
pub mod common;
pub mod contract;
pub mod environment;
pub mod farm;
pub mod field;
pub mod mission;
pub mod placeable;
pub mod sale;
pub mod vehicle;

use serde::{Deserialize, Serialize};

use career::CareerSavegame;
use collectible::Collectible;
use common::LocalizedMessage;
use contract::ContractSettings;
use environment::Environment;
use farm::Farm;
use field::{Farmland, Field};
use mission::Mission;
use placeable::Placeable;
use sale::SaleItem;
use vehicle::Vehicle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavegameData {
    pub path: String,
    pub career: CareerSavegame,
    pub farms: Vec<Farm>,
    pub vehicles: Vec<Vehicle>,
    pub sales: Vec<SaleItem>,
    pub fields: Vec<Field>,
    pub farmlands: Vec<Farmland>,
    pub placeables: Vec<Placeable>,
    pub missions: Vec<Mission>,
    pub collectibles: Vec<Collectible>,
    pub contract_settings: Option<ContractSettings>,
    pub environment: Option<Environment>,
    pub warnings: Vec<LocalizedMessage>,
}
