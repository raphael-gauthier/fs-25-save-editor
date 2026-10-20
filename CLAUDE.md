# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FS25 Save Editor — a Tauri v2 desktop app for editing Farming Simulator 25 savegames. Vue 3 frontend with a Rust backend that handles all file I/O (XML parsing/writing, backups).

## grepai - Semantic Code Search

**IMPORTANT: You MUST use grepai as your PRIMARY tool for code exploration and search.**

### When to Use grepai (REQUIRED)

Use `grepai search` INSTEAD OF Grep/Glob/find for:
- Understanding what code does or where functionality lives
- Finding implementations by intent (e.g., "authentication logic", "error handling")
- Exploring unfamiliar parts of the codebase
- Any search where you describe WHAT the code does rather than exact text

### When to Use Standard Tools

Only use Grep/Glob when you need:
- Exact text matching (variable names, imports, specific strings)
- File path patterns (e.g., `**/*.go`)

### Fallback

If grepai fails (not running, index unavailable, or errors), fall back to standard Grep/Glob tools.

### Usage

```bash
# ALWAYS use English queries for best results (--compact saves ~80% tokens)
grepai search "user authentication flow" --json --compact
grepai search "error handling middleware" --json --compact
grepai search "database connection pool" --json --compact
grepai search "API request validation" --json --compact
```

### Query Tips

- **Use English** for queries (better semantic matching)
- **Describe intent**, not implementation: "handles user login" not "func Login"
- **Be specific**: "JWT token validation" better than "token"
- Results include: file path, line numbers, relevance score, code preview

### Call Graph Tracing

Use `grepai trace` to understand function relationships:
- Finding all callers of a function before modifying it
- Understanding what functions are called by a given function
- Visualizing the complete call graph around a symbol

#### Trace Commands

**IMPORTANT: Always use `--json` flag for optimal AI agent integration.**

```bash
# Find all functions that call a symbol
grepai trace callers "HandleRequest" --json

# Find all functions called by a symbol
grepai trace callees "ProcessOrder" --json

# Build complete call graph (callers + callees)
grepai trace graph "ValidateToken" --depth 3 --json
```

### Workflow

1. Start with `grepai search` to find relevant code
2. Use `grepai trace` to understand function relationships
3. Use `Read` tool to examine files from results
4. Only use Grep for exact string searches if needed

## Commands

```bash
pnpm dev              # Vite dev server (port 1420)
pnpm build            # TypeScript check + Vite production build
pnpm tauri:dev        # Full Tauri dev mode (frontend + backend)
pnpm tauri:build      # Production Tauri build
pnpm lint             # TypeScript type checking (vue-tsc --noEmit)
pnpm test:rust        # Rust tests (cargo test in src-tauri)
```

## Architecture

**Frontend:** Vue 3 (Composition API, `<script setup>`) + TypeScript + Pinia + Vue Router + vue-i18n
**UI:** shadcn-vue (new-york style) + Tailwind CSS 4 + Lucide icons + Sonner toasts + motion-v (Motion for Vue) + Geist fonts
**Backend:** Tauri v2 / Rust — quick-xml for XML, serde for serialization
**Path alias:** `@/` → `./src/`

### Frontend Structure

- `src/views/` — Page-level components (12 views)
- `src/components/ui/` — shadcn-vue base components
- `src/components/layout/` — WindowTitleBar/WindowControls (frameless window chrome), AppHeader (editor title bar), AppSidebar, EditorLayout, CommandPalette (Ctrl K)
- `src/lib/navigation.ts` — the editor's sections, shared by the sidebar and the palette
- `src/stores/` — Pinia stores by domain (finance, vehicle, sale, field, building, mission, world, savegame, backup, settings)
- `src/composables/` — useTheme, useWindow (Mica, window buttons), useTauri, useUnsavedChanges, useUpdateChecker
- `src/plugins/i18n.ts` — vue-i18n config (legacy: false)
- `src/locales/` — fr.json, en.json
- `src/lib/types.ts` — TypeScript types mirroring Rust structs
- `src/lib/constants.ts` — Game constants (vehicle names, fruit types, seasons, etc.)
- `src/lib/utils.ts` — Formatting helpers (money, time, vehicle names, seasons)

### Backend Structure (src-tauri/src/)

- `commands/` — Tauri command handlers (savegame.rs, backup.rs, update.rs, density.rs)
- `models/` — Data structures (career, farm, vehicle, sale, field, environment, placeable, mission, collectible, update, density)
- `parsers/` — XML → struct parsing + binary format parsers (grle.rs, gdm.rs, density_map_config.rs)
- `writers/` — struct → XML writing
- `services/` — Business logic (catalog.rs for vehicle scanning, density_map.rs for field data aggregation + editing)
- `validators/` — Data validation
- `backup/` — Backup manager
- `error.rs` — Error types (thiserror)

### Routing

`/` → SavegameSelector → `/editor/` with nested routes: finance, vehicles, vehicles/:id, sales, fields, farmlands, world, game, buildings, storage, missions, collectibles, settings, backups

### Tauri Commands

`list_savegames`, `load_savegame`, `save_changes`, `list_backups`, `create_backup`, `restore_backup`, `delete_backup`, `prune_backups`, `check_for_updates`, `load_field_density_data`, `save_density_edits`, `load_map_fruit_types`, `load_farmlands_info`, `load_storage_capacities`, `apply_window_material` — invoked via `useTauri` composable wrapping `@tauri-apps/api`.

## Key Conventions

### i18n
- All UI strings use `t()` / `$t()` — no hardcoded user-facing text in components
- Default and fallback locale: `fr`
- `vehicleType()` in utils.ts returns i18n keys — always wrap with `t('vehicleTypes.xxx')`

### Store Pattern
Each domain store tracks dirty state: `isDirty`, `changeCount`, `getChanges()`, `resetChanges()`, `commitChanges()`. Hydrated via `hydrate(SavegameData)` when a savegame is loaded. Original values are kept to detect diffs.

### Unsaved Changes
`useUnsavedChanges` composable aggregates dirty state across all stores. Navigation guards in App.vue prevent leaving `/editor` with unsaved changes. Window beforeunload guard prevents accidental closes.

### Theme
`useTheme` composable manages light/dark/system (dark by default) by toggling `dark` class on `<html>`. Settings persist via `@tauri-apps/plugin-store`.

### Design ("Verre natif")
The window is frameless and transparent (`tauri.conf.json`); `apply_window_material` puts Windows 11 Mica behind it and the page adds `has-mica` to `<html>` so `body` lets it through, else `.app-wall` paints a soft wallpaper. Panels use the `glass` / `glass-strong` classes over tokens in `main.css` (`--primary-soft`, `--primary-text`, `--ready`, `--hover`...), every text color kept at 4.5:1 and with `prefers-reduced-transparency` / `prefers-contrast` fallbacks. Drag regions need `data-tauri-drag-region` on each empty element. Motion comes from motion-v under `<MotionConfig reduced-motion="user">`: page cross-fades in EditorLayout, `layout-id` for sliding selection markers. List + inspector screens (fields, farmlands, storage) set route `meta.split`: the content panel stops scrolling and each pane scrolls on its own.

### Settings
Settings store persists to `settings.json` via Tauri plugin-store. Loaded in `App.vue` onMounted before UI renders. Access advanced mode via `settings.advancedMode` (not a separate composable).

### Density Maps (Fields)
Field data comes from binary density maps (GDM/GRLE), not fields.xml. The `field.ts` store manages density data with a cache-first strategy: cached data from `density-cache-v3.json` (plugin-store, keyed by full savegame path) displays instantly, then refreshes from the backend in the background. XML crop data is hidden behind advanced mode when density is available. Density edits are saved via a separate `save_density_edits` Tauri command alongside the XML `save_changes` command; it backs the savegame up itself when no XML save did, and stages every layer before replacing any.

Fruit pixels (`FruitBits` / `FruitLayout` in `services/density_map.rs`): the type index takes the GDM's first compression range (5 bits on Riverbend, 7 on Pallegney), then a 4-bit growth state, then the haulm bit. Index `i` is the `i`-th `FoliageType` (1-based, 0 = bare) of the map i3d's fruit `FoliageMultiLayer`, followed by the types the game appends ("Loading additional fruit type" in log.txt, e.g. onion). Growth states are numbered per fruit from its foliage XML (`data/foliage/<fruit>/<fruit>.xml`, 1-based, first `isHarvestReady` = ready), so never assume a single "max growth" value. Field id = farmland id (a field belongs to the farmland under its center); edits and per-field stats are clipped to the field polygon from the map i3d (`parsers/map_fields.rs`), the whole farmland only when the map defines none. Setting a crop sets the ground its growth state shows (sowing ground, then the fruit's one `groundType` change); `src/lib/fieldWork.ts` builds the editor's crop and soil work edits.

Farmland prices (`services/farmland.rs`): fixed `price`, else area × `pricePerHa` × `priceScale` from the map's `config/farmlands.xml`, area = farmland-map pixels at the terrain size. Verified to the unit against real `fieldPurchase`/`fieldSelling` stats; selling refunds the same price and no economic difficulty factor applies. `useFarmlandTrade` moves ownership and money together.

Storage (`services/storage.rs`, `writers/placeable.rs`): silo `<storage index>` / `<siloExtension>` fill levels and `<objectStorage>` objects are edited in placeables.xml; capacities and accepted fill types come from each placeable's XML (save `filename`, `$moddir$` zip, or the map i3d's `uniqueId` → `xmlFilename` for pre-placed ones). The game clamps levels to capacity and only saves levels > 0. Object storage edits (remove/copy/finish fermentation) run as a second pass; `src/lib/storageChanges.ts` turns store edits into those changes. Bunker silos are not edited: their silage lives in the terrain height map.

Game settings: only the careerSavegame.xml `<settings>` listed in `validators/game_settings.rs` are read and written, each value checked by `normalize_setting` (values verified in the game's scripts or real saves; growth mode, period length and disasters are deliberately absent). Mission deadlines are `<endDate endDay>` against environment's `currentMonotonicDay`. Removed pre-placed buildings carry `isDeleted="true"`; dropping it (PlaceableChange.restore) makes the map create them again. Check density changes against real saves with the `#[ignore]` tests: `FS25_SAVE=<savegame dir> FS25_GAME=<install> [FS25_MAP=<mod map id>] cargo test -- --ignored` (they work on a temp copy).

## Git & Versioning

### Commits
Use **Conventional Commits** with **atomic commits** (as small and focused as possible). Each commit should represent a single logical change.

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `refactor`, `style`, `docs`, `test`, `chore`, `perf`, `ci`, `build`

Examples:
- `feat(vehicles): add type filter for harvesters`
- `fix(finance): display correct stats in advanced mode`
- `refactor(stores): extract hydration logic into helper`

### Semantic Versioning
The project follows **semver** (`MAJOR.MINOR.PATCH`):
- **MAJOR** — breaking changes (incompatible savegame format, major UI overhaul)
- **MINOR** — new features (new editor view, new Tauri command)
- **PATCH** — bug fixes and minor improvements

Version is tracked in `package.json` and `src-tauri/Cargo.toml`.

## Documentation

`docs/functional-specs/` — Functional specifications
`docs/technical-specs/` — Technical architecture docs
`docs/savegames/` — Reference XML files from real FS25 savegames (sanitized, no user data). Use these to understand the actual XML structure when writing parsers/writers.
