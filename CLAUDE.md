# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FS25 Save Editor — a Tauri v2 desktop app for editing Farming Simulator 25 savegames. Vue 3 frontend with a Rust backend that handles all file I/O (XML parsing/writing, backups).

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
**UI:** shadcn-vue (new-york style) + Tailwind CSS 4 + Lucide icons + Sonner toasts
**Backend:** Tauri v2 / Rust — quick-xml for XML, serde for serialization
**Path alias:** `@/` → `./src/`

### Frontend Structure

- `src/views/` — Page-level components (12 views)
- `src/components/ui/` — shadcn-vue base components
- `src/components/layout/` — AppHeader, AppSidebar, EditorLayout
- `src/stores/` — Pinia stores by domain (finance, vehicle, sale, field, building, mission, world, savegame, backup, settings)
- `src/composables/` — useTheme, useTauri, useUnsavedChanges, useFieldTracking
- `src/plugins/i18n.ts` — vue-i18n config (legacy: false)
- `src/locales/` — fr.json, en.json
- `src/lib/types.ts` — TypeScript types mirroring Rust structs
- `src/lib/constants.ts` — Game constants (vehicle names, fruit types, seasons, etc.)
- `src/lib/utils.ts` — Formatting helpers (money, time, vehicle names, seasons)

### Backend Structure (src-tauri/src/)

- `commands/` — Tauri command handlers (savegame.rs, backup.rs)
- `models/` — Data structures (career, farm, vehicle, sale, field, environment, placeable, mission, collectible)
- `parsers/` — XML → struct parsing
- `writers/` — struct → XML writing
- `validators/` — Data validation
- `backup/` — Backup manager
- `error.rs` — Error types (thiserror)

### Routing

`/` → SavegameSelector → `/editor/` with nested routes: finance, vehicles, vehicles/:id, sales, fields, world, buildings, missions, collectibles, settings, backups

### Tauri Commands

`list_savegames`, `load_savegame`, `save_changes`, `list_backups`, `create_backup`, `restore_backup`, `delete_backup` — invoked via `useTauri` composable wrapping `@tauri-apps/api`.

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
`useTheme` composable manages light/dark/system by toggling `dark` class on `<html>`. Settings persist via `@tauri-apps/plugin-store`.

### Settings
Settings store persists to `settings.json` via Tauri plugin-store. Loaded in `App.vue` onMounted before UI renders. Access advanced mode via `settings.advancedMode` (not a separate composable).

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
