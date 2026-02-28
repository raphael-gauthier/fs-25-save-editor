# FS25 Save Editor

A desktop application for editing Farming Simulator 25 savegames. Modify finances, vehicles, fields, buildings, sales, missions, collectibles, and world settings. Create and restore backups to keep your saves safe. Available in French and English with light and dark themes.

## Features

### Savegame Management
- Automatic detection of FS25 savegames on your system
- Load, edit, and save changes back to XML savegame files
- Unsaved changes detection with navigation guards and close protection

### Finance Editor
- Edit farm money, loan amount, and financial statistics
- View and modify daily financial history

### Vehicle Manager
- Browse all vehicles with filtering and sorting
- Edit individual vehicle properties (fill levels, wear, condition)
- Batch actions on multiple vehicles

### Used Market / Sales
- View catalog price and discount percentage for each listed vehicle
- Add vehicles from the game catalog (base game, DLCs, mods) with configurable price, discount, wear, and condition
- Edit sale price, condition, and time remaining
- Quick actions: reset to new, bargain price, extend sale

### Fields & Crops
- View and edit field ownership, crop types, and growth states

### World & Environment
- Modify weather settings, current season, and time of day

### Buildings
- Browse and manage farm buildings and structures

### Missions & Collectibles
- View and edit available missions
- Track and manage collectibles

### Backup System
- Create savegame backups before editing
- Restore or delete backups at any time

### Automatic Update Check
- Checks for new versions on GitHub Releases at startup (opt-out in settings)
- Manual check button in settings
- Update dialog with changelog and download link

### Settings
- Language: French / English
- Theme: Light / Dark / System
- Advanced mode for power users

## Installation

### Download

Download the latest release from the [Releases](https://github.com/raphael-gauthier/fs-25-save-editor/releases) page.

Available for **Windows**, **macOS**, and **Linux**.

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

#### Steps

```bash
# Clone the repository
git clone https://github.com/your-username/fs-25-save-editor.git
cd fs-25-save-editor

# Install frontend dependencies
pnpm install

# Run in development mode
pnpm tauri:dev

# Build for production
pnpm tauri:build
```

The production build output will be in `src-tauri/target/release/bundle/`.

## Releasing

Releases are automated via GitHub Actions. When a version tag is pushed, the CI builds the app for all platforms and creates a GitHub Release draft with the installers attached.

### How to release a new version

1. Update the version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`:
   ```bash
   # package.json         → "version": "x.y.z"
   # src-tauri/tauri.conf.json → "version": "x.y.z"
   # src-tauri/Cargo.toml      → version = "x.y.z"
   ```

2. Commit the version bump:
   ```bash
   git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
   git commit -m "chore: bump version to x.y.z"
   ```

3. Create and push a tag:
   ```bash
   git tag vx.y.z
   git push origin main --follow-tags
   ```

4. The [Release workflow](.github/workflows/release.yml) runs automatically and:
   - Builds the app on **Windows**, **macOS** (ARM + Intel) and **Linux**
   - Creates a **draft Release** on GitHub with all installers attached

5. Go to the [Releases](https://github.com/raphael-gauthier/fs-25-save-editor/releases) page, review the draft, edit the release notes if needed, then click **Publish release**

### Release artifacts

| Platform | Files |
|---|---|
| Windows | `.msi` (MSI installer) + `.exe` (NSIS installer) |
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |
| Linux | `.deb` + `.AppImage` |

## Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | [Tauri v2](https://v2.tauri.app/) |
| Frontend | [Vue 3](https://vuejs.org/) (Composition API, `<script setup>`) + TypeScript |
| State Management | [Pinia](https://pinia.vuejs.org/) |
| Routing | [Vue Router](https://router.vuejs.org/) |
| UI Components | [shadcn-vue](https://www.shadcn-vue.com/) (new-york style) |
| Styling | [Tailwind CSS 4](https://tailwindcss.com/) |
| Icons | [Lucide](https://lucide.dev/) |
| Notifications | [Sonner](https://vue-sonner.vercel.app/) |
| Data Tables | [TanStack Table](https://tanstack.com/table) |
| i18n | [vue-i18n](https://vue-i18n.intlify.dev/) |
| Backend | [Rust](https://www.rust-lang.org/) |
| XML Parsing | [quick-xml](https://docs.rs/quick-xml/) |
| Serialization | [serde](https://serde.rs/) |
| Error Handling | [thiserror](https://docs.rs/thiserror/) |

## Architecture

```
fs-25-save-editor/
├── src/                          # Vue 3 frontend
│   ├── views/                    # Page-level components (12 views)
│   ├── components/
│   │   ├── ui/                   # shadcn-vue base components
│   │   └── layout/               # AppHeader, AppSidebar, EditorLayout
│   ├── stores/                   # Pinia stores by domain
│   ├── composables/              # Vue composables (theme, tauri, unsaved changes)
│   ├── locales/                  # i18n translations (fr.json, en.json)
│   ├── lib/                      # Types, constants, utilities
│   ├── plugins/                  # Vue plugins (i18n)
│   └── router/                   # Vue Router configuration
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri command handlers
│       ├── models/               # Data structures
│       ├── parsers/              # XML → struct parsing
│       ├── writers/              # struct → XML writing
│       ├── services/             # Business logic (catalog scanner)
│       ├── validators/           # Data validation
│       ├── backup/               # Backup manager
│       └── error.rs              # Error types
└── docs/                         # Project documentation
    ├── functional-specs/         # Functional specifications
    └── technical-specs/          # Technical architecture
```

### How It Works

1. The Rust backend scans standard FS25 save directories and lists available savegames
2. When a savegame is loaded, XML files are parsed into typed Rust structs and sent to the frontend
3. Each domain (finance, vehicles, fields, etc.) has its own Pinia store that tracks original values and changes
4. On save, only modified data is sent back to Rust, which writes the changes to the XML files
5. Backups are managed entirely by the Rust backend (copy/restore/delete)

### Tauri Commands

| Command | Description |
|---------|-------------|
| `list_savegames` | Scan and list available FS25 savegames |
| `load_savegame` | Parse and load a savegame |
| `save_changes` | Write modifications back to XML files |
| `get_vehicle_catalog` | Scan game files for the vehicle catalog (base game, DLCs, mods) |
| `list_backups` | List existing backups |
| `create_backup` | Create a backup of the current savegame |
| `restore_backup` | Restore a savegame from a backup |
| `delete_backup` | Delete an existing backup |
| `check_for_updates` | Check GitHub Releases for a newer version |

## Development

```bash
# Start dev server (frontend + backend hot reload)
pnpm tauri:dev

# Frontend only (Vite dev server on port 1420)
pnpm dev

# Type checking
pnpm lint

# Run Rust tests
pnpm test:rust

# Production build
pnpm tauri:build
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

- [Farming Simulator 25](https://www.farming-simulator.com/) by GIANTS Software
- [Tauri](https://tauri.app/) for the desktop framework
- [shadcn-vue](https://www.shadcn-vue.com/) for the UI component library
