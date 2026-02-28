# 05 - Data Models

## Principles

- Each XML file is mapped to a Rust struct and a mirrored TypeScript type
- Rust structs implement `Serialize` + `Deserialize` (serde) for JSON transit via IPC
- TypeScript types are interfaces that exactly mirror the backend's JSON output
- Optional fields in the XML are `Option<T>` in Rust and `T | null` in TypeScript

## Shared Types

### Rust
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

### TypeScript
```typescript
interface Position {
  x: number
  y: number
  z: number
}

interface Rotation {
  x: number
  y: number
  z: number
}
```

## Savegame — Summary and Complete Data

### `SavegameSummary` — Savegame preview (initial list)

```rust
pub struct SavegameSummary {
    pub path: String,
    pub name: String,              // careerSavegame > settings > savegameName
    pub map_title: String,         // careerSavegame > settings > mapTitle
    pub money: f64,                // careerSavegame > statistics > money
    pub play_time: f64,            // In seconds
    pub save_date: String,         // careerSavegame > settings > saveDateFormatted
    pub economic_difficulty: String,
}
```

### `SavegameData` — Complete data for an opened savegame

```rust
pub struct SavegameData {
    pub path: String,
    pub career: CareerSavegame,
    pub farms: Vec<Farm>,
    pub vehicles: Vec<Vehicle>,
    pub sales: Vec<SaleItem>,
    pub fields: Vec<Field>,              // Phase 2
    pub farmlands: Vec<Farmland>,        // Phase 2
    pub environment: Environment,        // Phase 2
    pub placeables: Vec<Placeable>,      // Phase 2
    pub missions: Vec<Mission>,          // Phase 2
    pub collectibles: Vec<Collectible>,  // Phase 2
    pub contract_settings: ContractSettings, // Phase 2
    pub warnings: Vec<String>,           // Non-blocking warnings
}
```

## Career Data (`careerSavegame.xml`)

```rust
pub struct CareerSavegame {
    pub savegame_name: String,
    pub creation_date: String,
    pub map_id: String,
    pub map_title: String,
    pub save_date: String,
    pub economic_difficulty: String,     // "EASY", "NORMAL", "HARD"
    pub money: f64,
    pub play_time: f64,                  // Seconds

    // Game settings (read-only in simple mode, editable in advanced mode)
    pub growth_mode: u8,
    pub planned_days_per_period: u8,
    pub plowing_required: bool,
    pub stones_enabled: bool,
    pub weeds_enabled: bool,
    pub lime_required: bool,
    pub snow_enabled: bool,
    pub fuel_usage: u8,
    pub traffic_enabled: bool,
}
```

## Farm (`farms.xml`)

```rust
pub struct Farm {
    pub farm_id: u8,
    pub name: String,
    pub color: u8,
    pub loan: f64,
    pub money: f64,
    pub players: Vec<FarmPlayer>,
    pub statistics: FarmStatistics,
    pub daily_finances: Vec<DailyFinance>,
}

pub struct FarmPlayer {
    pub unique_user_id: String,
    pub farm_manager: bool,
    pub last_nickname: String,
    pub time_last_connected: String,
    // Permissions
    pub buy_vehicle: bool,
    pub sell_vehicle: bool,
    pub buy_placeable: bool,
    pub sell_placeable: bool,
    pub manage_contracts: bool,
    pub trade_animals: bool,
    pub create_fields: bool,
    pub landscaping: bool,
    pub hire_assistant: bool,
    pub reset_vehicle: bool,
    pub manage_productions: bool,
    pub cut_trees: bool,
    pub manage_rights: bool,
    pub transfer_money: bool,
    pub update_farm: bool,
    pub manage_contracting: bool,
}

pub struct FarmStatistics {
    pub traveled_distance: f64,
    pub fuel_usage: f64,
    pub seed_usage: f64,
    pub spray_usage: f64,
    pub worked_hectares: f64,
    pub cultivated_hectares: f64,
    pub sown_hectares: f64,
    pub sprayed_hectares: f64,
    pub threshed_hectares: f64,
    pub plowed_hectares: f64,
    pub bale_count: u32,
    pub mission_count: u32,
    pub play_time: f64,
    pub revenue: f64,
    pub expenses: f64,
    // ... (other counters)
}

pub struct DailyFinance {
    pub day: u32,
    pub new_vehicles_cost: f64,
    pub sold_vehicles: f64,
    pub new_animals_cost: f64,
    pub sold_animals: f64,
    pub construction_cost: f64,
    pub sold_buildings: f64,
    pub field_purchase: f64,
    pub sold_fields: f64,
    pub vehicle_running_cost: f64,
    pub vehicle_leasing_cost: f64,
    pub property_maintenance: f64,
    pub property_income: f64,
    pub production_costs: f64,
    pub sold_products: f64,
    pub harvest_income: f64,
    pub mission_income: f64,
    pub wage_payment: f64,
    pub loan_interest: f64,
    pub other_income: f64,
    pub other_expenses: f64,
}
```

## Vehicle (`vehicles.xml`)

```rust
pub struct Vehicle {
    pub unique_id: String,
    pub filename: String,              // Path to the definition XML
    pub display_name: String,          // Human-readable name (derived from filename)
    pub age: f64,                      // Days
    pub price: f64,
    pub farm_id: u8,
    pub property_state: PropertyState,
    pub operating_time: f64,           // Hours

    pub position: Option<Position>,    // From <component index="1">
    pub rotation: Option<Rotation>,

    pub configurations: Vec<VehicleConfiguration>,
    pub fill_units: Vec<FillUnit>,
    pub attached_implements: Vec<AttachedImplement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyState {
    None,       // Game vehicle (trains, etc.)
    Owned,
    Rented,
}

pub struct VehicleConfiguration {
    pub name: String,                  // "fillUnit", "wheel", "color"...
    pub id: String,
}

pub struct FillUnit {
    pub index: u32,
    pub fill_type: String,             // "DIESEL", "DEF", "SEEDS", "WHEAT"...
    pub fill_level: f64,
    pub capacity: Option<f64>,         // Max capacity (if available)
}

pub struct AttachedImplement {
    pub joint_index: u32,
    pub attached_vehicle_unique_id: String,
    pub move_down: bool,
}
```

## Used Market (`sales.xml`)

```rust
pub struct SaleItem {
    pub index: usize,                  // Position in the file (not an XML ID)
    pub xml_filename: String,
    pub display_name: String,          // Derived human-readable name
    pub age: u32,
    pub price: u32,
    pub damage: f64,                   // 0.0 - 1.0
    pub wear: f64,                     // 0.0 - 1.0
    pub operating_time: f64,           // Hours
    pub time_left: u32,                // Remaining days
    pub is_generated: bool,
    pub bought_configurations: Vec<BoughtConfiguration>,
}

pub struct BoughtConfiguration {
    pub name: String,
    pub id: String,
}
```

## Vehicle Catalog (scanned from game files)

```rust
pub struct CatalogVehicle {
    pub xml_filename: String,           // Relative path (base game/DLC) or $moddir$ path (mods)
    pub name: String,
    pub brand: String,
    pub category: String,
    pub price: u32,
    pub source: VehicleSource,
}

pub enum VehicleSource {
    BaseGame,
    Mod(String),                        // Mod name (zip filename without extension)
}
```

```typescript
interface CatalogVehicle {
  xmlFilename: string
  name: string
  brand: string
  category: string
  price: number
  source: "baseGame" | { mod: string }
}
```

## Sale Addition (`SaleAdditionPayload`)

```rust
pub struct SaleAdditionPayload {
    pub xml_filename: String,
    pub price: u32,
    pub damage: f64,
    pub wear: f64,
    pub age: u32,
    pub operating_time: f64,
    pub time_left: u32,
}
```

## Field (`fields.xml`) — Phase 2

```rust
pub struct Field {
    pub id: u32,
    pub planned_fruit: Option<String>,
    pub fruit_type: Option<String>,    // "WHEAT", "CANOLA", etc.
    pub growth_state: u8,              // 0-10
    pub last_growth_state: u8,
    pub weed_state: u8,                // 0-9
    pub stone_level: u8,               // 0-3
    pub spray_level: u8,
    pub lime_level: u8,                // 0-3
    pub plow_level: u8,
    pub roller_level: u8,
    pub stubble_shred_level: u8,
    pub water_level: u8,
    pub ground_type: String,           // "PLOWED", "CULTIVATED", "SOWN"...
}

pub struct Farmland {
    pub id: u32,
    pub farm_id: u8,                   // 0 = unowned
}
```

## Environment (`environment.xml`) — Phase 2

```rust
pub struct Environment {
    pub day_time: f64,                 // Time of day (seconds)
    pub current_day: u32,
    pub days_per_period: u8,
    pub weather_forecast: Vec<WeatherEvent>,
    pub snow_height: f64,
    pub ground_wetness: f64,
}

pub struct WeatherEvent {
    pub type_name: String,             // "SUN", "RAIN", "CLOUDY", "SNOW", "TWISTER"
    pub season: String,                // "SPRING", "SUMMER", "AUTUMN", "WINTER"
    pub variation_index: u8,
    pub start_day: u32,
    pub start_day_time: u64,           // Milliseconds since the start of the day
    pub duration: u64,                 // Milliseconds
}
```

## Mission (`missions.xml`) — Phase 2

```rust
pub struct Mission {
    pub unique_id: String,
    pub mission_type: String,          // "harvestMission", "plowMission", etc.
    pub status: MissionStatus,
    pub reward: f64,
    pub reimbursement: f64,
    pub completion: f64,               // 0.0 - 1.0
    pub field_id: Option<u32>,
    pub fruit_type: Option<String>,
    pub expected_liters: Option<f64>,
    pub deposited_liters: Option<f64>,
}

pub enum MissionStatus {
    Created,
    Running,
    Completed,
}
```

## Collectible (`collectibles.xml`) — Phase 2

```rust
pub struct Collectible {
    pub index: u32,
    pub collected: bool,
}
```

## Contract Settings (`r_contracts.xml`) — Phase 2

```rust
pub struct ContractSettings {
    pub lease_vehicle: f64,
    pub mission_per_farm: f64,
    pub allow_clear_add: f64,
}
```

## Change Object (`SavegameChanges`)

Structure sent by the frontend to the backend when saving. Contains **only** the modified fields.

```rust
pub struct SavegameChanges {
    pub finance: Option<FinanceChanges>,
    pub vehicles: Option<Vec<VehicleChange>>,
    pub sales: Option<Vec<SaleChange>>,
    pub sale_additions: Option<Vec<SaleAdditionPayload>>, // New vehicles added to market
    pub fields: Option<Vec<FieldChange>>,         // Phase 2
    pub environment: Option<EnvironmentChanges>,   // Phase 2
    pub missions: Option<Vec<MissionChange>>,      // Phase 2
    pub collectibles: Option<Vec<CollectibleChange>>, // Phase 2
}

pub struct FinanceChanges {
    pub money: Option<f64>,
    pub loan: Option<f64>,
    pub statistics: Option<FarmStatistics>,
}

pub struct VehicleChange {
    pub unique_id: String,
    pub delete: bool,                              // true = delete the vehicle
    pub age: Option<f64>,
    pub price: Option<f64>,
    pub farm_id: Option<u8>,
    pub property_state: Option<PropertyState>,
    pub operating_time: Option<f64>,
    pub position: Option<Position>,
    pub rotation: Option<Rotation>,
    pub fill_units: Option<Vec<FillUnitChange>>,
}

pub struct FillUnitChange {
    pub index: u32,
    pub fill_level: f64,
}

pub struct SaleChange {
    pub index: usize,
    pub delete: bool,
    pub price: Option<u32>,
    pub damage: Option<f64>,
    pub wear: Option<f64>,
    pub age: Option<u32>,
    pub operating_time: Option<f64>,
    pub time_left: Option<u32>,
}
```

## Save Result

```rust
pub struct SaveResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub files_modified: Vec<String>,
    pub errors: Vec<String>,
}

pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub created_at: String,            // ISO 8601
    pub size_bytes: u64,
}

pub struct AppSettings {
    pub locale: String,
    pub theme: String,
    pub advanced_mode: bool,
    pub default_path: Option<String>,
    pub max_backups: u32,
}
```
