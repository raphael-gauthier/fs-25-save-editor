# 01 - Stack & Tooling

## Desktop Runtime: Tauri v2

- **Version:** Tauri 2.x (stable)
- **Rationale:** Lightweight (~10MB vs ~150MB for Electron), native filesystem access, performant and secure Rust backend, framework-agnostic frontend (web standard)
- **Tauri plugins used:**
  - `tauri-plugin-dialog`: Native folder picker (manual savegame selection)
  - `tauri-plugin-fs`: Filesystem access (reading/writing XML files, copying directories for backups)
  - `tauri-plugin-shell`: Opening external links (help, GitHub)
  - `tauri-plugin-store`: Persisting user preferences (language, theme, advanced mode, paths)

## Rust Backend

- **Role:** All business logic — XML parsing, validation, transformation, backup, writing
- **Main crates:**
  - `quick-xml`: Performant XML parsing and writing, supports serde
  - `serde` + `serde_json`: Serialization/deserialization of structs to JSON (frontend communication)
  - `chrono`: Date handling (timestamped backups, save dates)
  - `fs_extra`: Recursive directory copying (full backups)
  - `thiserror`: Clean, typed error handling
  - `dirs`: Cross-platform system path detection (Documents, Home)

## Frontend

### Framework
- **Vue 3.5+** with Composition API and `<script setup>`
- **TypeScript** with strict mode enabled
- **Vite** as bundler (included by default with Tauri)

### UI
- **shadcn-vue** (based on Reka UI + Tailwind CSS v4)
  - Components are not installed as npm dependencies but copied into the project (`src/components/ui/`)
  - Customizable and themeable via CSS variables
- **Tailwind CSS v4**: Utility-first CSS
- **Lucide Vue**: Icons (used by shadcn-vue)

### Planned shadcn-vue Components
| Component | Usage |
|-----------|-------|
| `Sidebar` | Main navigation between sections |
| `DataTable` | Vehicle, field, and mission lists |
| `Dialog` | Confirmations (deletion, backup) |
| `Sheet` | Detailed element editing (side panel) |
| `Form` + `Input` | Editing forms |
| `Slider` | Fill levels, sliders |
| `Switch` | Toggles (advanced mode, booleans) |
| `Badge` | Statuses (owned/rented, mission state) |
| `Toast` | Notifications (success, errors) |
| `Breadcrumb` | Breadcrumb trail |
| `Tabs` | Sub-sections (e.g., simple/advanced Finances) |
| `Select` | Crop selection, weather type |
| `Card` | Save summary, stats |
| `Tooltip` | Tooltips on advanced fields |
| `Alert` | Warnings (risky modifications) |
| `Skeleton` | Data loading states |
| `DropdownMenu` | Vehicle actions, batch actions |
| `Command` | Global quick search (Ctrl+K) |

### State Management
- **Pinia** with typed modular stores
- One store per functional domain

### Internationalization
- **vue-i18n** v10+
- Separate JSON translation files per locale (`locales/fr.json`, `locales/en.json`)
- Automatic system locale detection on first launch

### Routing
- **vue-router** v4
- Routes organized by functional section

## Package Manager
- **pnpm** v9+
- Lockfile committed to the repo

## Development Tools

| Tool | Usage |
|------|-------|
| `pnpm` | Frontend dependency management |
| `cargo` | Rust dependency management |
| `Vite` | Dev server + frontend bundler |
| `ESLint` | TypeScript/Vue linting |
| `Prettier` | Code formatting |
| `rust-analyzer` | Rust LSP for IDEs |
| `clippy` | Rust linting |
| `rustfmt` | Rust formatting |

## System Prerequisites for Development

- **Node.js** 20+
- **pnpm** 9+
- **Rust** stable toolchain (via rustup)
- **Tauri CLI** (`cargo install tauri-cli`)
- OS-specific Tauri prerequisites: WebView2 (Windows), webkit2gtk (Linux)
