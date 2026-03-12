use serde::{Deserialize, Serialize};

/// Edit payload for modifying density map data on a single farmland/field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DensityEditPayload {
    pub farmland_id: u32,
    /// Set fruit type index (0 = clear). Only if set_fruit_name is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_fruit_name: Option<String>,
    /// Growth state to set (0-31). Used together with set_fruit_name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_growth_state: Option<u8>,
    /// Set lime level (0-3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_lime_level: Option<u8>,
    /// Set spray level (0-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_spray_level: Option<u8>,
    /// Set plow level (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_plow_level: Option<u8>,
    /// Set roller level (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_roller_level: Option<u8>,
    /// Set stubble shred level (0-1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_stubble_shred_level: Option<u8>,
    /// Set ground type (0-15, see GROUND_TYPES constant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_ground_type: Option<u8>,
    /// Clear all weeds (set to 0)
    #[serde(default)]
    pub clear_weeds: bool,
    /// Clear all stones (set to 0)
    #[serde(default)]
    pub clear_stones: bool,
    /// Only modify pixels that have crops (fruit_idx > 0).
    /// Use for harvest/mow where we only want to affect crop pixels.
    #[serde(default)]
    pub crop_area_only: bool,
    /// Only modify pixels with field-like ground types (1-13).
    /// Use for plow/cultivate/hoe where the field has no crops but has
    /// field ground types. Excludes NONE(0), GRASS(14), GRASS_CUT(15).
    #[serde(default)]
    pub field_area_only: bool,
}

/// Per-field aggregated density map data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDensityData {
    pub farmland_id: u32,
    pub pixel_count: u32,
    pub dominant_fruit: Option<String>,
    /// Percentage of pixels with the dominant fruit (0.0-100.0)
    pub fruit_coverage: f32,
    pub fruit_distribution: Vec<FruitCoverage>,
    pub avg_growth_state: f32,
    pub max_growth_state: u8,
    pub lime_status: LevelDistribution,
    pub plow_status: LevelDistribution,
    pub spray_status: LevelDistribution,
    pub roller_status: LevelDistribution,
    pub stubble_shred_status: LevelDistribution,
    /// Percentage of pixels with weeds > 0
    pub weed_coverage: f32,
    pub avg_weed_level: f32,
    pub stone_coverage: f32,
    pub ground_type_distribution: Vec<GroundCoverage>,
}

/// Coverage information for a single fruit type within a field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FruitCoverage {
    pub fruit_type: String,
    /// Percentage of field covered by this fruit (0.0-100.0)
    pub percentage: f32,
    pub avg_growth: f32,
}

/// Coverage information for a ground type within a field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundCoverage {
    pub ground_type: String,
    /// Percentage of field with this ground type (0.0-100.0)
    pub percentage: f32,
}

/// Distribution of a treatment level (lime, plow, spray, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDistribution {
    pub avg_level: f32,
    pub max_level: u8,
    /// Percentage of pixels at level 0 (needs treatment)
    pub pct_at_zero: f32,
    /// Percentage of pixels at max level (fully treated)
    pub pct_at_max: f32,
}
