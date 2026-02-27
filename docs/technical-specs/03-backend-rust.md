# 03 - Rust Backend

## Responsibilities

The Rust backend is responsible for:
1. **Detection** of savegames on the filesystem
2. **Parsing** XML files into typed structs
3. **Validation** of data consistency
4. **Backup** (full copy) before any modification
5. **Writing** modifications back to XML files
6. **Exposing** these capabilities via Tauri commands

## Tauri Commands

### Savegame Management

#### `list_savegames`
- **Parameters:** `custom_path: Option<String>`
- **Returns:** `Vec<SavegameSummary>`
- **Behavior:**
  - If `custom_path` is provided, scans that directory
  - Otherwise, detects the default path based on the OS via the `dirs` crate
  - For each `savegame*` folder, parses only `careerSavegame.xml` to extract the summary
  - Returns the list sorted by modification date in descending order

#### `load_savegame`
- **Parameters:** `path: String`
- **Returns:** `SavegameData`
- **Behavior:**
  - Verifies the directory exists and contains the expected files
  - Parses all XML files within scope (depending on the phase)
  - Validates cross-file consistency
  - Returns a `SavegameData` object containing all structured data
  - If a file is missing/corrupted: returns available data with a `warnings: Vec<String>` field

#### `save_changes`
- **Parameters:** `path: String`, `changes: SavegameChanges`
- **Returns:** `SaveResult`
- **Behavior:**
  1. Creates a backup via the backup module
  2. Re-reads the original XML files (to only modify changed fields and preserve the rest)
  3. Applies `SavegameChanges` modifications to the structs
  4. Synchronizes duplicated values (e.g., money in career + farms)
  5. Serializes the modified structs to XML
  6. Writes only the modified files
  7. Returns the result (success + backup path, or error)

### Backup Management

#### `list_backups`
- **Parameters:** `savegame_path: String`
- **Returns:** `Vec<BackupInfo>`
- **Behavior:** Lists existing backups with date and disk size

#### `create_backup`
- **Parameters:** `savegame_path: String`
- **Returns:** `BackupInfo`
- **Behavior:** Recursive directory copy, timestamp in the name

#### `restore_backup`
- **Parameters:** `savegame_path: String`, `backup_name: String`
- **Returns:** `Result<()>`
- **Behavior:** Replaces the savegame directory contents with the backup

#### `delete_backup`
- **Parameters:** `savegame_path: String`, `backup_name: String`
- **Returns:** `Result<()>`

### Settings

#### `get_settings`
- **Returns:** `AppSettings`
- **Source:** `tauri-plugin-store` plugin

#### `update_settings`
- **Parameters:** `settings: AppSettings`
- **Returns:** `Result<()>`

## Rust Modules

### `parsers/` — XML to Struct Parsing

Each parser follows the same pattern:

```
pub fn parse_career(path: &Path) -> Result<CareerSavegame, AppError>
pub fn parse_farms(path: &Path) -> Result<Vec<Farm>, AppError>
pub fn parse_vehicles(path: &Path) -> Result<Vec<Vehicle>, AppError>
...
```

**Parsing strategy:**
- Uses `quick-xml` with `serde` (`Deserialize` derive) when the XML structure is regular
- Manual parsing with `quick-xml::Reader` for complex or conditional structures (e.g., vehicles with variable child elements depending on type)
- Unknown or unmapped fields are **ignored** (forward-compatible with game updates)
- Optional fields use `Option<T>`: a missing field does not cause an error

### `writers/` — Struct to XML Writing

**Writing strategy:**
- **Patch** approach: the entire XML is not rewritten from scratch
- The writer re-reads the original file, modifies only the attributes/elements affected by the changes, and rewrites the file
- This preserves: XML comments, element ordering, unmapped attributes, mod compatibility
- UTF-8 encoding and the `<?xml version="1.0" encoding="utf-8" standalone="no"?>` header are preserved

### `validators/` — Data Validation

Checks performed:
- `money` in `careerSavegame.xml` == `money` in `farms.xml` (warning if different)
- `farmId` values referenced in `vehicles.xml` exist in `farms.xml`
- Attachment `uniqueId` values (`attachedVehicleUniqueId`) reference existing vehicles
- Numeric values are within reasonable ranges (price >= 0, levels 0-1 for wear/damage)

### `backup/` — Backup Management

```
backup/
└── manager.rs
    ├── create_backup(savegame_path) → BackupInfo
    ├── list_backups(savegame_path) → Vec<BackupInfo>
    ├── restore_backup(savegame_path, backup_name) → Result<()>
    ├── delete_backup(savegame_path, backup_name) → Result<()>
    └── cleanup_old_backups(savegame_path, max_count) → Result<()>
```

- Backups are stored in `{savegame_path}_backups/`
- Naming convention: `backup_YYYY-MM-DD_HHhMMmSSs/`
- The copy is recursive and includes **all** files (XML + binaries) to ensure a complete restoration
- `cleanup_old_backups` flags backups beyond the limit but does not delete them automatically

### `error.rs` — Error Handling

```rust
#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    #[error("File access error: {message}")]
    IoError { message: String },

    #[error("XML parsing error: {file} - {message}")]
    XmlParseError { file: String, message: String },

    #[error("Inconsistent data: {message}")]
    ValidationError { message: String },

    #[error("Backup error: {message}")]
    BackupError { message: String },

    #[error("Savegame not found: {path}")]
    SavegameNotFound { path: String },
}
```

The enum implements `Into<tauri::InvokeError>` so it can be returned directly by Tauri commands.

## Data Integrity Preservation

### Core Principles
1. **Never lose data**: patch-based XML writing ensures that elements not understood by the editor are preserved
2. **Backup before writing**: no write operation without a successful prior backup
3. **Atomic writing**: write to a temporary file then rename, to prevent corruption in case of a crash
4. **Unmodified files**: only files containing changes are rewritten
