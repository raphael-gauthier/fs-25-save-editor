use serde::{Deserialize, Serialize};

/// One animal husbandry, extracted from a `<placeable>` in placeables.xml.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Husbandry {
    /// Index of the owning placeable, using the same numbering as `parse_placeables`.
    /// This is the write target — it must stay aligned with that parser.
    pub placeable_index: usize,
    pub display_name: String,
    pub farm_id: u8,
    /// How well the husbandry is currently producing. Read-only.
    pub production_factor: f64,
    pub clusters: Vec<AnimalCluster>,
    /// `<husbandryFood><fillLevel>` — no capacity is recorded in the savegame.
    pub food: Vec<FillStock>,
    /// `<husbandry><storage><node>` — typically WATER. No capacity either.
    pub water: Vec<FillStock>,
    /// `<husbandryMeadow><fillType>` — the only stock that records a capacity.
    pub meadow: Vec<MeadowStock>,
}

/// A batch of same-age animals. `<animal>` carries no identifier, so clusters are
/// addressed by their position within `<clusters>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalCluster {
    pub index: usize,
    pub sub_type: String,
    pub num_animals: u32,
    /// Age in months.
    pub age: u32,
    pub health: f64,
    pub reproduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillStock {
    pub fill_type: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeadowStock {
    pub fill_type: String,
    pub amount: f64,
    pub capacity: f64,
}
