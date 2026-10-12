use serde::{Deserialize, Serialize};

use super::common::Position;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placeable {
    pub index: usize,
    pub unique_id: String,
    pub filename: String,
    pub display_name: String,
    pub farm_id: u8,
    pub price: f64,
    pub age: f64,
    pub position: Option<Position>,
    pub is_pre_placed: bool,
    pub is_under_construction: bool,
    /// Construction state, present only for `<constructible>` placeables.
    /// A finished building keeps its `<constructible>` block with state "OLD" and no inputs.
    pub construction: Option<ConstructionState>,
    /// Production lines declared by a `<productionPoint>`, with their enabled flag.
    pub productions: Vec<ProductionLine>,
    /// Flat stock list of a `<productionPoint>`. The savegame does not distinguish
    /// inputs from outputs, nor record capacities — both live in the game's placeable XML.
    pub production_storage: Vec<ProductionStock>,
    /// Farm silo storages: a `<silo>`'s `<storage>` elements and a `<siloExtension>`.
    pub silo_storages: Vec<SiloStorage>,
    /// Bales and pallets in an `<objectStorage>`, in file order.
    pub stored_objects: Vec<StoredObject>,
}

/// A bale or pallet kept in an object storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredObject {
    /// "Bale" or "Vehicle" (pallets)
    pub class_name: String,
    pub filename: String,
    pub fill_type: String,
    pub fill_level: f64,
    /// 1 for a wrapped bale, 0 otherwise
    pub wrapping_state: f64,
    /// A wrapped bale still fermenting (`<fermentation isFermenting="true">`)
    pub is_fermenting: bool,
}

/// Which element of placeables.xml holds a farm silo storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SiloStorageKind {
    /// `<silo><storage index>`: a silo, one storage per index
    Silo,
    /// `<siloExtension>`: extra capacity for the farm silo
    Extension,
}

/// Capacity and accepted fill types of one farm silo storage, from its
/// placeable XML.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiloCapacity {
    pub placeable_index: usize,
    pub kind: SiloStorageKind,
    pub index: u32,
    /// Total capacity, shared by the fill types when `shared`
    pub capacity: f64,
    /// Fill types with a capacity of their own
    pub fill_type_capacities: Vec<FillTypeCapacity>,
    /// Fill types the storage accepts
    pub fill_types: Vec<String>,
    /// Whether several fill types fit at once (`supportsMultipleFillTypes`)
    pub shared: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FillTypeCapacity {
    pub fill_type: String,
    pub capacity: f64,
}

/// How many objects (bales, pallets) a placeable's object storage holds.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStorageCapacity {
    pub placeable_index: usize,
    pub capacity: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageCapacities {
    pub silos: Vec<SiloCapacity>,
    pub object_storages: Vec<ObjectStorageCapacity>,
}

/// Fill levels of one farm silo storage. Only fill types holding something
/// are saved, so a fill type the storage accepts may be missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiloStorage {
    pub kind: SiloStorageKind,
    /// `<storage index>` of a silo, 0 for an extension
    pub index: u32,
    pub farm_id: u8,
    pub stocks: Vec<ProductionStock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructionState {
    pub state_name: String,
    pub inputs: Vec<ConstructionMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstructionMaterial {
    pub fill_type: String,
    /// Amount still to be delivered. The total required is not stored in the savegame.
    pub amount_remaining: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionLine {
    pub id: String,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductionStock {
    pub fill_type: String,
    pub amount: f64,
}

pub fn placeable_display_name(filename: &str) -> String {
    let name = filename
        .rsplit('/')
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".xml");

    let mut result = String::new();
    let mut prev_was_lower = false;
    let mut prev_was_digit = false;

    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            result.push(' ');
            prev_was_lower = false;
            prev_was_digit = false;
            continue;
        }
        if ch.is_uppercase() && prev_was_lower {
            result.push(' ');
        } else if ch.is_ascii_digit() && !prev_was_digit && !result.is_empty() {
            result.push(' ');
        } else if ch.is_lowercase() && prev_was_digit {
            result.push(' ');
        }
        if result.is_empty() || result.ends_with(' ') {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
        prev_was_lower = ch.is_lowercase();
        prev_was_digit = ch.is_ascii_digit();
    }

    result
}

/// Pre-placed map objects carry no `filename`, so their name must come from the
/// `uniqueId` — e.g. `preplaced_farmHouse05_306e852a...` becomes "Farm House 05".
pub fn display_name_from_unique_id(unique_id: &str) -> String {
    let stem = unique_id.strip_prefix("preplaced_").unwrap_or(unique_id);

    // Drop the trailing 32-character hex hash the game appends to pre-placed ids.
    let stem = match stem.rsplit_once('_') {
        Some((head, tail))
            if tail.len() == 32 && tail.chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            head
        }
        _ => stem,
    };

    placeable_display_name(stem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_from_unique_id_strips_prefix_and_hash() {
        assert_eq!(
            display_name_from_unique_id("preplaced_farmHouse05_306e852a51d90726d144af50a865cbf0"),
            "Farm House 05"
        );
    }

    #[test]
    fn test_display_name_from_unique_id_without_hash() {
        assert_eq!(
            display_name_from_unique_id("sellingStationTrain01"),
            "Selling Station Train 01"
        );
    }
}
