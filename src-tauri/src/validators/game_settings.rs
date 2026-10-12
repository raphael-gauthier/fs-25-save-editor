// The careerSavegame.xml <settings> the editor may change, and the values
// each accepts. Only settings whose values were checked against the game's
// scripts or real savegames are listed.

/// What values a game setting takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingKind {
    Bool,
    /// One of the listed values
    Choice(&'static [&'static str]),
    /// A whole number between the bounds
    Int(i64, i64),
    /// A number between the bounds, saved with 6 decimals as the game does
    Float(f64, f64),
    /// Non-empty text
    Text,
}

/// Editable settings and their kinds.
pub const GAME_SETTINGS: &[(&str, SettingKind)] = &[
    ("savegameName", SettingKind::Text),
    ("economicDifficulty", SettingKind::Choice(&["EASY", "NORMAL", "HARD"])),
    // Motorized.lua: 1 low, 2 medium, 3 high
    ("fuelUsage", SettingKind::Int(1, 3)),
    // Washable.lua: 1 off, 2 to 4 slower to normal dirt
    ("dirtInterval", SettingKind::Int(1, 4)),
    // Sprayer.lua: 1 none, 2 buy, 3 and up a loading station of the map
    ("helperSlurrySource", SettingKind::Int(1, 99)),
    ("helperManureSource", SettingKind::Int(1, 99)),
    ("helperBuyFuel", SettingKind::Bool),
    ("helperBuySeeds", SettingKind::Bool),
    ("helperBuyFertilizer", SettingKind::Bool),
    ("trafficEnabled", SettingKind::Bool),
    ("stopAndGoBraking", SettingKind::Bool),
    ("trailerFillLimit", SettingKind::Bool),
    ("automaticMotorStartEnabled", SettingKind::Bool),
    ("fruitDestruction", SettingKind::Bool),
    ("plowingRequiredEnabled", SettingKind::Bool),
    ("stonesEnabled", SettingKind::Bool),
    ("weedsEnabled", SettingKind::Bool),
    ("limeRequired", SettingKind::Bool),
    ("isSnowEnabled", SettingKind::Bool),
    ("timeScale", SettingKind::Float(0.5, 360.0)),
    // Minutes between autosaves
    ("autoSaveInterval", SettingKind::Float(0.0, 120.0)),
];

pub fn setting_kind(key: &str) -> Option<SettingKind> {
    GAME_SETTINGS.iter().find(|(k, _)| *k == key).map(|(_, kind)| *kind)
}

/// The text to save for `value` of setting `key`, or why it is refused.
pub fn normalize_setting(key: &str, value: &str) -> Result<String, String> {
    let kind = setting_kind(key).ok_or_else(|| format!("{} is not an editable setting", key))?;
    let value = value.trim();
    let refused = || format!("{} is not a valid value for {}", value, key);
    match kind {
        SettingKind::Bool => match value {
            "true" | "false" => Ok(value.to_string()),
            _ => Err(refused()),
        },
        SettingKind::Choice(choices) => choices
            .iter()
            .find(|c| **c == value)
            .map(|c| c.to_string())
            .ok_or_else(refused),
        SettingKind::Int(min, max) => match value.parse::<i64>() {
            Ok(n) if (min..=max).contains(&n) => Ok(n.to_string()),
            _ => Err(refused()),
        },
        SettingKind::Float(min, max) => match value.parse::<f64>() {
            Ok(n) if n.is_finite() && (min..=max).contains(&n) => Ok(format!("{:.6}", n)),
            _ => Err(refused()),
        },
        SettingKind::Text if !value.is_empty() => Ok(value.to_string()),
        SettingKind::Text => Err(refused()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accepts_valid_values_in_saved_form() {
        assert_eq!(normalize_setting("stonesEnabled", "true"), Ok("true".into()));
        assert_eq!(normalize_setting("economicDifficulty", "HARD"), Ok("HARD".into()));
        assert_eq!(normalize_setting("fuelUsage", "3"), Ok("3".into()));
        assert_eq!(normalize_setting("timeScale", "2"), Ok("2.000000".into()));
        assert_eq!(normalize_setting("autoSaveInterval", "15.5"), Ok("15.500000".into()));
        assert_eq!(normalize_setting("savegameName", "  Ferme & Fils "), Ok("Ferme & Fils".into()));
    }

    #[test]
    fn test_refuses_out_of_range_or_unknown_values() {
        assert!(normalize_setting("stonesEnabled", "yes").is_err());
        assert!(normalize_setting("economicDifficulty", "easy").is_err());
        assert!(normalize_setting("fuelUsage", "4").is_err());
        assert!(normalize_setting("dirtInterval", "0").is_err());
        assert!(normalize_setting("fuelUsage", "2.5").is_err());
        assert!(normalize_setting("timeScale", "0.1").is_err());
        assert!(normalize_setting("timeScale", "NaN").is_err());
        assert!(normalize_setting("savegameName", "   ").is_err());
        // Settings outside the list, such as revisions, are never written
        assert!(normalize_setting("densityMapRevision", "4").is_err());
        assert!(normalize_setting("growthMode", "1").is_err());
    }
}
