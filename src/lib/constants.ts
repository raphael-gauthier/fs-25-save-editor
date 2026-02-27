/// Common vehicle name mapping table.
export const VEHICLE_NAME_MAP: Record<string, string> = {
  // Tractors
  fendt1050Vario: "Fendt 1050 Vario",
  fendt942Vario: "Fendt 942 Vario",
  fendt939Vario: "Fendt 939 Vario",
  fendt724Vario: "Fendt 724 Vario",
  fendt516Vario: "Fendt 516 Vario",
  fendt313Vario: "Fendt 313 Vario",
  johnDeere9RX: "John Deere 9RX",
  johnDeere8R: "John Deere 8R",
  johnDeere7R: "John Deere 7R",
  johnDeere6R: "John Deere 6R",
  caseIHOptum300: "Case IH Optum 300",
  caseIHMagnum380: "Case IH Magnum 380",
  caseIHPuma185: "Case IH Puma 185",
  caseIHSteiger: "Case IH Steiger",
  masseyferguson8S: "Massey Ferguson 8S",
  masseyferguson5S: "Massey Ferguson 5S",
  newhollandT7: "New Holland T7",
  newhollandT6: "New Holland T6",
  claasAxion960: "Claas Axion 960",
  claasAxion870: "Claas Axion 870",
  deutzFahr9340: "Deutz-Fahr 9340",
  deutzFahr6185: "Deutz-Fahr 6185",
  valtraS394: "Valtra S394",
  valtraT254: "Valtra T254",
  kubotaM7172: "Kubota M7-172",

  // Harvesters
  caseIH9250: "Case IH 9250",
  caseIH1660: "Case IH 1660",
  claasLexion8900: "Claas Lexion 8900",
  claasLexion7700: "Claas Lexion 7700",
  johnDeereX9: "John Deere X9",
  johnDeereS790: "John Deere S790",
  newhollandCR1090: "New Holland CR 10.90",
  newhollandCH: "New Holland CH",

  // Forage harvesters
  claasJaguar: "Claas Jaguar",
  kroneBigX1180: "Krone BiG X 1180",
  johnDeere9000: "John Deere 9000",

  // Trailers
  krampe: "Krampe",
  krampeBandit: "Krampe Bandit",
  krampeHKD302: "Krampe HKD 302",
  kroegerAgroliner: "Kröger Agroliner",
  bergmannHTW: "Bergmann HTW",
  fliegl: "Fliegl",
  welgerDK: "Welger DK",

  // Tools
  horsch: "Horsch",
  lemken: "Lemken",
  kuhn: "Kuhn",
  amazone: "Amazone",
  poettinger: "Pöttinger",
  kverneland: "Kverneland",
  grimme: "Grimme",
}

/// Fruit types found in FS25 fields.xml.
export const FRUIT_TYPES = [
  "WHEAT", "BARLEY", "CANOLA", "OAT", "CORN", "SUNFLOWER",
  "SOYBEAN", "POTATO", "SUGARBEET", "COTTON", "SORGHUM",
  "RICE", "ONION", "CARROT", "PARSNIP", "BEETROOT", "GRASS",
  "SPINACH", "GREENBEAN", "PEA", "MAIZE", "UNKNOWN", "FALLOW",
] as const;

/// Ground types found in FS25 fields.xml.
export const GROUND_TYPES = [
  "PLOWED", "CULTIVATED", "SOWN", "PLANTED", "GRASS",
  "HARVEST_READY", "HARVEST_READY_OTHER", "RIDGE_SOWN",
] as const;

/// Maximum growth state in FS25.
export const MAX_GROWTH_STATE = 10;

/// Weather event types in FS25 environment.xml.
export const WEATHER_TYPES = ["SUN", "RAIN", "CLOUDY", "SNOW", "TWISTER"] as const;

/// Season names.
export const SEASONS = ["SPRING", "SUMMER", "AUTUMN", "WINTER"] as const;

/// Seconds in one FS25 day (24h = 86400s).
export const SECONDS_PER_DAY = 86400;

/// Vehicle type keys derived from XML path — used as i18n keys via `vehicleTypes.<key>`.
export const VEHICLE_TYPE_KEYS: string[] = [
  "tractors", "harvesters", "trailers", "tools", "cars", "trucks",
  "cutters", "forageHarvesters", "loaders", "teleHandlers", "wheelLoaders",
  "placeables", "sprayers", "mowers", "balers", "spreaders", "cultivators",
  "plows", "seeders", "weeders", "rollers", "levelers", "forklifts",
  "conveyors", "augerWagons", "mixerWagons", "animals", "pallets", "other",
]
