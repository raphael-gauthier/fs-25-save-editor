# 06 - Testing Strategy

## Overview

The testing strategy covers both layers of the application with tools suited to each environment.

```
                    Test pyramid

                         ╱╲
                        ╱  ╲
                       ╱ E2E╲          Optional (Phase 2+)
                      ╱──────╲
                     ╱Components╲      Vue Test Utils
                    ╱────────────╲
                   ╱  Unit tests  ╲    Vitest + cargo test
                  ╱────────────────╲
```

## Backend Tests (Rust)

### Tools
- `cargo test`: native Rust test framework
- Test files within each module (`#[cfg(test)] mod tests`)
- Fixtures: test XML files in `src-tauri/tests/fixtures/`

### Coverage

#### Parsers (high priority)
Each parser is tested with:
- **Nominal file**: a well-formed XML from a real savegame
- **Minimal file**: a valid XML with the minimum required fields
- **Missing fields**: verify that `Option<T>` values are properly None without errors
- **Malformed XML**: verify that the returned error is clean (`AppError::XmlParseError`)

Example tests for `parse_farms`:
```
test parse_farms_nominal           → Parses a complete farms.xml, verifies all values
test parse_farms_minimal           → Parses a farms.xml with a single farm without stats
test parse_farms_missing_stats     → The <statistics> block is absent → stats = default values
test parse_farms_invalid_xml       → Broken XML → returns XmlParseError
test parse_farms_empty_file        → Empty file → returns XmlParseError
```

#### Writers (high priority)
Each writer is tested with:
- **Round-trip**: parse an XML -> modify values -> write -> re-parse -> verify that modified values are correct AND that unmodified values are intact
- **Preservation**: verify that unmapped elements/attributes are preserved after writing
- **Encoding**: verify the XML header and UTF-8 encoding

Example tests for the vehicle writer:
```
test write_vehicle_price           → Modifies the price → re-parse → price is correct
test write_vehicle_fill_level      → Modifies a fillLevel → re-parse → level is correct
test write_vehicle_preserves_unknown → Writes a vehicle with unknown elements → they are preserved
test write_vehicle_roundtrip       → Parse → write → re-parse → identical data
```

#### Backup Manager (high priority)
```
test create_backup_creates_directory → The backup directory is created with the correct name
test create_backup_copies_all_files  → All files are copied (XML + binaries)
test create_backup_timestamp_format  → The backup name follows the expected format
test restore_backup_replaces_files   → Restoration properly replaces the contents
test delete_backup_removes_directory → Deletion cleans up the directory
test list_backups_sorted_by_date     → The list is sorted by date in descending order
```

#### Validators (medium priority)
```
test validate_money_consistency      → Detects money mismatch between career and farms
test validate_vehicle_farm_exists    → Detects a vehicle with a non-existent farmId
test validate_attachment_references  → Detects an invalid attachedVehicleUniqueId
```

#### Tauri Commands (medium priority)
Integration tests for commands using a test savegame directory:
```
test cmd_list_savegames              → Detects savegames in a fixture directory
test cmd_load_savegame               → Loads a complete savegame and returns SavegameData
test cmd_save_changes_creates_backup → Saving creates a backup first
test cmd_save_changes_applies_money  → Modifying money is reflected in the files
```

### Test Fixtures
A `src-tauri/tests/fixtures/` directory contains:
- `savegame_complete/`: copy of a real savegame (anonymized) for integration tests
- `savegame_minimal/`: minimalist savegame with the bare minimum in each file
- `xml/`: individual XML files for parser unit tests
  - `career_nominal.xml`, `career_minimal.xml`
  - `farms_nominal.xml`, `farms_minimal.xml`
  - `vehicles_nominal.xml`, `vehicles_single.xml`
  - `sales_nominal.xml`
  - etc.

## Frontend Tests (TypeScript/Vue)

### Tools
- **Vitest**: test framework (Vite-compatible, fast)
- **Vue Test Utils**: official library for testing Vue components
- **@pinia/testing**: helpers for testing Pinia stores

### Unit Tests (high priority)

#### Utilities and Helpers
```
test formatMoney              → Correct formatting based on locale (1000 → "1 000 €" / "$1,000")
test formatPlayTime           → Conversion from seconds → "Xh XXmin"
test vehicleDisplayName       → Extracts human-readable name from filename
test formatOperatingTime      → Converts hours → readable format
```

#### Pinia Stores
Each store is tested in isolation with mocked data (no IPC calls):

```
// finance.store.test.ts
test setMoney updates money          → The value is updated
test setMoney marks dirty            → isDirty becomes true
test repayLoan with deduction        → Money is deducted, loan set to 0
test repayLoan without deduction     → Only the loan is set to 0
test resetChanges restores original  → Values revert to original
test getChanges returns null if clean → No changes → null
test getChanges returns diff         → Returns only modified fields

// vehicle.store.test.ts
test filter by search query          → Filters by vehicle name
test filter by property state        → Filters owned/rented
test fillAllTanks                    → All fillUnits set to max
test batchResetAge                   → All selected vehicles set to age 0
test deleteVehicle removes from list → The vehicle is removed
```

### Component Tests (medium priority)

Tests for main components with Vue Test Utils:

```
// SavegameSelector.test.ts
test renders savegame list           → Displays detected savegames
test click selects savegame          → Click triggers loading
test shows loading state             → Skeleton is displayed during loading

// FinanceEditor.test.ts
test displays money and loan         → Values are displayed
test money input updates store       → Input updates the store
test quick add buttons               → +10k, +100k, +1M buttons work
test repay loan dialog               → Confirmation dialog is shown

// VehicleList.test.ts
test renders vehicle table           → DataTable displays vehicles
test search filters list             → Search filters in real-time
test click navigates to detail       → Click opens the detail view

// VehicleDetail.test.ts
test displays fill levels            → Fill level sliders are shown
test fill all button                 → Button fills everything
test advanced fields hidden          → Advanced fields are hidden in simple mode
test advanced fields visible         → Advanced fields are visible in advanced mode
```

### Tauri IPC Mocking

`invoke()` calls are mocked in tests via a helper:

```typescript
// tests/helpers/tauri-mock.ts
// Global mock for @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Helper to configure mocked responses
export function mockInvoke(command: string, response: unknown) {
  const { invoke } = await import('@tauri-apps/api/core')
  vi.mocked(invoke).mockImplementation((cmd, args) => {
    if (cmd === command) return Promise.resolve(response)
    return Promise.reject(new Error(`Unmocked command: ${cmd}`))
  })
}
```

## npm Scripts

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "test:rust": "cd src-tauri && cargo test",
    "test:all": "pnpm test && pnpm test:rust"
  }
}
```

## CI (recommended, out of MVP scope)

Eventually, a CI pipeline (GitHub Actions) will run:
1. `cargo test` (Rust backend)
2. `cargo clippy` (Rust linting)
3. `pnpm test` (frontend tests)
4. `pnpm lint` (ESLint)
5. `pnpm tauri build` (verify the build compiles)
