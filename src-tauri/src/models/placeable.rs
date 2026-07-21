use serde::{Deserialize, Serialize};

use super::common::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
