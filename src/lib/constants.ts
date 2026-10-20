/// Base-game fruit types as fields.xml names them (data/foliage), plus the
/// UNKNOWN and FALLOW placeholders. Map mods can add their own.
export const FRUIT_TYPES = [
  "WHEAT", "BARLEY", "CANOLA", "OAT", "MAIZE", "SUNFLOWER", "SOYBEAN",
  "POTATO", "SUGARBEET", "SUGARCANE", "COTTON", "SORGHUM", "RICE",
  "RICELONGGRAIN", "ONION", "CARROT", "PARSNIP", "BEETROOT", "GRASS",
  "SPINACH", "GREENBEAN", "PEA", "OILSEEDRADISH", "POPLAR", "GRAPE",
  "OLIVE", "MEADOW", "UNKNOWN", "FALLOW",
] as const;

/// Ground types of the ground density map (data/maps/maps_fieldGround.xml), NONE excluded.
export const GROUND_TYPES = [
  "STUBBLE_TILLAGE", "CULTIVATED", "SEEDBED", "PLOWED", "ROLLED_SEEDBED",
  "RIDGE", "SOWN", "DIRECT_SOWN", "PLANTED", "RIDGE_SOWN", "ROLLER_LINES",
  "HARVEST_READY", "HARVEST_READY_OTHER", "GRASS", "GRASS_CUT",
] as const;

/// Highest growth state: the fruits density map stores it in 4 bits.
export const MAX_GROWTH_STATE = 15;

/// Growth state each base-game fruit is ready to harvest in: its first
/// foliage state with isHarvestReady in data/foliage/<fruit>/<fruit>.xml.
/// Growth states are numbered per fruit, so "ready" is not a single value.
export const HARVEST_READY_STATES: Readonly<Record<string, number>> = {
  BARLEY: 8, BEETROOT: 5, CANOLA: 9, CARROT: 5, COTTON: 9, GRAPE: 10,
  GRASS: 3, GREENBEAN: 5, MAIZE: 7, MEADOW: 3, OAT: 5, OILSEEDRADISH: 2,
  OLIVE: 9, ONION: 6, PARSNIP: 5, PEA: 5, POPLAR: 14, POTATO: 9,
  RICE: 5, RICELONGGRAIN: 6, SORGHUM: 5, SOYBEAN: 7, SPINACH: 6,
  SUGARBEET: 11, SUGARCANE: 11, SUNFLOWER: 8, WHEAT: 8,
};

/// Weather event types in FS25 environment.xml.
export const WEATHER_TYPES = ["SUN", "RAIN", "CLOUDY", "SNOW", "TWISTER"] as const;
