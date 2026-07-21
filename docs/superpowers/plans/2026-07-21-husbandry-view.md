# Husbandry (Animals) View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an "Élevages" section letting the player inspect and adjust owned animals — herd size, age, health, reproduction — plus their food, water and pasture stocks.

**Architecture:** Husbandries live in `placeables.xml` but get their own model / parser / writer chain so their change payload never collides with `PlaceableChange`. A second sequential pass over the file keeps the two features decoupled. The frontend adds a dedicated route, store and card-based view.

**Tech Stack:** Rust (quick-xml, serde) for parsing/writing; Vue 3 `<script setup>` + Pinia + shadcn-vue + vue-i18n for the UI.

**Spec:** `docs/superpowers/specs/2026-07-21-husbandry-view-design.md`

---

## Conventions for every task

- Backend tests: `cd src-tauri && cargo test`
- Frontend type check: `pnpm lint`
- TDD is mandatory: write the test, **run it and see it fail**, then implement.
- The fixture used by backend tests is `src-tauri/tests/fixtures/savegame_complete/`.
- Commit after each task. Conventional Commits, atomic.

## File Structure

| File | Responsibility |
|---|---|
| `src-tauri/src/models/husbandry.rs` | **Create.** `Husbandry`, `AnimalCluster`, `FillStock`, `MeadowStock` |
| `src-tauri/src/models/placeable.rs` | **Modify.** Receives `display_name_from_unique_id` so both parsers share it |
| `src-tauri/src/models/mod.rs` | **Modify.** Register module; add `husbandries` to `SavegameData` |
| `src-tauri/src/models/changes.rs` | **Modify.** `HusbandryChange`, `AnimalClusterChange`, `FillStockChange`; add to `SavegameChanges` |
| `src-tauri/src/parsers/husbandry.rs` | **Create.** Second pass over `placeables.xml` |
| `src-tauri/src/parsers/placeable.rs` | **Modify.** Drops its private copy of the name helper |
| `src-tauri/src/parsers/mod.rs` | **Modify.** Register module |
| `src-tauri/src/writers/husbandry.rs` | **Create.** Patches by `(placeable_index, cluster_index)` |
| `src-tauri/src/writers/mod.rs` | **Modify.** Register module |
| `src-tauri/src/commands/savegame.rs` | **Modify.** Wire parse into load, writer into save |
| `src-tauri/tests/fixtures/savegame_complete/placeables.xml` | **Modify.** Add the three real husbandry shapes |
| `src/lib/types.ts` | **Modify.** Mirror the Rust types |
| `src/stores/animal.ts` | **Create.** Dirty-tracking store |
| `src/components/animals/HusbandryCard.vue` | **Create.** One husbandry, edited in place |
| `src/views/AnimalListView.vue` | **Create.** List of cards |
| `src/router/index.ts` | **Modify.** Route `/editor/animals` |
| `src/components/layout/AppSidebar.vue` | **Modify.** Sidebar entry |
| `src/components/layout/EditorLayout.vue` | **Modify.** Hydrate the store |
| `src/components/layout/AppHeader.vue` | **Modify.** Hydrate on reload (Task 16) |
| `src/composables/useUnsavedChanges.ts` | **Modify.** Aggregate dirty state and changes |
| `src/locales/fr.json`, `src/locales/en.json` | **Modify.** Strings + species names |

---

## Task 1: Husbandry model

**Files:**
- Create: `src-tauri/src/models/husbandry.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: Create the model file**

```rust
use serde::{Deserialize, Serialize};

/// One animal husbandry, extracted from a `<placeable>` in placeables.xml.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Husbandry {
    /// Index of the owning placeable, using the same numbering as `parse_placeables`.
    /// This is the write target — it must stay aligned with that parser.
    pub placeable_index: usize,
    pub display_name: String,
    pub farm_id: u8,
    /// How well the husbandry is currently producing. Read-only.
    pub production_factor: f64,
    pub clusters: Vec<AnimalCluster>,
    /// `<husbandryFood><fillLevel>` — no capacity is recorded in the savegame.
    pub food: Vec<FillStock>,
    /// `<husbandry><storage><node>` — typically WATER. No capacity either.
    pub water: Vec<FillStock>,
    /// `<husbandryMeadow><fillType>` — the only stock that records a capacity.
    pub meadow: Vec<MeadowStock>,
}

/// A batch of same-age animals. `<animal>` carries no identifier, so clusters are
/// addressed by their position within `<clusters>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalCluster {
    pub index: usize,
    pub sub_type: String,
    pub num_animals: u32,
    /// Age in months.
    pub age: u32,
    pub health: f64,
    pub reproduction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillStock {
    pub fill_type: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeadowStock {
    pub fill_type: String,
    pub amount: f64,
    pub capacity: f64,
}
```

- [ ] **Step 2: Register the module and extend `SavegameData`**

In `src-tauri/src/models/mod.rs`, add `pub mod husbandry;` alongside the other module
declarations (keep alphabetical order — it goes after `farm`, before `mission`... actually
after `field`).

Then add the import and the field:

```rust
use husbandry::Husbandry;
```

and inside `pub struct SavegameData`, after `pub placeables: Vec<Placeable>,`:

```rust
    pub husbandries: Vec<Husbandry>,
```

- [ ] **Step 3: Verify it compiles (it will fail at the SavegameData literal)**

Run: `cd src-tauri && cargo build`
Expected: FAIL — `missing field husbandries in initializer of SavegameData` at
`src/commands/savegame.rs:244`. That is the wiring done in Task 6; for now add the field
with an empty vector so the tree compiles.

In `src-tauri/src/commands/savegame.rs`, in the `SavegameData { … }` literal (around line
244), add after `placeables,`:

```rust
        husbandries: Vec::new(),
```

- [ ] **Step 4: Run the test suite**

Run: `cd src-tauri && cargo test`
Expected: PASS — 148 tests, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models/husbandry.rs src-tauri/src/models/mod.rs src-tauri/src/commands/savegame.rs
git commit -m "feat(husbandry): add husbandry data model"
```

---

## Task 2: Share the pre-placed name helper

Pre-placed husbandries carry no `filename`, so the husbandry parser needs the same
`uniqueId`-based naming the placeable parser already does. Move the helper so both use it.

**Files:**
- Modify: `src-tauri/src/models/placeable.rs`
- Modify: `src-tauri/src/parsers/placeable.rs`

- [ ] **Step 1: Write the failing test**

Append to the `mod tests` block at the bottom of `src-tauri/src/models/placeable.rs` (create
the block if the file has none):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_from_unique_id_strips_prefix_and_hash() {
        assert_eq!(
            display_name_from_unique_id("preplaced_farmHouse05_306e852a51d90726d144af50a865cbf0"),
            "Farm House 05"
        );
    }

    #[test]
    fn test_display_name_from_unique_id_without_hash() {
        assert_eq!(
            display_name_from_unique_id("sellingStationTrain01"),
            "Selling Station Train 01"
        );
    }
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd src-tauri && cargo test models::placeable`
Expected: FAIL — `cannot find function display_name_from_unique_id in this scope`.

- [ ] **Step 3: Move the function**

Cut `display_name_from_unique_id` out of `src-tauri/src/parsers/placeable.rs` and paste it
into `src-tauri/src/models/placeable.rs`, above the `mod tests` block, making it public:

```rust
/// Pre-placed map objects carry no `filename`, so their name must come from the
/// `uniqueId` — e.g. `preplaced_farmHouse05_306e852a...` becomes "Farm House 05".
pub fn display_name_from_unique_id(unique_id: &str) -> String {
    let stem = unique_id.strip_prefix("preplaced_").unwrap_or(unique_id);

    // Drop the trailing 32-character hex hash the game appends to pre-placed ids.
    let stem = match stem.rsplit_once('_') {
        Some((head, tail))
            if tail.len() == 32 && tail.chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            head
        }
        _ => stem,
    };

    placeable_display_name(stem)
}
```

In `src-tauri/src/parsers/placeable.rs`, update the import to pull it from the model:

```rust
use crate::models::placeable::{
    display_name_from_unique_id, placeable_display_name, ConstructionMaterial, ConstructionState,
    Placeable, ProductionLine, ProductionStock,
};
```

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test`
Expected: PASS — 150 tests. The existing
`parsers::placeable::tests::test_parse_placeables_display_name_from_unique_id` must still pass,
proving the move did not change behaviour.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models/placeable.rs src-tauri/src/parsers/placeable.rs
git commit -m "refactor(placeables): move pre-placed name helper into the model"
```

---

## Task 3: Extend the test fixture with real husbandry data

The parser tests in Tasks 4–5 need the three shapes observed in the reference save.

**Files:**
- Modify: `src-tauri/tests/fixtures/savegame_complete/placeables.xml`

- [ ] **Step 1: Append three placeables before `</placeables>`**

Insert these immediately before the closing `</placeables>` tag, keeping the five existing
placeables untouched (they are indices 0–4; these become 5, 6, 7):

```xml
    <placeable filename="data/placeables/rudolfHormann/chickenBarnBig/chickenBarnBig.xml" uniqueId="placeable4d5e6f70819a2b3c4d5e6f70819a2b3c" position="-621.584 46.992 84.672" rotation="-180.00 0.89 180.00" age="42.000000" price="79000.000000" farmId="1">
        <husbandry globalProductionFactor="1.000000" productionFactor="1.000000">
            <storage farmId="1"/>
        </husbandry>
        <husbandryAnimals>
            <clusters>
                <animal subType="CHICKEN" numAnimals="360" age="48" health="100" reproduction="0"/>
            </clusters>
        </husbandryAnimals>
        <husbandryFood>
            <fillLevel fillType="WHEAT" fillLevel="5400.000000"/>
        </husbandryFood>
    </placeable>
    <placeable filename="data/placeables/brandless/animalHusbandries/sheepBarnSmall/sheepBarnSmall.xml" uniqueId="placeable5e6f70819a2b3c4d5e6f70819a2b3c4d" position="-670.750 47.000 141.000" rotation="-0.00 90.00 0.00" age="21.000000" price="14000.000000" farmId="1">
        <husbandry globalProductionFactor="1.000000" productionFactor="0.750000">
            <storage farmId="1">
                <node fillType="WATER" fillLevel="1500.000000"/>
            </storage>
        </husbandry>
        <husbandryAnimals>
            <clusters>
                <animal subType="GOAT" numAnimals="20" age="2" health="100" reproduction="0"/>
                <animal subType="GOAT" numAnimals="71" age="24" health="90" reproduction="40"/>
            </clusters>
        </husbandryAnimals>
        <husbandryMeadow>
            <fillType name="GRASS_WINDROW" fillLevel="21689.705078" capacity="25693.414062"/>
        </husbandryMeadow>
    </placeable>
    <placeable isPreplaced="true" uniqueId="preplaced_chickenCoop02_ec6e2c2f9b9bff576b4775465b1de1dc" age="59.000000" price="0.000000" farmId="0">
        <husbandry globalProductionFactor="0.000000" productionFactor="0.000000"/>
    </placeable>
```

- [ ] **Step 2: Update the placeable count assertions**

Two existing tests assert the fixture size and must move from 5 to 8.

In `src-tauri/src/parsers/placeable.rs`, in `test_parse_placeables_nominal`:

```rust
        assert_eq!(placeables.len(), 8);
```

In `src-tauri/src/commands/savegame.rs`, in `test_load_savegame_complete`:

```rust
        assert_eq!(data.placeables.len(), 8);
```

The new chicken coop is pre-placed, so `test_parse_placeables_pre_placed_uses_attribute_not_farm_id`
in `src-tauri/src/parsers/placeable.rs` goes from 2 to 3:

```rust
        assert_eq!(pre_placed.len(), 3);
```

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test`
Expected: PASS — 150 tests, no warnings.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tests/fixtures/savegame_complete/placeables.xml src-tauri/src/parsers/placeable.rs src-tauri/src/commands/savegame.rs
git commit -m "test(fixtures): add real husbandry shapes to the placeables fixture"
```

---

## Task 4: Parse animal clusters

**Files:**
- Create: `src-tauri/src/parsers/husbandry.rs`
- Modify: `src-tauri/src/parsers/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/parsers/husbandry.rs` containing only the test module plus a stub, so
the test compiles and fails on the assertion rather than on a missing symbol:

```rust
use std::path::Path;

use crate::error::AppError;
use crate::models::husbandry::Husbandry;

pub fn parse_husbandries(_path: &Path) -> Result<Vec<Husbandry>, AppError> {
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    #[test]
    fn test_parse_husbandries_finds_only_husbandry_placeables() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();
        assert_eq!(husbandries.len(), 3);
    }

    #[test]
    fn test_parse_husbandries_single_cluster() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Chicken Barn Big"))
            .unwrap();
        assert_eq!(barn.farm_id, 1);
        assert_eq!(barn.clusters.len(), 1);

        let cluster = &barn.clusters[0];
        assert_eq!(cluster.index, 0);
        assert_eq!(cluster.sub_type, "CHICKEN");
        assert_eq!(cluster.num_animals, 360);
        assert_eq!(cluster.age, 48);
        assert!((cluster.health - 100.0).abs() < 0.01);
        assert!((cluster.reproduction - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_multiple_clusters_keep_document_order() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Sheep Barn Small"))
            .unwrap();
        assert_eq!(barn.clusters.len(), 2);
        assert_eq!(barn.clusters[0].index, 0);
        assert_eq!(barn.clusters[0].num_animals, 20);
        assert_eq!(barn.clusters[1].index, 1);
        assert_eq!(barn.clusters[1].num_animals, 71);
        assert!((barn.clusters[1].health - 90.0).abs() < 0.01);
        assert!((barn.production_factor - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_empty_preplaced_is_kept_and_named() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        // No <husbandryAnimals> at all, and no filename to derive a name from.
        let coop = husbandries
            .iter()
            .find(|h| h.display_name == "Chicken Coop 02")
            .unwrap();
        assert!(coop.clusters.is_empty());
        assert!((coop.production_factor - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_placeable_index_matches_placeable_parser() {
        // The writer finds its target by this index, so it must agree with the
        // numbering parse_placeables assigns — counting every <placeable>, not
        // just the ones holding animals.
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();
        let placeables = crate::parsers::placeable::parse_placeables(&path).unwrap();

        for h in &husbandries {
            let p = placeables
                .iter()
                .find(|p| p.index == h.placeable_index)
                .unwrap();
            assert_eq!(p.display_name, h.display_name);
        }
    }

    #[test]
    fn test_parse_husbandries_missing_file() {
        let dir = std::env::temp_dir().join("fs25_test_no_husbandry");
        let _ = std::fs::create_dir_all(&dir);
        let result = parse_husbandries(&dir);
        assert!(matches!(result, Err(AppError::IoError { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

Register the module — in `src-tauri/src/parsers/mod.rs` add:

```rust
pub mod husbandry;
```

- [ ] **Step 2: Run the tests and watch them fail**

Run: `cd src-tauri && cargo test parsers::husbandry`
Expected: FAIL — `test_parse_husbandries_finds_only_husbandry_placeables` reports
`left: 0, right: 3`, and the others panic on `unwrap()` of a missing husbandry.

- [ ] **Step 3: Implement the parser**

Replace the stub in `src-tauri/src/parsers/husbandry.rs` (keep the `mod tests` block) with:

```rust
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;
use crate::models::husbandry::{AnimalCluster, Husbandry};
use crate::models::placeable::{display_name_from_unique_id, placeable_display_name};

fn attr_str(e: &quick_xml::events::BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

fn attr_f64(e: &quick_xml::events::BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

fn attr_u8(e: &quick_xml::events::BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

fn attr_u32(e: &quick_xml::events::BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

/// Extracts every animal husbandry from placeables.xml.
///
/// This is a second pass over the same file `parse_placeables` reads. Husbandries keep
/// their own change payload so that editing animals can never collide with editing
/// buildings, and `placeable_index` is what ties the two together.
pub fn parse_husbandries(path: &Path) -> Result<Vec<Husbandry>, AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let mut reader = Reader::from_str(&content);
    let mut husbandries: Vec<Husbandry> = Vec::new();

    // Counts EVERY <placeable> so indices stay aligned with parse_placeables.
    let mut placeable_index: usize = 0;
    let mut current: Option<Husbandry> = None;
    let mut is_husbandry = false;

    let mut in_clusters = false;

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;

        let (e, is_empty) = match &event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(e) => {
                match e.name().as_ref() {
                    b"placeable" => {
                        if is_husbandry {
                            if let Some(h) = current.take() {
                                husbandries.push(h);
                            }
                        }
                        current = None;
                        is_husbandry = false;
                        in_clusters = false;
                    }
                    b"clusters" => in_clusters = false,
                    _ => {}
                }
                continue;
            }
            Event::Eof => break,
            _ => continue,
        };

        match e.name().as_ref() {
            b"placeable" => {
                // Flush a previous self-closing placeable that was never closed by an End.
                if is_husbandry {
                    if let Some(h) = current.take() {
                        husbandries.push(h);
                    }
                }
                is_husbandry = false;
                in_clusters = false;

                let filename = attr_str(e, "filename");
                let unique_id = attr_str(e, "uniqueId");
                let display_name = if filename.is_empty() {
                    display_name_from_unique_id(&unique_id)
                } else {
                    placeable_display_name(&filename)
                };

                current = Some(Husbandry {
                    placeable_index,
                    display_name,
                    farm_id: attr_u8(e, "farmId"),
                    production_factor: 0.0,
                    clusters: Vec::new(),
                    food: Vec::new(),
                    water: Vec::new(),
                    meadow: Vec::new(),
                });
                placeable_index += 1;

                if is_empty {
                    current = None;
                }
            }
            b"husbandry" => {
                is_husbandry = true;
                if let Some(ref mut h) = current {
                    h.production_factor = attr_f64(e, "productionFactor");
                }
            }
            b"husbandryAnimals" => is_husbandry = true,
            b"clusters" => in_clusters = !is_empty,
            b"animal" if in_clusters => {
                if let Some(ref mut h) = current {
                    h.clusters.push(AnimalCluster {
                        index: h.clusters.len(),
                        sub_type: attr_str(e, "subType"),
                        num_animals: attr_u32(e, "numAnimals"),
                        age: attr_u32(e, "age"),
                        health: attr_f64(e, "health"),
                        reproduction: attr_f64(e, "reproduction"),
                    });
                }
            }
            // Food, water and pasture are added in Task 5.
            _ => {}
        }
    }

    if is_husbandry {
        if let Some(h) = current.take() {
            husbandries.push(h);
        }
    }

    Ok(husbandries)
}
```

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test parsers::husbandry`
Expected: PASS — 6 tests.

Then run the whole suite: `cd src-tauri && cargo test`
Expected: PASS — 156 tests, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parsers/husbandry.rs src-tauri/src/parsers/mod.rs
git commit -m "feat(husbandry): parse animal clusters from placeables.xml"
```

---

## Task 5: Parse food, water and pasture stocks

Task 4 deliberately left the stock fields empty. This task adds them.

**Files:**
- Modify: `src-tauri/src/parsers/husbandry.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block in `src-tauri/src/parsers/husbandry.rs`:

```rust
    #[test]
    fn test_parse_husbandries_food_stock() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Chicken Barn Big"))
            .unwrap();
        assert_eq!(barn.food.len(), 1);
        assert_eq!(barn.food[0].fill_type, "WHEAT");
        assert!((barn.food[0].amount - 5400.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_water_and_meadow() {
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        let barn = husbandries
            .iter()
            .find(|h| h.display_name.contains("Sheep Barn Small"))
            .unwrap();

        assert_eq!(barn.water.len(), 1);
        assert_eq!(barn.water[0].fill_type, "WATER");
        assert!((barn.water[0].amount - 1500.0).abs() < 0.01);

        // The meadow is the only stock the savegame gives a capacity for.
        assert_eq!(barn.meadow.len(), 1);
        assert_eq!(barn.meadow[0].fill_type, "GRASS_WINDROW");
        assert!((barn.meadow[0].amount - 21689.705078).abs() < 0.01);
        assert!((barn.meadow[0].capacity - 25693.414062).abs() < 0.01);
    }

    #[test]
    fn test_parse_husbandries_ignores_silo_and_production_storage() {
        // Fixture index 0 is a silo with <silo><storage><node fillType="WHEAT">, and
        // index 2 is a production point with its own <storage><node>. Neither is a
        // husbandry, so neither may leak into the results.
        let path = fixtures_path().join("savegame_complete");
        let husbandries = parse_husbandries(&path).unwrap();

        assert!(husbandries.iter().all(|h| h.placeable_index >= 5));
        for h in &husbandries {
            assert!(
                h.water.iter().all(|w| w.fill_type == "WATER"),
                "silo or production stock leaked into husbandry water"
            );
        }
    }
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cd src-tauri && cargo test parsers::husbandry`
Expected: FAIL — `test_parse_husbandries_food_stock` reports `left: 0, right: 1`, and
`test_parse_husbandries_water_and_meadow` likewise. Task 4 left these vectors empty.

- [ ] **Step 3: Add stock parsing**

In `src-tauri/src/parsers/husbandry.rs`, extend the import:

```rust
use crate::models::husbandry::{AnimalCluster, FillStock, Husbandry, MeadowStock};
```

Add three context flags next to `let mut in_clusters = false;`:

```rust
    // <storage>/<node> are also used by silos and production points, so water
    // collection is gated on being inside <husbandry>.
    let mut in_husbandry = false;
    let mut in_husbandry_storage = false;
    let mut in_food = false;
    let mut in_meadow = false;
```

In the `Event::End` match, extend the `b"placeable"` arm to also reset them, and add the new
arms:

```rust
                    b"placeable" => {
                        if is_husbandry {
                            if let Some(h) = current.take() {
                                husbandries.push(h);
                            }
                        }
                        current = None;
                        is_husbandry = false;
                        in_clusters = false;
                        in_husbandry = false;
                        in_husbandry_storage = false;
                        in_food = false;
                        in_meadow = false;
                    }
                    b"husbandry" => {
                        in_husbandry = false;
                        in_husbandry_storage = false;
                    }
                    b"storage" if in_husbandry => in_husbandry_storage = false,
                    b"clusters" => in_clusters = false,
                    b"husbandryFood" => in_food = false,
                    b"husbandryMeadow" => in_meadow = false,
```

In the start-tag `b"placeable"` arm, reset the same four flags alongside `in_clusters = false;`.

In the `b"husbandry"` arm, add `in_husbandry = !is_empty;`.

Finally replace the `// Food, water and pasture are added in Task 5.` comment with:

```rust
            b"husbandryFood" => {
                is_husbandry = true;
                in_food = !is_empty;
            }
            b"fillLevel" if in_food => {
                if let Some(ref mut h) = current {
                    h.food.push(FillStock {
                        fill_type: attr_str(e, "fillType"),
                        amount: attr_f64(e, "fillLevel"),
                    });
                }
            }
            b"husbandryMeadow" => {
                is_husbandry = true;
                in_meadow = !is_empty;
            }
            b"fillType" if in_meadow => {
                if let Some(ref mut h) = current {
                    h.meadow.push(MeadowStock {
                        fill_type: attr_str(e, "name"),
                        amount: attr_f64(e, "fillLevel"),
                        capacity: attr_f64(e, "capacity"),
                    });
                }
            }
            b"storage" if in_husbandry => in_husbandry_storage = !is_empty,
            b"node" if in_husbandry_storage => {
                if let Some(ref mut h) = current {
                    h.water.push(FillStock {
                        fill_type: attr_str(e, "fillType"),
                        amount: attr_f64(e, "fillLevel"),
                    });
                }
            }
```

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test parsers::husbandry`
Expected: PASS — 9 tests.

Then: `cd src-tauri && cargo test`
Expected: PASS — 159 tests, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parsers/husbandry.rs
git commit -m "feat(husbandry): parse food, water and pasture stocks"
```

---

## Task 6: Wire parsing into load_savegame

**Files:**
- Modify: `src-tauri/src/commands/savegame.rs`

- [ ] **Step 1: Write the failing test**

In the `mod tests` block of `src-tauri/src/commands/savegame.rs`, inside
`test_load_savegame_complete`, add after the `assert_eq!(data.placeables.len(), 8);` line:

```rust
        assert_eq!(data.husbandries.len(), 3);
        let total_animals: u32 = data
            .husbandries
            .iter()
            .flat_map(|h| &h.clusters)
            .map(|c| c.num_animals)
            .sum();
        assert_eq!(total_animals, 451);
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd src-tauri && cargo test test_load_savegame_complete`
Expected: FAIL — `left: 0, right: 3` (Task 1 hard-coded an empty vector).

- [ ] **Step 3: Wire the parser in**

Add the import near the other parser imports at the top of the file (around line 18):

```rust
use crate::parsers::husbandry::parse_husbandries;
```

After the `// Parse placeables (optional)` block (around line 173–182), add:

```rust
    // Parse husbandries — a second pass over placeables.xml (optional)
    let husbandries = match parse_husbandries(&save_path) {
        Ok(h) => h,
        Err(_) => {
            // placeables.xml already produced a warning above if it is unreadable.
            Vec::new()
        }
    };
```

Then replace the placeholder in the `SavegameData { … }` literal:

```rust
        husbandries,
```

- [ ] **Step 3b: Remove the now-unnecessary dead_code allow**

Task 4 added `#[allow(dead_code)]` to `parse_husbandries` in `src-tauri/src/parsers/husbandry.rs`
purely because nothing called it yet. This task is what calls it, so delete that attribute —
along with any item-level allows on the `attr_*` helpers in that file.

Then run `cd src-tauri && cargo build` and confirm zero warnings. If a warning reappears, the
wiring is not actually reaching the parser; fix the wiring rather than restoring the allow.

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test`
Expected: PASS — 159 tests, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/savegame.rs
git commit -m "feat(husbandry): expose husbandries in loaded savegame data"
```

---

## Task 7: Change payload types

**Files:**
- Modify: `src-tauri/src/models/changes.rs`

- [ ] **Step 1: Add the types**

Append to `src-tauri/src/models/changes.rs`, next to `PlaceableChange`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HusbandryChange {
    /// Same numbering as `Placeable::index`.
    pub placeable_index: usize,
    pub clusters: Option<Vec<AnimalClusterChange>>,
    pub food: Option<Vec<FillStockChange>>,
    pub water: Option<Vec<FillStockChange>>,
    pub meadow: Option<Vec<FillStockChange>>,
}

/// Every field is optional so untouched attributes are rewritten verbatim.
/// `subType` is absent by design: changing species is out of scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalClusterChange {
    /// Position within `<clusters>`.
    pub index: usize,
    pub num_animals: Option<u32>,
    pub age: Option<u32>,
    pub health: Option<f64>,
    pub reproduction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillStockChange {
    pub fill_type: String,
    pub amount: f64,
}
```

Then add the field to `SavegameChanges`, after `pub placeables: Option<Vec<PlaceableChange>>,`:

```rust
    pub husbandries: Option<Vec<HusbandryChange>>,
}
```

- [ ] **Step 2: Build and fix the literals**

Run: `cd src-tauri && cargo build`
Expected: FAIL — `missing field husbandries` in every `SavegameChanges { … }` literal in the
test module of `src/commands/savegame.rs` (lines ~688, 715, 741, 775, 812, 855, 940).

Add `husbandries: None,` next to the existing `placeables: None,` in each of those literals.

- [ ] **Step 3: Run the tests**

Run: `cd src-tauri && cargo test`
Expected: PASS — 159 tests.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models/changes.rs src-tauri/src/commands/savegame.rs
git commit -m "feat(husbandry): add husbandry change payload types"
```

---

## Task 8: Write cluster changes

**Files:**
- Create: `src-tauri/src/writers/husbandry.rs`
- Modify: `src-tauri/src/writers/mod.rs`

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/writers/husbandry.rs` with a stub plus tests:

```rust
use std::path::Path;

use crate::error::AppError;
use crate::models::changes::HusbandryChange;

pub fn write_husbandry_changes(
    _path: &Path,
    _changes: &[HusbandryChange],
) -> Result<(), AppError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::changes::{AnimalClusterChange, FillStockChange};
    use crate::parsers::husbandry::parse_husbandries;

    fn setup_fixture(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("savegame_complete");
        let dst = std::env::temp_dir().join(format!("fs25_test_wh_{}", name));
        let _ = std::fs::remove_dir_all(&dst);
        std::fs::create_dir_all(&dst).unwrap();
        for entry in std::fs::read_dir(&src).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), dst.join(entry.file_name())).unwrap();
            }
        }
        dst
    }

    fn husbandry_named<'a>(
        list: &'a [crate::models::husbandry::Husbandry],
        needle: &str,
    ) -> &'a crate::models::husbandry::Husbandry {
        list.iter()
            .find(|h| h.display_name.contains(needle))
            .unwrap()
    }

    #[test]
    fn test_write_cluster_patches_self_closing_animal() {
        let save = setup_fixture("cluster");
        let before = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&before, "Chicken Barn Big");

        let changes = vec![HusbandryChange {
            placeable_index: barn.placeable_index,
            clusters: Some(vec![AnimalClusterChange {
                index: 0,
                num_animals: Some(500),
                age: None,
                health: Some(75.0),
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&after, "Chicken Barn Big");
        assert_eq!(barn.clusters[0].num_animals, 500);
        assert!((barn.clusters[0].health - 75.0).abs() < 0.01);
        // Untouched attributes keep their original values.
        assert_eq!(barn.clusters[0].age, 48);
        assert_eq!(barn.clusters[0].sub_type, "CHICKEN");
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_cluster_targets_the_right_index() {
        // Editing the SECOND cluster catches off-by-one drift between parser and writer.
        let save = setup_fixture("cluster_second");
        let before = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&before, "Sheep Barn Small");

        let changes = vec![HusbandryChange {
            placeable_index: barn.placeable_index,
            clusters: Some(vec![AnimalClusterChange {
                index: 1,
                num_animals: Some(7),
                age: None,
                health: None,
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let barn = husbandry_named(&after, "Sheep Barn Small");
        assert_eq!(barn.clusters[0].num_animals, 20, "first cluster must be untouched");
        assert_eq!(barn.clusters[1].num_animals, 7);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_husbandry_leaves_other_placeables_untouched() {
        let save = setup_fixture("others");
        let before = crate::parsers::placeable::parse_placeables(&save).unwrap();
        let barn = husbandry_named(&parse_husbandries(&save).unwrap(), "Chicken Barn Big")
            .placeable_index;

        let changes = vec![HusbandryChange {
            placeable_index: barn,
            clusters: Some(vec![AnimalClusterChange {
                index: 0,
                num_animals: Some(1),
                age: None,
                health: None,
                reproduction: None,
            }]),
            food: None,
            water: None,
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = crate::parsers::placeable::parse_placeables(&save).unwrap();
        assert_eq!(after.len(), before.len());
        for (a, b) in after.iter().zip(before.iter()) {
            assert_eq!(a.farm_id, b.farm_id);
            assert!((a.price - b.price).abs() < 0.01);
            assert_eq!(a.production_storage.len(), b.production_storage.len());
        }
        let _ = std::fs::remove_dir_all(&save);
    }
}
```

Register it — in `src-tauri/src/writers/mod.rs` add:

```rust
pub mod husbandry;
```

- [ ] **Step 2: Run and watch them fail**

Run: `cd src-tauri && cargo test writers::husbandry`
Expected: FAIL — `test_write_cluster_patches_self_closing_animal` reports
`left: 360, right: 500`; the stub writes nothing.

- [ ] **Step 3: Implement the writer**

Replace the stub (keep the `mod tests` block):

```rust
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::AppError;
use crate::models::changes::{AnimalClusterChange, HusbandryChange};

#[allow(dead_code)]
fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

/// Applies husbandry changes to placeables.xml.
///
/// Runs as a separate pass from `write_placeable_changes`, reading whatever that pass
/// wrote. Targets are found by `(placeable_index, cluster_index)` because `<animal>`
/// carries no identifier.
pub fn write_husbandry_changes(
    path: &Path,
    changes: &[HusbandryChange],
) -> Result<(), AppError> {
    let xml_path = path.join("placeables.xml");
    let content = std::fs::read_to_string(&xml_path).map_err(|e| AppError::IoError {
        message: format!("{}: {}", xml_path.display(), e),
    })?;

    let change_map: std::collections::HashMap<usize, &HusbandryChange> =
        changes.iter().map(|c| (c.placeable_index, c)).collect();

    let mut reader = Reader::from_str(&content);
    let mut writer = Writer::new(Vec::new());

    let mut placeable_index: usize = 0;
    let mut current: Option<&HusbandryChange> = None;
    let mut cluster_index: usize = 0;
    let mut in_clusters = false;

    loop {
        let event = reader.read_event().map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })?;

        match event {
            Event::Eof => break,
            Event::Start(ref e) | Event::Empty(ref e) => {
                let is_empty = matches!(event, Event::Empty(_));
                let tag = e.name().as_ref().to_vec();

                match tag.as_slice() {
                    b"placeable" => {
                        current = change_map.get(&placeable_index).copied();
                        placeable_index += 1;
                        cluster_index = 0;
                        in_clusters = false;
                    }
                    b"clusters" => {
                        in_clusters = !is_empty;
                        cluster_index = 0;
                    }
                    _ => {}
                }

                // Decide whether this element needs patching.
                // Stock arms are added in Task 9.
                let patched: Option<BytesStart<'static>> = match (current, tag.as_slice()) {
                    (Some(change), b"animal") if in_clusters => {
                        let this_index = cluster_index;
                        cluster_index += 1;
                        change
                            .clusters
                            .as_ref()
                            .and_then(|cs| cs.iter().find(|c| c.index == this_index))
                            .map(|c| patch_animal(e, c))
                    }
                    _ => {
                        if tag.as_slice() == b"animal" && in_clusters {
                            cluster_index += 1;
                        }
                        None
                    }
                };

                let out = match patched {
                    Some(elem) => elem,
                    None => e.clone().into_owned(),
                };
                let out_event = if is_empty {
                    Event::Empty(out)
                } else {
                    Event::Start(out)
                };
                write_event(&mut writer, &xml_path, out_event)?;
            }
            Event::End(ref e) => {
                match e.name().as_ref() {
                    b"placeable" => current = None,
                    b"clusters" => in_clusters = false,
                    _ => {}
                }
                write_event(&mut writer, &xml_path, Event::End(e.clone().into_owned()))?;
            }
            other => {
                write_event(&mut writer, &xml_path, other.into_owned())?;
            }
        }
    }

    let output = writer.into_inner();
    let tmp_path = xml_path.with_extension("xml.tmp");
    std::fs::write(&tmp_path, &output)?;
    std::fs::rename(&tmp_path, &xml_path)?;

    Ok(())
}

fn patch_animal(e: &BytesStart, change: &AnimalClusterChange) -> BytesStart<'static> {
    let mut elem = BytesStart::new("animal");
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let original = String::from_utf8_lossy(&attr.value).to_string();
        let value = match key.as_str() {
            "numAnimals" => change.num_animals.map(|v| v.to_string()),
            "age" => change.age.map(|v| v.to_string()),
            "health" => change.health.map(|v| format!("{}", v.round() as i64)),
            "reproduction" => change.reproduction.map(|v| format!("{}", v.round() as i64)),
            _ => None,
        }
        .unwrap_or(original);
        elem.push_attribute((key.as_str(), value.as_str()));
    }
    elem
}

fn write_event(
    writer: &mut Writer<Vec<u8>>,
    xml_path: &Path,
    event: Event<'static>,
) -> Result<(), AppError> {
    writer
        .write_event(event)
        .map_err(|e| AppError::XmlParseError {
            file: xml_path.display().to_string(),
            message: e.to_string(),
        })
}
```

> Note on `health`/`reproduction`: the game writes them as bare integers
> (`health="100"`), not the 6-decimal float form used for fill levels, so they are
> rounded to match.

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test writers::husbandry`
Expected: PASS — 3 tests.

Then: `cd src-tauri && cargo test`
Expected: PASS — 162 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/writers/husbandry.rs src-tauri/src/writers/mod.rs
git commit -m "feat(husbandry): write animal cluster changes"
```

---

## Task 9: Write stock changes and prove scoping

Task 8 handled clusters only. This task adds food, water and pasture patching.

**Files:**
- Modify: `src-tauri/src/writers/husbandry.rs`

- [ ] **Step 1: Write the failing tests**

Add to the `mod tests` block:

```rust
    #[test]
    fn test_write_food_water_and_meadow() {
        let save = setup_fixture("stocks");
        let all = parse_husbandries(&save).unwrap();
        let coop_index = husbandry_named(&all, "Chicken Barn Big").placeable_index;
        let barn_index = husbandry_named(&all, "Sheep Barn Small").placeable_index;

        let changes = vec![
            HusbandryChange {
                placeable_index: coop_index,
                clusters: None,
                food: Some(vec![FillStockChange {
                    fill_type: "WHEAT".to_string(),
                    amount: 999.0,
                }]),
                water: None,
                meadow: None,
            },
            HusbandryChange {
                placeable_index: barn_index,
                clusters: None,
                food: None,
                water: Some(vec![FillStockChange {
                    fill_type: "WATER".to_string(),
                    amount: 250.0,
                }]),
                meadow: Some(vec![FillStockChange {
                    fill_type: "GRASS_WINDROW".to_string(),
                    amount: 100.0,
                }]),
            },
        ];
        write_husbandry_changes(&save, &changes).unwrap();

        let after = parse_husbandries(&save).unwrap();
        let coop = husbandry_named(&after, "Chicken Barn Big");
        assert!((coop.food[0].amount - 999.0).abs() < 0.01);

        let barn = husbandry_named(&after, "Sheep Barn Small");
        assert!((barn.water[0].amount - 250.0).abs() < 0.01);
        assert!((barn.meadow[0].amount - 100.0).abs() < 0.01);
        // Capacity is not editable and must survive untouched.
        assert!((barn.meadow[0].capacity - 25693.414062).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&save);
    }

    #[test]
    fn test_write_husbandry_does_not_touch_silo_or_production_stock() {
        // Fixture index 0 is a silo holding a WHEAT <node>; index 2 is a production point
        // holding a WHEAT <node>. Both share the tag names husbandry water uses.
        let save = setup_fixture("scoping");
        let all = parse_husbandries(&save).unwrap();
        let barn_index = husbandry_named(&all, "Sheep Barn Small").placeable_index;

        let changes = vec![HusbandryChange {
            placeable_index: barn_index,
            clusters: None,
            food: None,
            water: Some(vec![FillStockChange {
                fill_type: "WHEAT".to_string(),
                amount: 1.0,
            }]),
            meadow: None,
        }];
        write_husbandry_changes(&save, &changes).unwrap();

        let raw = std::fs::read_to_string(save.join("placeables.xml")).unwrap();
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"20657.392578\""),
            "silo stock was modified"
        );
        assert!(
            raw.contains("fillType=\"WHEAT\" fillLevel=\"2500.000000\""),
            "production stock was modified"
        );
        let _ = std::fs::remove_dir_all(&save);
    }
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cd src-tauri && cargo test writers::husbandry`
Expected: FAIL — `test_write_food_water_and_meadow` reports the food amount still at 5400
instead of 999. `test_write_husbandry_does_not_touch_silo_or_production_stock` will already
pass, because Task 8 patches nothing; it becomes meaningful once Step 3 lands, and must
still pass afterwards.

- [ ] **Step 3: Add stock patching**

Extend the import:

```rust
use crate::models::changes::{AnimalClusterChange, FillStockChange, HusbandryChange};
```

and drop the `#[allow(dead_code)]` above `attr_str`, which is now used.

Add the context flags next to `let mut in_clusters = false;`:

```rust
    // <storage>/<node> are shared with silos and production points, so patching is
    // gated on being inside <husbandry>.
    let mut in_husbandry = false;
    let mut in_husbandry_storage = false;
    let mut in_food = false;
    let mut in_meadow = false;
```

Extend the start-tag context match — the `b"placeable"` arm resets all four, plus:

```rust
                    b"husbandry" => in_husbandry = !is_empty,
                    b"storage" if in_husbandry => in_husbandry_storage = !is_empty,
                    b"husbandryFood" => in_food = !is_empty,
                    b"husbandryMeadow" => in_meadow = !is_empty,
```

Extend the `Event::End` match:

```rust
                    b"placeable" => {
                        current = None;
                        in_husbandry = false;
                        in_husbandry_storage = false;
                        in_food = false;
                        in_meadow = false;
                    }
                    b"husbandry" => {
                        in_husbandry = false;
                        in_husbandry_storage = false;
                    }
                    b"storage" if in_husbandry => in_husbandry_storage = false,
                    b"husbandryFood" => in_food = false,
                    b"husbandryMeadow" => in_meadow = false,
```

Add three arms to the `patched` match, before the `_ =>` fallback:

```rust
                    (Some(change), b"fillLevel") if in_food => {
                        find_stock(change.food.as_ref(), &attr_str(e, "fillType"))
                            .map(|sc| patch_amount(e, "fillLevel", "fillLevel", sc))
                    }
                    (Some(change), b"node") if in_husbandry_storage => {
                        find_stock(change.water.as_ref(), &attr_str(e, "fillType"))
                            .map(|sc| patch_amount(e, "node", "fillLevel", sc))
                    }
                    (Some(change), b"fillType") if in_meadow => {
                        find_stock(change.meadow.as_ref(), &attr_str(e, "name"))
                            .map(|sc| patch_amount(e, "fillType", "fillLevel", sc))
                    }
```

Add the two helpers next to `patch_animal`:

```rust
fn find_stock<'a>(
    stocks: Option<&'a Vec<FillStockChange>>,
    fill_type: &str,
) -> Option<&'a FillStockChange> {
    stocks?.iter().find(|s| s.fill_type == fill_type)
}

/// Rewrites the amount attribute, preserving every other attribute — notably the
/// meadow's `capacity`, which is not editable.
fn patch_amount(
    e: &BytesStart,
    tag_name: &str,
    amount_key: &str,
    change: &FillStockChange,
) -> BytesStart<'static> {
    let mut elem = BytesStart::new(tag_name.to_string());
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        if key == amount_key {
            elem.push_attribute((amount_key, format!("{:.6}", change.amount).as_str()));
        } else {
            elem.push_attribute((
                key.as_str(),
                String::from_utf8_lossy(&attr.value).as_ref(),
            ));
        }
    }
    elem
}
```

- [ ] **Step 4: Run the tests**

Run: `cd src-tauri && cargo test writers::husbandry`
Expected: PASS — 5 tests. If
`test_write_husbandry_does_not_touch_silo_or_production_stock` now fails, the `in_husbandry`
gating is wrong — fix the writer, never the test.

Then: `cd src-tauri && cargo test`
Expected: PASS — 164 tests, no warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/writers/husbandry.rs
git commit -m "feat(husbandry): write food, water and pasture stock changes"
```

---

## Task 10: Wire the writer into save_changes

**Files:**
- Modify: `src-tauri/src/commands/savegame.rs`

- [ ] **Step 1: Add the write block**

Immediately after the `// Apply placeable changes` block (around lines 429–444), add:

```rust
    // Apply husbandry changes — second pass over placeables.xml, after the placeable pass
    if let Some(ref husbandry_changes) = changes.husbandries {
        match writers::husbandry::write_husbandry_changes(&save_path, husbandry_changes) {
            Ok(()) => {
                if !files_modified.contains(&"placeables.xml".to_string()) {
                    files_modified.push("placeables.xml".to_string());
                }
            }
            Err(e) => errors.push(
                LocalizedMessage::new("errors.fileWriteError")
                    .with_param("file", "placeables.xml")
                    .with_param("details", e),
            ),
        }
    }
```

Ordering matters: it must come **after** the placeable block so it reads the file that pass
wrote.

- [ ] **Step 2: Run the tests**

Run: `cd src-tauri && cargo test`
Expected: PASS — 164 tests, no warnings.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/savegame.rs
git commit -m "feat(husbandry): apply husbandry changes on save"
```

---

## Task 11: Frontend types

**Files:**
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add the interfaces**

Add near the `Placeable` interfaces:

```ts
export interface Husbandry {
  placeableIndex: number;
  displayName: string;
  farmId: number;
  /** How well the husbandry is producing. Read-only. */
  productionFactor: number;
  clusters: AnimalCluster[];
  /** No capacity is recorded in the savegame. */
  food: FillStock[];
  water: FillStock[];
  /** The only stock that carries a capacity. */
  meadow: MeadowStock[];
}

export interface AnimalCluster {
  index: number;
  subType: string;
  numAnimals: number;
  /** Age in months. */
  age: number;
  health: number;
  reproduction: number;
}

export interface FillStock {
  fillType: string;
  amount: number;
}

export interface MeadowStock {
  fillType: string;
  amount: number;
  capacity: number;
}

export interface HusbandryChangePayload {
  placeableIndex: number;
  clusters?: AnimalClusterChangePayload[];
  food?: FillStockChangePayload[];
  water?: FillStockChangePayload[];
  meadow?: FillStockChangePayload[];
}

export interface AnimalClusterChangePayload {
  index: number;
  numAnimals?: number;
  age?: number;
  health?: number;
  reproduction?: number;
}

export interface FillStockChangePayload {
  fillType: string;
  amount: number;
}
```

Add `husbandries: Husbandry[];` to the `SavegameData` interface, and
`husbandries?: HusbandryChangePayload[];` to the `SavegameChanges` interface.

- [ ] **Step 2: Type check**

Run: `pnpm lint`
Expected: PASS, no output.

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat(husbandry): add frontend husbandry types"
```

---

## Task 12: Animal store

**Files:**
- Create: `src/stores/animal.ts`

- [ ] **Step 1: Create the store**

Modelled on `src/stores/building.ts` so it honours the same contract.

```ts
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  AnimalClusterChangePayload,
  FillStock,
  FillStockChangePayload,
  Husbandry,
  HusbandryChangePayload,
  MeadowStock,
} from "@/lib/types";

type StockKind = "food" | "water" | "meadow";

export const useAnimalStore = defineStore("animal", () => {
  const husbandries = ref<Husbandry[]>([]);
  const originalHusbandries = ref<Husbandry[]>([]);

  const totalAnimals = computed(() =>
    husbandries.value.reduce(
      (sum, h) => sum + h.clusters.reduce((s, c) => s + c.numAnimals, 0),
      0,
    ),
  );

  function findOriginal(placeableIndex: number): Husbandry | undefined {
    return originalHusbandries.value.find((h) => h.placeableIndex === placeableIndex);
  }

  function stocksOf(h: Husbandry, kind: StockKind): (FillStock | MeadowStock)[] {
    if (kind === "food") return h.food;
    if (kind === "water") return h.water;
    return h.meadow;
  }

  function isHusbandryModified(h: Husbandry, orig: Husbandry): boolean {
    const clusterChanged = h.clusters.some((c, i) => {
      const o = orig.clusters[i];
      if (!o) return false;
      return (
        c.numAnimals !== o.numAnimals ||
        c.age !== o.age ||
        c.health !== o.health ||
        c.reproduction !== o.reproduction
      );
    });
    if (clusterChanged) return true;

    return (["food", "water", "meadow"] as StockKind[]).some((kind) =>
      stocksOf(h, kind).some((s, i) => s.amount !== stocksOf(orig, kind)[i]?.amount),
    );
  }

  const isDirty = computed(() =>
    husbandries.value.some((h) => {
      const orig = findOriginal(h.placeableIndex);
      return orig ? isHusbandryModified(h, orig) : false;
    }),
  );

  const changeCount = computed(() => {
    let count = 0;
    for (const h of husbandries.value) {
      const orig = findOriginal(h.placeableIndex);
      if (!orig) continue;
      h.clusters.forEach((c, i) => {
        const o = orig.clusters[i];
        if (!o) return;
        if (c.numAnimals !== o.numAnimals) count++;
        if (c.age !== o.age) count++;
        if (c.health !== o.health) count++;
        if (c.reproduction !== o.reproduction) count++;
      });
      for (const kind of ["food", "water", "meadow"] as StockKind[]) {
        stocksOf(h, kind).forEach((s, i) => {
          if (s.amount !== stocksOf(orig, kind)[i]?.amount) count++;
        });
      }
    }
    return count;
  });

  function hydrate(data: Husbandry[]) {
    const serialized = JSON.stringify(data ?? []);
    husbandries.value = JSON.parse(serialized);
    originalHusbandries.value = JSON.parse(serialized);
  }

  function updateCluster(
    placeableIndex: number,
    clusterIndex: number,
    field: "numAnimals" | "age" | "health" | "reproduction",
    value: number,
  ) {
    const h = husbandries.value.find((x) => x.placeableIndex === placeableIndex);
    const cluster = h?.clusters.find((c) => c.index === clusterIndex);
    if (cluster) cluster[field] = value;
  }

  function updateStock(
    placeableIndex: number,
    kind: StockKind,
    fillType: string,
    amount: number,
  ) {
    const h = husbandries.value.find((x) => x.placeableIndex === placeableIndex);
    if (!h) return;
    const stock = stocksOf(h, kind).find((s) => s.fillType === fillType);
    if (stock) stock.amount = amount;
  }

  /** Original value of a field, used by the UI to flag suspiciously large edits. */
  function originalClusterValue(
    placeableIndex: number,
    clusterIndex: number,
    field: "numAnimals" | "age" | "health" | "reproduction",
  ): number | undefined {
    return findOriginal(placeableIndex)?.clusters.find((c) => c.index === clusterIndex)?.[field];
  }

  function originalStockValue(
    placeableIndex: number,
    kind: StockKind,
    fillType: string,
  ): number | undefined {
    const orig = findOriginal(placeableIndex);
    if (!orig) return undefined;
    return stocksOf(orig, kind).find((s) => s.fillType === fillType)?.amount;
  }

  function resetChanges() {
    husbandries.value = JSON.parse(JSON.stringify(originalHusbandries.value));
  }

  function getChanges(): HusbandryChangePayload[] | null {
    const changes: HusbandryChangePayload[] = [];

    for (const h of husbandries.value) {
      const orig = findOriginal(h.placeableIndex);
      if (!orig || !isHusbandryModified(h, orig)) continue;

      const change: HusbandryChangePayload = { placeableIndex: h.placeableIndex };

      const clusters: AnimalClusterChangePayload[] = [];
      h.clusters.forEach((c, i) => {
        const o = orig.clusters[i];
        if (!o) return;
        const entry: AnimalClusterChangePayload = { index: c.index };
        let touched = false;
        if (c.numAnimals !== o.numAnimals) { entry.numAnimals = c.numAnimals; touched = true; }
        if (c.age !== o.age) { entry.age = c.age; touched = true; }
        if (c.health !== o.health) { entry.health = c.health; touched = true; }
        if (c.reproduction !== o.reproduction) { entry.reproduction = c.reproduction; touched = true; }
        if (touched) clusters.push(entry);
      });
      if (clusters.length > 0) change.clusters = clusters;

      for (const kind of ["food", "water", "meadow"] as StockKind[]) {
        const dirty: FillStockChangePayload[] = stocksOf(h, kind)
          .filter((s, i) => s.amount !== stocksOf(orig, kind)[i]?.amount)
          .map((s) => ({ fillType: s.fillType, amount: s.amount }));
        if (dirty.length > 0) change[kind] = dirty;
      }

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalHusbandries.value = JSON.parse(JSON.stringify(husbandries.value));
  }

  return {
    husbandries,
    totalAnimals,
    isDirty,
    changeCount,
    hydrate,
    updateCluster,
    updateStock,
    originalClusterValue,
    originalStockValue,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
```

- [ ] **Step 2: Type check**

Run: `pnpm lint`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/stores/animal.ts
git commit -m "feat(husbandry): add animal store with dirty tracking"
```

---

## Task 13: Husbandry card component

**Files:**
- Create: `src/components/animals/HusbandryCard.vue`

- [ ] **Step 1: Create the component**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useAnimalStore } from "@/stores/animal";
import type { FillStock, Husbandry, MeadowStock } from "@/lib/types";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Separator } from "@/components/ui/separator";

const props = defineProps<{ husbandry: Husbandry }>();

const { t, te } = useI18n();
const store = useAnimalStore();

const totalAnimals = computed(() =>
  props.husbandry.clusters.reduce((sum, c) => sum + c.numAnimals, 0),
);

const isEmpty = computed(() => props.husbandry.clusters.length === 0);

/** Falls back to the raw code so DLC and modded species still display. */
function animalName(subType: string): string {
  const key = `animalTypes.${subType}`;
  return te(key) ? t(key) : subType;
}

function fillName(fillType: string): string {
  const key = `fillTypes.${fillType}`;
  return te(key) ? t(key) : fillType;
}

function ownerLabel(farmId: number): string {
  if (farmId === 0) return t("field.free");
  if (farmId === 1) return t("field.myFarm");
  return t("field.farm", { id: farmId });
}

/**
 * The savegame records no capacity for headcount, food or water, so no real maximum
 * can be enforced. Flag values well past the original instead of blocking them.
 */
function isSuspicious(current: number, original: number | undefined): boolean {
  if (original === undefined) return false;
  if (original === 0) return current > 0;
  return current > original * 2;
}

function warnClass(current: number, original: number | undefined): string {
  return isSuspicious(current, original)
    ? "border-amber-500 focus-visible:ring-amber-500"
    : "";
}

function onClusterInput(
  clusterIndex: number,
  field: "numAnimals" | "age",
  event: Event,
) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value >= 0) {
    store.updateCluster(props.husbandry.placeableIndex, clusterIndex, field, Math.round(value));
  }
}

function onClusterSlider(
  clusterIndex: number,
  field: "health" | "reproduction",
  value: number[] | undefined,
) {
  if (value?.length) {
    store.updateCluster(props.husbandry.placeableIndex, clusterIndex, field, value[0]);
  }
}

function onStockInput(kind: "food" | "water", fillType: string, event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  if (Number.isFinite(value) && value >= 0) {
    store.updateStock(props.husbandry.placeableIndex, kind, fillType, value);
  }
}

function onMeadowSlider(fillType: string, value: number[] | undefined) {
  if (value?.length) {
    store.updateStock(props.husbandry.placeableIndex, "meadow", fillType, value[0]);
  }
}

function originalCluster(
  clusterIndex: number,
  field: "numAnimals" | "age" | "health" | "reproduction",
) {
  return store.originalClusterValue(props.husbandry.placeableIndex, clusterIndex, field);
}

function originalStock(kind: "food" | "water", fillType: string) {
  return store.originalStockValue(props.husbandry.placeableIndex, kind, fillType);
}

function stockLabel(stock: FillStock | MeadowStock): string {
  return fillName(stock.fillType);
}
</script>

<template>
  <Card :class="isEmpty ? 'opacity-60' : ''">
    <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-3">
      <div>
        <p class="font-medium">{{ husbandry.displayName }}</p>
        <p class="text-sm text-muted-foreground">
          <template v-if="isEmpty">{{ t("animal.empty") }}</template>
          <template v-else>
            {{ t("animal.headcount", { count: totalAnimals }) }}
            <span v-if="husbandry.clusters.length > 1">
              · {{ t("animal.clusterCount", { count: husbandry.clusters.length }) }}
            </span>
          </template>
        </p>
      </div>
      <Badge variant="outline">{{ ownerLabel(husbandry.farmId) }}</Badge>
    </CardHeader>

    <CardContent v-if="!isEmpty" class="space-y-5">
      <!-- Clusters -->
      <div
        v-for="cluster in husbandry.clusters"
        :key="cluster.index"
        class="space-y-3 rounded-md border p-3"
      >
        <p class="text-sm font-medium">
          {{ animalName(cluster.subType) }}
          <span class="text-muted-foreground">
            · {{ t("animal.ageMonths", { count: cluster.age }) }}
          </span>
        </p>

        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1">
            <Label class="text-xs">{{ t("animal.numAnimals") }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="cluster.numAnimals"
              :class="warnClass(cluster.numAnimals, originalCluster(cluster.index, 'numAnimals'))"
              @change="onClusterInput(cluster.index, 'numAnimals', $event)"
            />
          </div>
          <div class="space-y-1">
            <Label class="text-xs">{{ t("animal.age") }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="cluster.age"
              :class="warnClass(cluster.age, originalCluster(cluster.index, 'age'))"
              @change="onClusterInput(cluster.index, 'age', $event)"
            />
          </div>
        </div>

        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs">
            <Label>{{ t("animal.health") }}</Label>
            <span class="font-mono text-muted-foreground">{{ Math.round(cluster.health) }}</span>
          </div>
          <Slider
            :model-value="[cluster.health]"
            :min="0"
            :max="100"
            :step="1"
            @update:model-value="onClusterSlider(cluster.index, 'health', $event)"
          />
        </div>

        <div class="space-y-1">
          <div class="flex items-center justify-between text-xs">
            <Label>{{ t("animal.reproduction") }}</Label>
            <span class="font-mono text-muted-foreground">
              {{ Math.round(cluster.reproduction) }}
            </span>
          </div>
          <Slider
            :model-value="[cluster.reproduction]"
            :min="0"
            :max="100"
            :step="1"
            @update:model-value="onClusterSlider(cluster.index, 'reproduction', $event)"
          />
        </div>
      </div>

      <!-- Stocks -->
      <template v-if="husbandry.food.length || husbandry.water.length || husbandry.meadow.length">
        <Separator />
        <div class="space-y-3">
          <p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {{ t("animal.stocks") }}
          </p>

          <div
            v-for="stock in husbandry.food"
            :key="`food-${stock.fillType}`"
            class="grid grid-cols-2 items-center gap-2"
          >
            <Label class="text-sm">{{ stockLabel(stock) }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="Math.round(stock.amount)"
              :class="warnClass(stock.amount, originalStock('food', stock.fillType))"
              @change="onStockInput('food', stock.fillType, $event)"
            />
          </div>

          <div
            v-for="stock in husbandry.water"
            :key="`water-${stock.fillType}`"
            class="grid grid-cols-2 items-center gap-2"
          >
            <Label class="text-sm">{{ stockLabel(stock) }}</Label>
            <Input
              type="number"
              min="0"
              step="1"
              :model-value="Math.round(stock.amount)"
              :class="warnClass(stock.amount, originalStock('water', stock.fillType))"
              @change="onStockInput('water', stock.fillType, $event)"
            />
          </div>

          <!-- Pasture is the only stock with a real capacity, so it gets a true slider. -->
          <div v-for="stock in husbandry.meadow" :key="`meadow-${stock.fillType}`" class="space-y-1">
            <div class="flex items-center justify-between text-sm">
              <Label>{{ stockLabel(stock) }}</Label>
              <span class="font-mono text-xs text-muted-foreground">
                {{ Math.round(stock.amount) }} / {{ Math.round(stock.capacity) }}
              </span>
            </div>
            <Slider
              :model-value="[stock.amount]"
              :min="0"
              :max="stock.capacity"
              :step="1"
              @update:model-value="onMeadowSlider(stock.fillType, $event)"
            />
          </div>
        </div>
      </template>
    </CardContent>
  </Card>
</template>
```

- [ ] **Step 2: Type check**

Run: `pnpm lint`
Expected: PASS. If `Card`/`CardHeader`/`CardContent` are not exported from
`@/components/ui/card`, check the actual exports in that folder and adjust the import.

- [ ] **Step 3: Commit**

```bash
git add src/components/animals/HusbandryCard.vue
git commit -m "feat(husbandry): add husbandry card with in-place editing"
```

---

## Task 14: View, route, sidebar and hydration

**Files:**
- Create: `src/views/AnimalListView.vue`
- Modify: `src/router/index.ts`
- Modify: `src/components/layout/AppSidebar.vue`
- Modify: `src/components/layout/EditorLayout.vue`
- Modify: `src/composables/useUnsavedChanges.ts`

- [ ] **Step 1: Create the view**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useAnimalStore } from "@/stores/animal";
import HusbandryCard from "@/components/animals/HusbandryCard.vue";
import { PiggyBank } from "lucide-vue-next";

const { t } = useI18n();
const store = useAnimalStore();

const sorted = computed(() =>
  [...store.husbandries].sort((a, b) => a.placeableIndex - b.placeableIndex),
);
</script>

<template>
  <div class="space-y-6">
    <div>
      <h2 class="text-2xl font-semibold">{{ t("animal.title") }}</h2>
      <p class="text-sm text-muted-foreground">
        {{ t("animal.subtitle", { count: store.totalAnimals }) }}
      </p>
    </div>

    <div v-if="sorted.length === 0" class="py-12 text-center">
      <PiggyBank class="mx-auto mb-3 size-10 text-muted-foreground" />
      <p class="text-muted-foreground">{{ t("animal.none") }}</p>
    </div>

    <div v-else class="grid gap-4 lg:grid-cols-2">
      <HusbandryCard
        v-for="husbandry in sorted"
        :key="husbandry.placeableIndex"
        :husbandry="husbandry"
      />
    </div>
  </div>
</template>
```

- [ ] **Step 2: Add the route**

In `src/router/index.ts`, inside the `/editor` children array, after the `buildings` entry:

```ts
        {
          path: "animals",
          name: "animals",
          component: () => import("@/views/AnimalListView.vue"),
        },
```

- [ ] **Step 3: Add the sidebar entry**

In `src/components/layout/AppSidebar.vue`, add `PiggyBank` to the `lucide-vue-next` import
list, then add to `phase2Nav` after the `buildings` entry:

```ts
  { labelKey: "sidebar.animals", icon: PiggyBank, route: "/editor/animals" },
```

- [ ] **Step 4: Hydrate the store**

In `src/components/layout/EditorLayout.vue`, add the import next to the other stores:

```ts
import { useAnimalStore } from "@/stores/animal";
```

and instantiate it next to the others:

```ts
const animalStore = useAnimalStore();
```

then after `buildingStore.hydrate(store.currentSavegame.placeables);`:

```ts
    animalStore.hydrate(store.currentSavegame.husbandries);
```

- [ ] **Step 5: Aggregate dirty state**

In `src/composables/useUnsavedChanges.ts`:

Add the import:

```ts
import { useAnimalStore } from "@/stores/animal";
```

Instantiate inside `useUnsavedChanges()`:

```ts
  const animalStore = useAnimalStore();
```

Add `animalStore.isDirty ||` to the `isDirty` computed, `animalStore.changeCount +` to the
`changeCount` computed, and inside `collectAllChanges()` after the building block:

```ts
    const animalChanges = animalStore.getChanges();
    if (animalChanges) changes.husbandries = animalChanges;
```

and inside `resetAll()`:

```ts
    animalStore.resetChanges();
```

- [ ] **Step 6: Type check**

Run: `pnpm lint`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/views/AnimalListView.vue src/router/index.ts src/components/layout/AppSidebar.vue src/components/layout/EditorLayout.vue src/composables/useUnsavedChanges.ts
git commit -m "feat(husbandry): add animals view, route and store wiring"
```

---

## Task 15: Translations

**Files:**
- Modify: `src/locales/fr.json`
- Modify: `src/locales/en.json`

- [ ] **Step 1: Add French strings**

In `src/locales/fr.json`, add `"animals": "Élevages"` inside the existing `sidebar` object,
then add these two top-level objects (merge `fillTypes` into the existing object if there
already is one):

```json
  "animal": {
    "title": "Élevages",
    "subtitle": "{count} animaux au total",
    "none": "Aucun élevage dans cette sauvegarde",
    "empty": "Vide",
    "headcount": "{count} animaux",
    "clusterCount": "{count} lots",
    "ageMonths": "{count} mois",
    "numAnimals": "Nombre",
    "age": "Âge (mois)",
    "health": "Santé",
    "reproduction": "Reproduction",
    "stocks": "Stocks"
  },
  "animalTypes": {
    "CHICKEN": "Poule",
    "GOAT": "Chèvre"
  },
  "fillTypes": {
    "WHEAT": "Blé",
    "WATER": "Eau",
    "GRASS_WINDROW": "Herbe"
  },
```

- [ ] **Step 2: Add English strings**

In `src/locales/en.json`, add `"animals": "Husbandries"` inside `sidebar`, then:

```json
  "animal": {
    "title": "Husbandries",
    "subtitle": "{count} animals in total",
    "none": "No husbandry in this savegame",
    "empty": "Empty",
    "headcount": "{count} animals",
    "clusterCount": "{count} groups",
    "ageMonths": "{count} months",
    "numAnimals": "Headcount",
    "age": "Age (months)",
    "health": "Health",
    "reproduction": "Reproduction",
    "stocks": "Stocks"
  },
  "animalTypes": {
    "CHICKEN": "Chicken",
    "GOAT": "Goat"
  },
  "fillTypes": {
    "WHEAT": "Wheat",
    "WATER": "Water",
    "GRASS_WINDROW": "Grass"
  },
```

> Only species actually observed in a real save are listed. `animalName()` falls back to the
> raw code, so DLC and modded species still display; add entries as they are confirmed.

- [ ] **Step 3: Verify both files parse and the app type-checks**

Run: `node -e "JSON.parse(require('fs').readFileSync('src/locales/fr.json','utf8')); JSON.parse(require('fs').readFileSync('src/locales/en.json','utf8')); console.log('JSON ok')"`
Expected: `JSON ok`

Run: `pnpm lint`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/locales/fr.json src/locales/en.json
git commit -m "feat(husbandry): add French and English translations"
```

---

## Task 16: Re-hydrate buildings, missions and animals on reload

`handleReload` in `AppHeader.vue` re-hydrates only some stores, so after "reload from disk"
the Buildings and Missions views keep showing stale data. Fixing it here prevents the new
Animals view from inheriting the same defect.

**Files:**
- Modify: `src/components/layout/AppHeader.vue`

- [ ] **Step 1: Add the missing hydration calls**

Add the store imports if absent:

```ts
import { useBuildingStore } from "@/stores/building";
import { useMissionStore } from "@/stores/mission";
import { useAnimalStore } from "@/stores/animal";
```

and instantiate them next to the other stores:

```ts
const buildingStore = useBuildingStore();
const missionStore = useMissionStore();
const animalStore = useAnimalStore();
```

Then inside `handleReload`, after
`fieldStore.hydrate(savegameStore.currentSavegame.fields, savegameStore.currentSavegame.farmlands);`:

```ts
      buildingStore.hydrate(savegameStore.currentSavegame.placeables);
      missionStore.hydrate(
        savegameStore.currentSavegame.missions,
        savegameStore.currentSavegame.collectibles,
        savegameStore.currentSavegame.contractSettings,
      );
      animalStore.hydrate(savegameStore.currentSavegame.husbandries);
```

- [ ] **Step 2: Type check**

Run: `pnpm lint`
Expected: PASS.

- [ ] **Step 3: Manual verification**

Run: `pnpm tauri:dev`

1. Open a savegame, go to Élevages — the herd appears with correct counts.
2. Change a headcount to more than double its original — the input border turns amber, and
   the value is still accepted.
3. Save, then use "reload from disk" — the new value persists and the view refreshes.
4. Open the saved `placeables.xml` and confirm the silo and production `<node>` fill levels
   are unchanged.

- [ ] **Step 4: Commit**

```bash
git add src/components/layout/AppHeader.vue
git commit -m "fix(editor): re-hydrate buildings, missions and animals on reload"
```

---

## Task 17: Extract the duplicated attribute helpers

Added during execution. The code quality review of Task 4 flagged that
`attr_str` / `attr_f64` / `attr_u8` / `attr_u32` are now byte-identical in three parsers
(`placeable.rs`, `vehicle.rs`, `husbandry.rs`). It was deferred out of Task 4 because it
touches two files outside that task's blast radius; doing it as its own commit keeps both
changes atomic.

**Files:**
- Create: `src-tauri/src/parsers/attrs.rs`
- Modify: `src-tauri/src/parsers/mod.rs`
- Modify: `src-tauri/src/parsers/placeable.rs`, `vehicle.rs`, `husbandry.rs`

- [ ] **Step 1: Create the shared module**

```rust
//! Attribute readers shared by the savegame XML parsers.
//!
//! Every reader is total: a missing or unparseable attribute yields the type's
//! zero value rather than an error, because savegame attributes are frequently
//! optional and a missing one is not a parse failure.

use quick_xml::events::BytesStart;

pub fn attr_str(e: &BytesStart, key: &str) -> String {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key.as_bytes())
        .map(|a| String::from_utf8_lossy(&a.value).to_string())
        .unwrap_or_default()
}

pub fn attr_f64(e: &BytesStart, key: &str) -> f64 {
    attr_str(e, key).parse().unwrap_or(0.0)
}

pub fn attr_u8(e: &BytesStart, key: &str) -> u8 {
    attr_str(e, key).parse().unwrap_or(0)
}

pub fn attr_u32(e: &BytesStart, key: &str) -> u32 {
    attr_str(e, key).parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element() -> BytesStart<'static> {
        let mut e = BytesStart::new("sample");
        e.push_attribute(("name", "WHEAT"));
        e.push_attribute(("amount", "12.5"));
        e.push_attribute(("bad", "not-a-number"));
        e
    }

    #[test]
    fn test_attr_str_reads_and_defaults_to_empty() {
        assert_eq!(attr_str(&element(), "name"), "WHEAT");
        assert_eq!(attr_str(&element(), "missing"), "");
    }

    #[test]
    fn test_numeric_readers_default_to_zero_on_missing_or_malformed() {
        assert!((attr_f64(&element(), "amount") - 12.5).abs() < 0.001);
        assert!((attr_f64(&element(), "missing") - 0.0).abs() < 0.001);
        assert!((attr_f64(&element(), "bad") - 0.0).abs() < 0.001);
        assert_eq!(attr_u32(&element(), "missing"), 0);
        assert_eq!(attr_u8(&element(), "bad"), 0);
    }
}
```

- [ ] **Step 2: Run the new tests**

Add `pub mod attrs;` to `src-tauri/src/parsers/mod.rs`.

Run: `cd src-tauri && cargo test parsers::attrs`
Expected: PASS — 2 tests.

- [ ] **Step 3: Replace the three copies**

In each of `placeable.rs`, `vehicle.rs` and `husbandry.rs`: delete the local `attr_*`
definitions and add `use crate::parsers::attrs::{attr_f64, attr_str, attr_u8, attr_u32};`,
importing only the readers that file actually uses (an unused import is a warning).

Note `vehicle.rs` and `husbandry.rs` type their helpers against
`quick_xml::events::BytesStart` via the fully qualified path; the shared module imports the
type directly, which is equivalent.

- [ ] **Step 4: Verify nothing changed behaviourally**

Run: `cd src-tauri && cargo test`
Expected: PASS — same count as before this task, zero warnings. Any test that changes result
means the extraction was not behaviour-preserving; fix the extraction, not the test.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/parsers/
git commit -m "refactor(parsers): extract shared XML attribute readers"
```

---

## Definition of Done

- `cd src-tauri && cargo test` — all green, no warnings
- `pnpm lint` — clean
- Élevages appears in the sidebar and lists every husbandry, empty ones dimmed
- Editing headcount, age, health, reproduction and stocks marks the editor dirty and
  survives save + reload
- Silo and production-point stocks are provably untouched by husbandry edits
- Both locales complete; unknown species fall back to their raw code
