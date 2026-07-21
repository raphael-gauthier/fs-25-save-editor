# Husbandry (Animals) View — Design

**Date:** 2026-07-21
**Status:** Approved, pending implementation plan

## Overview

Add an "Élevages" (Husbandry) section that lets the player inspect and adjust the animals
they already own: herd size, age, health, reproduction, and the food, water and pasture
stocks feeding them.

Husbandry data lives in `placeables.xml` but is invisible in the editor today. In the
reference savegame it covers 451 animals across 3 husbandry placeables.

## Savegame Format

Verified against `docs/savegames/placeables.xml`.

```xml
<placeable filename="…/chickenBarnBig.xml" uniqueId="placeable2aa7…" farmId="1" …>
    <husbandry globalProductionFactor="1.000000" productionFactor="1.000000">
        <storage farmId="1">
            <node fillType="WATER" fillLevel="1500.000000"/>
        </storage>
    </husbandry>
    <husbandryAnimals>
        <clusters>
            <animal subType="CHICKEN" numAnimals="360" age="48" health="100" reproduction="0"/>
        </clusters>
    </husbandryAnimals>
    <husbandryFood>
        <fillLevel fillType="WHEAT" fillLevel="5400.000000"/>
    </husbandryFood>
    <husbandryMeadow>
        <fillType name="GRASS_WINDROW" fillLevel="21689.705078" capacity="25693.414062"/>
    </husbandryMeadow>
</placeable>
```

Observations that drive the design:

- Animals are grouped into **clusters** — one cluster per same-age batch. A single husbandry
  can hold several (the sheep barn holds two goat clusters).
- The building name does **not** determine the species: `sheepBarnSmall` contains goats.
- An empty pre-placed husbandry appears as `<husbandry globalProductionFactor="0" …/>`
  with no `<husbandryAnimals>` and **no `filename` attribute**.
- `<animal>`, `<fillLevel>` and `<node>` are all **self-closing** elements.
- Only `husbandryMeadow` records a `capacity`. Food, water and animal counts have no known
  upper bound in the savegame — those limits live in the game's own placeable XML files.

## Scope

**In scope** — adjusting what already exists:

- Per cluster: number of animals, age, health, reproduction
- Food, water and pasture fill levels
- Read-only: production factor, owning farm

**Out of scope** — creating entities:

- Adding or removing clusters
- Changing an animal's species
- Creating a husbandry

Rationale: a species incompatible with its building can break savegame loading, and the
per-building whitelist of valid `subType` values is not in the savegame. This can be
revisited once recipe/limit resolution from the game install exists.

## Backend Architecture

Husbandries get their own model, parser and writer rather than extending the placeable
chain:

```
models/husbandry.rs    Husbandry, AnimalCluster, FillStock, MeadowStock
parsers/husbandry.rs   second pass over placeables.xml → Vec<Husbandry>
writers/husbandry.rs   patches by (placeable_index, cluster_index)
```

`SavegameData` gains `husbandries: Vec<Husbandry>`; `SavegameChanges` gains
`husbandries: Option<Vec<HusbandryChange>>`.

**Why a separate chain.** If the buildings store and the animals store both emitted
`PlaceableChange` values, `save_changes` would have to merge two change lists targeting the
same placeable index before writing — a silent-corruption risk. Two sequential passes over
`placeables.xml` keep the two features fully decoupled. The extra parse costs milliseconds.

**On pass ordering.** An earlier draft of this spec claimed the husbandry pass had to run
after the placeable pass, or it would read a stale file and discard the building edits. That
was verified during implementation and is **false**. Both writers are streaming copy-through
writers: each patches only the attributes it owns and passes every other XML event through
byte-for-byte, so whichever runs second faithfully preserves the first one's output. Order is
a readability convention here, not a correctness constraint. It is recorded because the
opposite belief would otherwise look like a rule worth defending.

### Model

```rust
pub struct Husbandry {
    pub placeable_index: usize,   // write target, shared numbering with Placeable
    pub display_name: String,
    pub farm_id: u8,
    pub production_factor: f64,   // read-only
    pub clusters: Vec<AnimalCluster>,
    pub food: Vec<FillStock>,     // <husbandryFood><fillLevel>
    pub water: Vec<FillStock>,    // <husbandry><storage><node>
    pub meadow: Vec<MeadowStock>, // <husbandryMeadow><fillType> — has capacity
}

pub struct AnimalCluster {
    pub index: usize,             // positional; <animal> has no id
    pub sub_type: String,
    pub num_animals: u32,
    pub age: u32,
    pub health: f64,
    pub reproduction: f64,
}

pub struct FillStock  { pub fill_type: String, pub amount: f64 }
pub struct MeadowStock { pub fill_type: String, pub amount: f64, pub capacity: f64 }
```

Placeables with no `<husbandry*>` element are skipped. Empty husbandries are kept, with an
empty `clusters` vector, so the UI can show them.

**Index alignment is load-bearing.** `placeable_index` must refer to the same numbering
`parse_placeables` assigns, because it is what the writer uses to find the element again.
The husbandry parser therefore increments its counter on **every** `<placeable>` it walks
past, not only the ones holding a `<husbandry*>` element. A test asserts that the husbandry
of a known placeable reports the index that `parse_placeables` gives that same placeable.

### Shared refactor

`display_name_from_unique_id` moves from `parsers/placeable.rs` to `models/placeable.rs`
and becomes public, so the husbandry parser can name pre-placed husbandries that carry no
`filename` (e.g. `preplaced_chickenCoop02_ec6e2c…` → "Chicken Coop 02").

### Writing

Targeting is positional: `(placeable_index, cluster_index)`. `<animal>` elements carry no
identifier, so parser and writer must count clusters identically.

```rust
pub struct HusbandryChange {
    pub placeable_index: usize,
    pub clusters: Option<Vec<AnimalClusterChange>>,  // matched by cluster index
    pub food: Option<Vec<FillStockChange>>,          // matched by fill type
    pub water: Option<Vec<FillStockChange>>,
    pub meadow: Option<Vec<FillStockChange>>,
}

pub struct AnimalClusterChange {
    pub index: usize,                 // position within <clusters>
    pub num_animals: Option<u32>,
    pub age: Option<u32>,
    pub health: Option<f64>,
    pub reproduction: Option<f64>,
}

pub struct FillStockChange { pub fill_type: String, pub amount: f64 }
```

Every field is optional so an untouched attribute is rewritten verbatim; `sub_type` is absent
by design, since changing species is out of scope.

**Self-closing elements.** `<animal/>`, `<fillLevel/>` and `<node/>` are self-closing, so the
writer must patch them in the `Event::Empty` branch. This is the exact defect that made every
sale edit a no-op before it was fixed; a dedicated test covers it here.

**Shared `<storage>`/`<node>` tags.** These same tag names are used by silos and production
points. The husbandry writer must only patch nodes inside a `<husbandry>` element, mirroring
the scoping already applied to the placeable writer.

## Frontend

- Route `/editor/animals`, sidebar entry "Élevages" (`PiggyBank` icon), placed after Buildings
- `stores/animal.ts` — same contract as the other stores: `hydrate`, `isDirty`, `changeCount`,
  `getChanges`, `resetChanges`, `commitChanges`
- `views/AnimalListView.vue` — list of cards
- `components/animals/HusbandryCard.vue` — one husbandry, edited in place

### Layout

One card per husbandry rather than a table: a save holds only a handful of husbandries (2
active here, against 57 vehicles and 171 placeables), and the data is hierarchical —
husbandry → clusters — which a single table row flattens badly.

Each card shows the name, owning farm and total headcount, then one block per cluster
(headcount, age, health, reproduction) followed by the stocks. Empty husbandries stay
visible, dimmed.

### Editing controls

Controls follow what the savegame actually bounds:

| Field | Control | Bound |
|---|---|---|
| Health, Reproduction | Slider | 0–100, known |
| Pasture | Slider | `capacity` from the save |
| Headcount, Age, Food, Water | Numeric input | none known |

Unbounded fields accept any non-negative value and show an **amber outline past 2× their
original value** — a visual warning, never a block. The editor does not invent a limit it
cannot verify.

## Testing

TDD throughout. The `savegame_complete` fixture gains the three real husbandry shapes:
single-cluster with food, multi-cluster with pasture and water, and an empty pre-placed one
with no `filename`.

Parser tests:
- clusters parsed with all attributes, in document order
- food, water and pasture stocks, pasture keeping its capacity
- display name resolved from `uniqueId` when `filename` is absent
- empty husbandry yields an entry with no clusters
- non-husbandry placeables are skipped

Writer tests:
- edit a cluster's health/headcount
- edit the **second** cluster of a multi-cluster husbandry — catches index drift
- edit food, water and pasture levels
- self-closing `<animal/>` is actually patched
- regression: editing a husbandry leaves silo `<node>` and production-point stocks untouched
- untouched attributes and document shape preserved

Frontend: `pnpm lint` clean; `fr`/`en` locales complete, including species names
(`CHICKEN` → "Poule", `GOAT` → "Chèvre", …) with a fallback to the raw code for unknown
species so DLC and modded animals still display.

## Known Limitation: stocks are keyed by fill type

`FillStockChange` identifies a stock only by `fillType`, and the writer's lookup takes the
first match. If a husbandry ever held two entries of the same fill type in one collection —
two `WHEAT` nodes in `<husbandryFood>`, say — an edit to one would be applied to whichever
appears first, and the frontend would collapse both into a single payload entry.

No such savegame has been observed, and the constraint is inherited from the payload shape
rather than introduced by the UI. Fixing it would mean adding a positional field to
`FillStockChange` and teaching the writer to count entries within a collection, the same way
it already counts clusters. Worth doing if a real save ever exhibits the shape.

Clusters do not have this problem: they carry an explicit positional `index`.

## Known Test Coverage Gap: cross-placeable bleed

If the husbandry writer failed to clear its active change when moving between placeables, one
edit would bleed into every later husbandry in the file. No test catches that, and one was
attempted and honestly abandoned rather than faked.

The reason it cannot be caught: `current` is reassigned on every `<placeable>`, and
`</placeable>` clears it, so staleness can only survive past a **self-closing** `<placeable/>`
— which has no children, therefore no `<husbandry>`, therefore is never itself a husbandry.
Corruption would require a mis-aimed change landing on a self-closing placeable that is
immediately followed by a husbandry. Scanning the reference save: 63 of 171 placeables are
self-closing, and **zero** of them are followed by a husbandry.

Building that adjacency into the fixture was rejected. Both shapes are real, but their
adjacency is not observed, and this project's entire bug history comes from fixtures shaped by
assumption rather than evidence. The mis-aimed-change failure mode is already covered by
`test_write_husbandry_change_aimed_at_a_non_husbandry_is_inert`. If a save is ever captured
with that adjacency, add it and the gap closes.

## Known Test Coverage Gap

The husbandry parser gates water collection on being inside a `<husbandry>` element, because
`<storage>`/`<node>` tag names are shared with silos and production points. With the shapes
observed in a real save, that gate cannot be falsified by a test: a silo or production
placeable never sets `is_husbandry`, so anything wrongly collected is discarded when
`</placeable>` closes.

The gate does protect a real scenario the fixture cannot express — a single placeable that
is a husbandry *and* carries an unrelated `<storage>` elsewhere, such as a husbandry building
still under construction with a `<constructible><storage/>` section. There, `is_husbandry` is
already true, so the discard-on-close safety net does not fire and the gate is the only
protection.

No fixture covers that case, because no such placeable has been observed in a real savegame
and this project has been bitten badly by invented fixture shapes. If one is ever captured,
add it to `savegame_complete/placeables.xml` and assert the foreign storage does not appear
in `water`.

## Known Limitation

Food, water and headcount have no capacity in the savegame, so no true progress bar can be
shown for them and no real maximum can be enforced. Resolving those limits would require
reading the game's own placeable XML files — the same dependency already noted for production
recipes and vehicle fill-unit capacities. Out of scope here.
