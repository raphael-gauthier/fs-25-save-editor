# 04 - Vue Frontend

## View Structure and Routing

### Routes

```typescript
const routes = [
  {
    path: '/',
    component: SavegameSelector,     // Savegame selection screen
  },
  {
    path: '/editor',
    component: EditorLayout,          // Layout with sidebar
    children: [
      { path: 'finance',    component: FinanceView },
      { path: 'vehicles',   component: VehicleListView },
      { path: 'vehicles/:id', component: VehicleDetailView },
      { path: 'sales',      component: SaleListView },
      { path: 'sales/add',  component: SaleAddView },
      { path: 'fields',     component: FieldListView },      // Phase 2
      { path: 'world',      component: WorldView },           // Phase 2
      { path: 'buildings',  component: BuildingListView },    // Phase 2
      { path: 'missions',   component: MissionListView },     // Phase 2
      { path: 'collectibles', component: CollectibleView },   // Phase 2
      { path: 'settings',   component: SettingsView },
      { path: 'backups',    component: BackupView },
    ],
  },
]
```

### Main Layout (`EditorLayout`)

```
┌──────────────────────────────────────────────────────┐
│  ┌──────────┐  ┌──────────────────────────────────┐  │
│  │           │  │ Breadcrumb              [Save]   │  │
│  │  Sidebar  │  ├──────────────────────────────────┤  │
│  │           │  │                                  │  │
│  │ Finances  │  │         Main content             │  │
│  │ Vehicles  │  │                                  │  │
│  │ Used Mkt  │  │         (routed view)            │  │
│  │ ───────── │  │                                  │  │
│  │ Fields    │  │                                  │  │
│  │ World     │  │                                  │  │
│  │ Buildings │  │                                  │  │
│  │ Missions  │  │                                  │  │
│  │ ───────── │  │                                  │  │
│  │ Backups   │  │                                  │  │
│  │ Settings  │  │                                  │  │
│  │           │  │                                  │  │
│  │ [Simple]  │  │                                  │  │
│  │ [Advanced]│  └──────────────────────────────────┘  │
│  └──────────┘                                        │
└──────────────────────────────────────────────────────┘
```

- The sidebar uses the shadcn-vue `Sidebar` component
- Phase 2+ sections are displayed grayed out with a "Coming Soon" badge
- The simple/advanced mode toggle is at the bottom of the sidebar
- The "Save" button is in the header, with a badge showing the number of pending changes

## Pinia Stores

### `savegame.ts` — Main Store

```typescript
interface SavegameStore {
  // State
  savegames: SavegameSummary[]       // List of detected savegames
  currentPath: string | null          // Path of the opened savegame
  currentSavegame: SavegameData | null // Complete loaded data
  isLoading: boolean
  warnings: string[]                  // Parsing warnings

  // Actions
  listSavegames(customPath?: string): Promise<void>
  loadSavegame(path: string): Promise<void>
  saveChanges(): Promise<SaveResult>
  reloadFromDisk(): Promise<void>
  closeSavegame(): void
}
```

### `finance.ts` — Finances

```typescript
interface FinanceStore {
  // State (derived from savegame.currentSavegame)
  money: number
  loan: number
  statistics: FarmStatistics
  dailyFinances: DailyFinance[]

  // Dirty tracking
  originalMoney: number
  originalLoan: number
  isDirty: boolean

  // Actions
  setMoney(value: number): void
  setLoan(value: number): void
  addMoney(amount: number): void
  repayLoan(deductFromMoney: boolean): void
  resetChanges(): void
  getChanges(): FinanceChanges | null   // Returns null if no changes
}
```

### `vehicle.ts` — Vehicles

```typescript
interface VehicleStore {
  // State
  vehicles: Vehicle[]
  selectedVehicleIds: string[]

  // Filters
  searchQuery: string
  typeFilter: string | null
  propertyStateFilter: string | null

  // Computed
  filteredVehicles: Vehicle[]           // Filtered vehicles
  playerVehicles: Vehicle[]             // Player-owned vehicles only (farmId > 0)
  isDirty: boolean

  // Actions
  updateVehicle(id: string, changes: Partial<Vehicle>): void
  updateFillLevel(vehicleId: string, unitIndex: number, level: number): void
  fillAllTanks(vehicleId: string): void
  emptyAllTanks(vehicleId: string): void
  resetVehicleAge(vehicleId: string): void
  deleteVehicle(vehicleId: string): void
  batchFillAll(vehicleIds: string[]): void
  batchResetAge(vehicleIds: string[]): void
  resetChanges(): void
  getChanges(): VehicleChanges | null
}
```

### `sale.ts` — Used Market

```typescript
interface SaleStore {
  // State
  items: SaleItem[]
  addedItems: SaleAdditionPayload[]
  isDirty: boolean
  changeCount: number

  // Actions
  updateItem(index: number, changes: Partial<SaleItem>): void
  addItem(payload: SaleAdditionPayload): void  // Add a new vehicle to the market
  resetToNew(index: number): void          // wear=0, damage=0, age=0
  setDiscountPrice(index: number): void    // Price at 10%
  extendSale(index: number, days: number): void
  deleteItem(index: number): void
  resetChanges(): void
  getChanges(): { sales: SaleChange[] | null, saleAdditions: SaleAdditionPayload[] | null }
}
```

### `settings.ts` — Preferences

```typescript
interface SettingsStore {
  // State
  locale: 'fr' | 'en'
  theme: 'light' | 'dark' | 'system'
  advancedMode: boolean
  defaultPath: string | null
  maxBackups: number

  // Actions
  setLocale(locale: 'fr' | 'en'): void
  setTheme(theme: 'light' | 'dark' | 'system'): void
  toggleAdvancedMode(): void
  setDefaultPath(path: string | null): void
  setMaxBackups(count: number): void
  persist(): Promise<void>              // Save via tauri-plugin-store
  load(): Promise<void>                 // Load from tauri-plugin-store
}
```

### `backup.ts` — Backups

```typescript
interface BackupStore {
  // State
  backups: BackupInfo[]
  isCreating: boolean

  // Actions
  listBackups(): Promise<void>
  restore(backupName: string): Promise<void>
  delete(backupName: string): Promise<void>
}
```

## Dirty Tracking Pattern

Unsaved change tracking follows a uniform pattern:

1. On load, each store saves a copy of the original values
2. Each modification updates the current value but not the original
3. `isDirty` is a getter that compares current values to the originals
4. `resetChanges()` reverts current values to the originals
5. `getChanges()` returns a structured diff (only modified fields) or `null` if nothing has changed

A `useUnsavedChanges` composable aggregates the `isDirty` state from all stores to:
- Display a global change counter
- Block navigation if there are pending changes (confirmation dialog)
- Collect all `getChanges()` results for sending to the backend

## Internationalization (i18n)

### Translation File Structure

```json
// locales/fr.json
{
  "common": {
    "save": "Sauvegarder",
    "cancel": "Annuler",
    "delete": "Supprimer",
    "confirm": "Confirmer",
    "search": "Rechercher",
    "loading": "Chargement...",
    "advanced": "Mode avancé",
    "simple": "Mode simple"
  },
  "sidebar": {
    "finance": "Finances",
    "vehicles": "Véhicules",
    "sales": "Marché d'occasion",
    "fields": "Champs",
    "world": "Monde",
    "buildings": "Bâtiments",
    "missions": "Missions",
    "collectibles": "Collectibles",
    "backups": "Sauvegardes",
    "settings": "Paramètres",
    "comingSoon": "Bientôt"
  },
  "finance": {
    "title": "Finances de la ferme",
    "money": "Argent disponible",
    "loan": "Prêt en cours",
    "repayLoan": "Rembourser le prêt",
    "addMoney": "Ajouter de l'argent"
  },
  "vehicle": {
    "title": "Véhicules et équipements",
    "age": "Âge",
    "hours": "Heures d'utilisation",
    "fillAll": "Remplir tout",
    "emptyAll": "Vider tout",
    "resetAge": "Remettre à neuf",
    "owned": "Possédé",
    "rented": "Loué"
  },
  "fillTypes": {
    "DIESEL": "Diesel",
    "DEF": "AdBlue",
    "SEEDS": "Semences",
    "FERTILIZER": "Engrais",
    "WHEAT": "Blé",
    "BARLEY": "Orge",
    "CANOLA": "Colza"
  }
}
```

### Vehicle Names
Vehicle names are extracted from the XML path (`filename`) using a utility function:
- `data/vehicles/fendt/fendt942Vario/fendt942Vario.xml` -> `Fendt 942 Vario`
- Mapping is done via a lookup table in `constants.ts` for common vehicles
- Fallback: the last path segment is humanized (camelCase -> separated words)

## Theme and Design

### CSS Variables (shadcn-vue theme)
Light/dark theme is managed via shadcn-vue CSS variables, defined in `main.css`. The theme toggle adds/removes the `dark` class on `<html>`.

### Color Palette
The default shadcn-vue palette is used as a base, with custom accents evoking the Farming Simulator aesthetic:
- Primary accent: green (evoking agriculture)
- Statuses use semantic colors: green (success/owned), orange (warning/rented), red (error/deletion)

### Responsive
The application is designed for a desktop window (minimum 1024x768). The sidebar is collapsible for smaller windows. No mobile support (desktop app only).
