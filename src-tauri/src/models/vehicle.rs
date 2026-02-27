use serde::{Deserialize, Serialize};

use super::common::{Position, Rotation};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vehicle {
    pub unique_id: String,
    pub filename: String,
    pub display_name: String,
    pub age: f64,
    pub price: f64,
    pub farm_id: u8,
    pub property_state: PropertyState,
    pub operating_time: f64,
    pub position: Option<Position>,
    pub rotation: Option<Rotation>,
    pub configurations: Vec<VehicleConfiguration>,
    pub fill_units: Vec<FillUnit>,
    pub attached_implements: Vec<AttachedImplement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyState {
    None,
    Owned,
    Rented,
}

impl PropertyState {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => PropertyState::Owned,
            2 => PropertyState::Rented,
            _ => PropertyState::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleConfiguration {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillUnit {
    pub index: u32,
    pub fill_type: String,
    pub fill_level: f64,
    pub capacity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedImplement {
    pub joint_index: u32,
    pub attached_vehicle_unique_id: String,
    pub move_down: bool,
}

/// Derives a human-readable display name from a vehicle filename path.
/// "data/vehicles/fendt/fendt942Vario/fendt942Vario.xml" → "Fendt 942 Vario"
pub fn vehicle_display_name(filename: &str) -> String {
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
        } else if ch.is_alphabetic() && prev_was_digit {
            result.push(' ');
        }
        prev_was_lower = ch.is_lowercase();
        prev_was_digit = ch.is_ascii_digit();
        result.push(ch);
    }

    // Capitalize first letter of each word
    result
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
