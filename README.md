# FS25 Save Editor

> [!CAUTION]
> **DISCLAIMER — USE AT YOUR OWN RISK**
>
> This tool writes directly into your Farming Simulator 25 savegame files. **The author cannot be held responsible for any savegame that is altered, corrupted or lost**, nor for any other consequence of using this software. It is provided "as is", without warranty of any kind (see the [license](LICENSE)).
>
> **It is up to you to back up your savegames** before editing them. The editor makes a backup before each save, but keep your own copies of the saves you care about.
>
> **AVERTISSEMENT — UTILISATION À VOS RISQUES ET PÉRILS**
>
> Cet outil modifie directement les fichiers de vos sauvegardes Farming Simulator 25. **L'auteur ne peut en aucun cas être tenu responsable d'une sauvegarde altérée, corrompue ou perdue**, ni de toute autre conséquence de l'utilisation de ce logiciel, fourni « en l'état », sans aucune garantie.
>
> **Il appartient à chacun de sauvegarder ses parties** avant de les modifier. L'éditeur crée une copie de sécurité avant chaque enregistrement, mais gardez vos propres copies des sauvegardes auxquelles vous tenez.

A desktop editor for **Farming Simulator 25** savegames. Change your money, vehicles, fields, crops, farmlands, silos, buildings, missions, weather and game rules, with a backup taken before every save.

It reads the same data the game does — including the binary density maps that hold the real state of every field — and writes back only what you changed. Available in French and English.

> **Not affiliated with GIANTS Software.** Farming Simulator is a trademark of GIANTS Software GmbH. Always keep the game closed while you edit a savegame, and keep backups of saves you care about.

## Highlights

- **Native-feeling desktop app** — frameless window with Windows 11 Mica, glass panels, dark theme by default (light and system themes available), smooth motion that respects your system's "reduce motion" setting.
- **List + details layout** — fields, farmlands and storage show a list with the selected item's details always open beside it; no dialogs to open and close.
- **Command palette** — <kbd>Ctrl</kbd>+<kbd>K</kbd> anywhere to jump to a section, a field or a vehicle, save, undo, or switch theme.
- **Accurate fields** — crops, growth stages and soil state come from the game's density maps, verified against real saves.
- **Safe by design** — every save first backs the savegame up; unsaved changes are tracked per section and guarded on navigation and window close.
- **Accessible** — keyboard navigation everywhere, visible focus, 4.5:1 text contrast, and fallbacks for reduced transparency and high contrast.

## Features

### Savegames and safety
- Detects your FS25 savegames automatically (or open any folder)
- Launcher with each save's map, money, play time and difficulty
- Automatic backup before every save; restore, delete or prune backups (keeps the number you choose)
- Unsaved changes counter in the title bar, discard or reload from disk at any time

### Finances
- Money and loan, farm statistics and daily financial history

### Vehicles and used market
- Browse, filter and batch-edit vehicles (fill levels, wear, damage, condition), with images from the game files
- Used market: edit listings, or add any vehicle from the game catalog (base game, DLCs and mods)

### Fields and crops
- Real crop distribution, growth stage and soil state per field from the density maps (GDM/GRLE)
- Set any crop of the map at any of its growth stages — stages are read per crop from the game's foliage files
- One-click fixes next to each soil value: lime, fertilizer, plowing, weeds, stones
- Soil work: plow, cultivate, roll, mulch stubble
- Batch actions on checked fields, per-field undo, arrow-key navigation
- Edits stay inside the field's own polygon from the map
- Works with base-game maps and modded maps (zip archives)

### Farmlands
- Buy and sell parcels at the exact price the game charges (area × price per hectare × parcel factor)
- Combined operations: sell and buy several parcels at once, with the money left afterwards
- Parcels with buildings on them can't be sold, just like in the game

### Storage
- Silos and silo extensions: set, fill or empty each product, add any product the building accepts — capacities come from each building's own files, mods included
- Bale and pallet storage: add, copy or remove objects, finish fermentation of wrapped silage

### Buildings, animals and world
- Buildings: manage your placeables and restore pre-placed map buildings you demolished
- Husbandries: animals per building
- World and weather: time of day and weather forecast
- Economy: market prices and great demands

### Missions and game rules
- Missions and contracts, including their deadlines
- Collectibles
- Game settings stored in the savegame: savegame name, economic difficulty, fuel usage, traffic, plowing/lime/weeds/stones, crop destruction, snow, vehicle dirt, helpers' purchases, time scale, autosave interval

### App
- French and English, dark / light / system theme, advanced mode for the raw XML values
- Updates itself: new versions are checked at startup (can be turned off), downloaded, verified against the release signature and installed in one click

## Requirements

- **Windows 10 or 11**, 64-bit (Mica needs Windows 11; Windows 10 gets a plain background)
- **Farming Simulator 25** installed: the editor reads the game files for field data, crop stages, farmland prices, storage capacities and vehicle images. Point it to the game folder in Settings if it isn't detected.

macOS and Linux are not supported yet.

## Installation

Download `FS25 Save Editor_x.y.z_x64-setup.exe` from the latest [release](https://github.com/raphael-gauthier/fs-25-save-editor/releases/latest) and run it. It installs for your user account, no administrator rights needed; later versions install themselves from inside the app.

The installer is not code-signed, so Windows SmartScreen may warn about an unrecognized app: choose **More info**, then **Run anyway**.

## Usage

1. Close Farming Simulator 25.
2. Open the editor and pick a savegame.
3. Make your changes; the title bar counts what is pending.
4. Press <kbd>Ctrl</kbd>+<kbd>S</kbd> (a backup is taken first).
5. Start the game and load the save.

### Keyboard shortcuts

| Shortcut | Action |
|---|---|
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Command palette |
| <kbd>Ctrl</kbd>+<kbd>S</kbd> | Save changes |
| <kbd>Ctrl</kbd>+<kbd>Z</kbd> | Discard all pending changes (asks first) |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Move through lists (fields, saves, palette) |

## Build from source

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS) and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Steps

```bash
git clone https://github.com/raphael-gauthier/fs-25-save-editor.git
cd fs-25-save-editor
pnpm install
pnpm tauri:dev      # run in development mode
pnpm tauri:build    # build installers
```

Installers are written to `src-tauri/target/release/bundle/`.

## Development

```bash
pnpm tauri:dev      # app with hot reload (frontend + backend)
pnpm dev            # frontend only, Vite on port 1420
pnpm lint           # TypeScript type check (vue-tsc)
pnpm build          # type check + production frontend build
pnpm test:rust      # Rust tests
```

Some Rust tests run against a real savegame and are ignored by default. They always work on a temporary copy:

```bash
cd src-tauri
FS25_SAVE="<savegame folder>" FS25_GAME="<game install folder>" cargo test -- --ignored
# add FS25_MAP=<mod map id> for a modded map
```

## Releasing

Releases are built by GitHub Actions when a version tag is pushed. The workflow type-checks, runs the Rust tests, builds the Windows installer and signs the update bundle the in-app updater verifies.

**One-time setup:** add two repository secrets (Settings → Secrets and variables → Actions):

| Secret | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | content of the updater private key (`~/.tauri/fs25-save-editor.key`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | its password (`~/.tauri/fs25-save-editor.key.password`) |

The matching public key is in `src-tauri/tauri.conf.json`. Keep the private key safe: without it, installed apps can't receive updates.

**Each release:**

1. Bump the version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, and commit it.
2. Tag and push:
   ```bash
   git tag vX.Y.Z
   git push origin main --follow-tags
   ```
3. The [Release workflow](.github/workflows/release.yml) creates a **draft release** with `FS25 Save Editor_X.Y.Z_x64-setup.exe`, its signature and `latest.json`.
4. Review the draft on the [Releases](https://github.com/raphael-gauthier/fs-25-save-editor/releases) page and publish it. Installed apps only see published releases.

## Tech stack

| Layer | Technology |
|---|---|
| Desktop framework | [Tauri v2](https://v2.tauri.app/), [window-vibrancy](https://github.com/tauri-apps/window-vibrancy) for Mica |
| Frontend | [Vue 3](https://vuejs.org/) (`<script setup>`) + TypeScript, [Pinia](https://pinia.vuejs.org/), [Vue Router](https://router.vuejs.org/), [vue-i18n](https://vue-i18n.intlify.dev/) |
| UI | [shadcn-vue](https://www.shadcn-vue.com/) + [Tailwind CSS 4](https://tailwindcss.com/), [Lucide](https://lucide.dev/) icons, [Sonner](https://vue-sonner.vercel.app/) toasts |
| Motion and type | [Motion for Vue](https://motion.dev/docs/vue), [Geist](https://vercel.com/font) fonts |
| Backend | [Rust](https://www.rust-lang.org/), [quick-xml](https://docs.rs/quick-xml/), [serde](https://serde.rs/), [thiserror](https://docs.rs/thiserror/) |

## Architecture

```
fs-25-save-editor/
├── src/                      # Vue 3 frontend
│   ├── views/                # One view per editor section, plus the launcher
│   ├── components/
│   │   ├── layout/           # Window title bar and buttons, sidebar, editor layout, command palette
│   │   ├── fields/ …         # Section components (field inspector, building, mission editors…)
│   │   └── ui/               # shadcn-vue base components
│   ├── stores/               # Pinia store per domain, each tracking its own changes
│   ├── composables/          # Theme, window (Mica), Tauri calls, unsaved changes, updates
│   ├── lib/                  # Types mirroring Rust, constants, helpers, navigation
│   └── locales/              # fr.json, en.json
├── src-tauri/src/            # Rust backend
│   ├── commands/             # Tauri commands
│   ├── parsers/              # XML and binary (GDM/GRLE) readers, map and game file readers
│   ├── writers/              # Changes written back to XML
│   ├── services/             # Density maps, farmland prices, storage capacities, vehicle catalog
│   ├── validators/           # Value checks (game settings…)
│   ├── backup/               # Backup manager
│   └── models/               # Data structures
└── docs/                     # Specs and sanitized reference savegame files
```

### How it works

1. The backend lists the savegames in the FS25 profile folder.
2. Loading a savegame parses its XML files into typed Rust structs sent to the frontend; field data is read from the density maps in the background (and cached for instant display next time).
3. Each section has a Pinia store that keeps the original values and tracks changes.
4. Saving backs the savegame up, sends only the changes to Rust, writes the XML files, then the density map edits (all layers are staged before any file is replaced).

### Field data

The game keeps fields in two places: `fields.xml` holds summaries for AI and missions, while the binary density maps (`.gdm`, `.grle`) hold the real per-pixel state. The editor reads and writes the density maps:

- **GDM** (chunked, palette-compressed): fruits, ground, weeds, stones
- **GRLE** (run-length encoded): lime, plow, spray, roller, stubble and farmland info layers

A fruit pixel stores its type index, a growth state and a haulm bit. The type index follows the order of the fruit foliage layers in the map's i3d, followed by the types the game appends at load (from `log.txt`). Growth states are numbered per crop from its foliage file, so "ready to harvest" differs from crop to crop. A field belongs to the farmland under its center and edits are clipped to the field polygon from the map.

### Tauri commands

| Command | Purpose |
|---|---|
| `list_savegames`, `load_savegame`, `save_changes` | List, load and save savegames |
| `list_backups`, `create_backup`, `restore_backup`, `delete_backup`, `prune_backups`, `open_backups_folder` | Backups |
| `load_field_density_data`, `save_density_edits`, `load_map_fruit_types` | Field density maps and the map's crops |
| `load_farmlands_info` | Farmland areas and prices |
| `load_storage_capacities` | Silo and object storage capacities and accepted products |
| `get_vehicle_catalog`, `detect_game_path`, `get_vehicle_images_batch`, `clear_image_cache`, `get_image_cache_size` | Game catalog and vehicle images |
| `check_for_updates` | Newer release on GitHub |
| `apply_window_material` | Windows 11 Mica behind the window |

## Contributing

Contributions are welcome: fork the repository, create a branch, and open a pull request. The project uses [Conventional Commits](https://www.conventionalcommits.org/).

## License

[MIT](LICENSE)

## Acknowledgments

- [Farming Simulator 25](https://www.farming-simulator.com/) by GIANTS Software
- [Tauri](https://tauri.app/), [shadcn-vue](https://www.shadcn-vue.com/) and [Motion](https://motion.dev/)
