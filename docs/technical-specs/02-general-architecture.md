# 02 - General Architecture

## Overview

```
┌──────────────────────────────────────────────────────────┐
│                     Tauri v2 Shell                        │
│                                                          │
│  ┌─────────────────────┐    ┌──────────────────────────┐ │
│  │   Frontend (WebView) │    │    Backend (Rust)         │ │
│  │                     │    │                          │ │
│  │  Vue 3 + TypeScript │    │  Tauri Commands          │ │
│  │  shadcn-vue         │◄──►│  XML Parser (quick-xml)  │ │
│  │  Pinia Stores       │IPC │  Backup Manager          │ │
│  │  vue-router         │    │  Save Validator          │ │
│  │  vue-i18n           │    │  File System Manager     │ │
│  │                     │    │                          │ │
│  └─────────────────────┘    └──────────────────────────┘ │
│                                       │                  │
│                                       ▼                  │
│                              ┌─────────────────┐         │
│                              │  Filesystem      │         │
│                              │  savegame*/      │         │
│                              │  *_backups/      │         │
│                              │  preferences     │         │
│                              └─────────────────┘         │
└──────────────────────────────────────────────────────────┘
```

## Frontend-Backend Communication

### Protocol
Tauri uses an IPC (Inter-Process Communication) system via `invoke()`. The frontend calls **Tauri commands** defined in Rust, which return results serialized as JSON.

### Data Flow

```
Frontend                          Backend
   │                                 │
   │  invoke("list_savegames")       │
   │────────────────────────────────►│
   │                                 │ Scan filesystem
   │                                 │ Parse careerSavegame.xml (preview)
   │         Vec<SavegameSummary>    │
   │◄────────────────────────────────│
   │                                 │
   │  invoke("load_savegame", path)  │
   │────────────────────────────────►│
   │                                 │ Parse all XML files
   │                                 │ Validate consistency
   │         SavegameData            │
   │◄────────────────────────────────│
   │                                 │
   │  invoke("save_changes", delta)  │
   │────────────────────────────────►│
   │                                 │ Create backup
   │                                 │ Apply modifications
   │                                 │ Write modified XML files
   │         SaveResult              │
   │◄────────────────────────────────│
```

### Principles
- The frontend **never** accesses the filesystem directly — everything goes through Tauri commands
- Data is transmitted as **JSON** between Rust and TypeScript
- Rust structs implement `Serialize` + `Deserialize` (serde) for automatic conversion
- Errors are typed on the Rust side and converted to user-readable messages on the frontend side

## Project Structure

```
fs-25-save-editor/
├── docs/
│   ├── functional-specs/        # Functional specifications
│   └── technical-specs/         # Technical specifications (this folder)
│
├── src-tauri/                   # Rust backend (Tauri)
│   ├── Cargo.toml               # Rust dependencies
│   ├── tauri.conf.json          # Tauri configuration (window, permissions, plugins)
│   ├── capabilities/            # Tauri v2 permissions
│   └── src/
│       ├── main.rs              # Tauri entry point
│       ├── lib.rs               # App setup, command registration
│       ├── commands/            # Tauri commands (API exposed to the frontend)
│       │   ├── mod.rs
│       │   ├── savegame.rs      # list_savegames, load_savegame, save_changes
│       │   ├── backup.rs        # create_backup, list_backups, restore_backup, delete_backup
│       │   └── settings.rs      # get_settings, update_settings
│       ├── models/              # Data structs (XML mapping)
│       │   ├── mod.rs
│       │   ├── career.rs        # CareerSavegame
│       │   ├── farm.rs          # Farm, FarmFinances, FarmStatistics
│       │   ├── vehicle.rs       # Vehicle, FillUnit, Configuration
│       │   ├── sale.rs          # SaleItem
│       │   ├── field.rs         # Field
│       │   ├── environment.rs   # Environment, WeatherForecast
│       │   ├── placeable.rs     # Placeable
│       │   ├── mission.rs       # Mission
│       │   ├── collectible.rs   # Collectible
│       │   └── common.rs        # Shared types (Position, Rotation...)
│       ├── parsers/             # XML parsing → structs
│       │   ├── mod.rs
│       │   ├── career.rs
│       │   ├── farm.rs
│       │   ├── vehicle.rs
│       │   ├── sale.rs
│       │   ├── field.rs
│       │   ├── environment.rs
│       │   ├── placeable.rs
│       │   ├── mission.rs
│       │   └── collectible.rs
│       ├── writers/             # Structs → XML writing
│       │   ├── mod.rs
│       │   └── (same files as parsers/)
│       ├── validators/          # Data validation and consistency checks
│       │   ├── mod.rs
│       │   └── savegame.rs
│       ├── backup/              # Backup management
│       │   ├── mod.rs
│       │   └── manager.rs
│       └── error.rs             # Centralized error types
│
├── src/                         # Vue frontend
│   ├── main.ts                  # Vue entry point
│   ├── App.vue                  # Root component
│   ├── assets/
│   │   └── css/
│   │       └── main.css         # Tailwind directives + theme CSS variables
│   ├── components/
│   │   ├── ui/                  # shadcn-vue components (copied via CLI)
│   │   ├── layout/              # Shared layouts (Sidebar, Header, Breadcrumb)
│   │   ├── savegame/            # Savegame selection components
│   │   ├── finance/             # Finance editing components
│   │   ├── vehicle/             # Vehicle editing components
│   │   ├── sale/                # Used market components
│   │   ├── field/               # Field editing components
│   │   ├── world/               # World/weather components
│   │   ├── building/            # Building components
│   │   ├── mission/             # Mission components
│   │   └── common/              # Reusable components (EditableField, QuickAction...)
│   ├── composables/             # Vue composables (hooks)
│   │   ├── useTauri.ts          # Typed wrapper around invoke()
│   │   ├── useUnsavedChanges.ts # Unsaved changes tracking
│   │   └── useAdvancedMode.ts   # Simple/advanced mode toggle
│   ├── stores/                  # Pinia stores
│   │   ├── savegame.ts          # Main store: current savegame
│   │   ├── finance.ts           # Financial data
│   │   ├── vehicle.ts           # Vehicle data
│   │   ├── sale.ts              # Used market data
│   │   ├── field.ts             # Field data
│   │   ├── world.ts             # World/weather data
│   │   ├── building.ts          # Building data
│   │   ├── mission.ts           # Mission data
│   │   ├── settings.ts          # User preferences
│   │   └── backup.ts            # Backup management
│   ├── router/
│   │   └── index.ts             # Route definitions
│   ├── locales/
│   │   ├── fr.json              # French translations
│   │   └── en.json              # English translations
│   ├── lib/
│   │   ├── types.ts             # TypeScript types (mirroring Rust structs)
│   │   ├── constants.ts         # Constants (crop types, display names...)
│   │   └── utils.ts             # Utilities (formatting, conversion)
│   └── plugins/
│       ├── i18n.ts              # vue-i18n configuration
│       └── pinia.ts             # Pinia configuration
│
├── tests/                       # Frontend tests (Vitest)
│   ├── unit/                    # Unit tests
│   └── components/              # Component tests
│
├── index.html                   # HTML entry point (Vite)
├── package.json
├── pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
├── eslint.config.js
└── components.json              # shadcn-vue config
```

## Application Lifecycle

### Startup
1. Tauri launches the native window with the WebView
2. The Vue frontend mounts
3. The `settings` store loads persisted preferences (language, theme, default path)
4. The `savegame` store invokes `list_savegames` to scan available savegames
5. The savegame selection screen is displayed

### Loading a Savegame
1. The user selects a savegame
2. The frontend invokes `load_savegame(path)`
3. The backend parses all XML files and returns a complete `SavegameData` object
4. The Pinia stores are hydrated with the received data
5. The editor interface is displayed

### Editing and Saving
1. The user modifies values in the interface
2. The Pinia stores track changes (dirty state)
3. The user clicks "Save"
4. The frontend collects all modifications and invokes `save_changes(delta)`
5. The backend creates a backup, applies the modifications, and writes the XML files
6. The frontend receives confirmation and resets the dirty state

## Error Handling

### Backend to Frontend
Rust errors are defined via an `AppError` enum (using `thiserror`):

```
AppError
├── IoError          # Filesystem error (permissions, full disk)
├── XmlParseError    # Malformed XML or unexpected structure
├── ValidationError  # Inconsistent data
├── BackupError      # Backup creation/restoration failure
└── SavegameNotFound # Invalid path or savegame not found
```

Each variant is serialized to JSON with an error code and message, then translated on the frontend side into a localized user message via vue-i18n.

## Tauri v2 Security

### Capabilities
Tauri v2 uses a capabilities system (granular permissions). The `capabilities/*.json` file declares:
- `fs:read` + `fs:write`: limited to the `FarmingSimulator2025/` folder and its subfolders
- `dialog:open`: opening the folder picker
- `store:read` + `store:write`: preference persistence
- No network access (the app is 100% offline)
