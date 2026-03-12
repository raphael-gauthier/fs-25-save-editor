# 06 - Fields and Crops Editing

## Overview

The Fields section displays accurate field data from binary density maps (GDM/GRLE files) rather than the AI/mission summaries in `fields.xml`. It supports both viewing and editing crop types, growth states, treatments, and land ownership.

## Data Sources

### Binary Density Maps (Primary — Accurate)
- **`densityMap_fruits.gdm`** — Fruit type index + growth state per pixel (10 channels)
- **`densityMap_ground.gdm`** — Ground type per pixel (11 channels)
- **`densityMap_weed.gdm`** — Weed state per pixel (4 channels)
- **`densityMap_stones.gdm`** — Stone level per pixel (3 channels)
- **`infoLayer_limeLevel.grle`** — Lime level (2 bits, values 0-3)
- **`infoLayer_sprayLevel.grle`** — Spray/fertilizer level (2 bits, values 0-2)
- **`infoLayer_plowLevel.grle`** — Plow level (1 bit, values 0-1)
- **`infoLayer_rollerLevel.grle`** — Roller level (1 bit, values 0-1)
- **`infoLayer_stubbleShredLevel.grle`** — Stubble shred level (1 bit, values 0-1)
- **`infoLayer_farmlands.grle`** — Pixel → farmland ID mapping (from map data)

### XML Files (Secondary — AI/Mission Data)
- **`fields.xml`** — AI/mission system summaries (fruit type, growth state, treatments)
- **`farmland.xml`** — Land ownership

### Fruit Type Index Resolution (4 sources, in order)
1. **`maps_fruitTypes.xml`** — 25 base game types (indices 1-25)
2. **Map's own XML** (e.g., `mapUS.xml`) — map-specific types (e.g., MEADOW at index 26)
3. **Game log** (`log.txt`) — DLC types (e.g., ONION at index 27+)
4. **Known fallback list** — base game crops that may not appear in log (GREENBEAN, PEA, SPINACH)

## Field List

### Display
Each field is displayed with:
- **Field number** (farmland identifier)
- **Current crop** — dominant fruit from density map (with +N badge if multiple crops)
- **Growth state** — average growth as percentage (progress bar)
- **Ground type** — most common ground type from density map
- **Treatments** — badge showing issues (low lime, weeds, low fertilizer)
- **Owner** — farmland ownership from farmland.xml

### Loading States
- **Cache available**: displays cached data instantly, shows "refreshing" banner
- **No cache, loading**: shows skeleton placeholders for crop/growth/ground columns
- **No game path**: shows info banner, falls back to XML data
- **Error**: shows error banner, keeps cached data if available

### Filtering
- Search by field ID or fruit name
- Filter by crop type (from density data when available, XML otherwise)
- Filter by owner (my fields / all fields)

## Field Editing

### Density Map Editing (Quick Actions)
When density data is available, the editor shows quick action buttons:
- **Max growth** — set all pixels to growth state 10
- **Max lime** — set lime level to maximum (3)
- **Max fertilizer** — set spray level to maximum (2)
- **Max plow** — set plow level to maximum (1)
- **Clear weeds** — set weed level to 0
- **Clear stones** — set stone level to 0

These edits modify the actual binary density map files on save.

### XML Editing (Advanced Mode)
When density data is available, XML editing fields are hidden behind advanced mode.
Without density data, XML editing is always visible.

Editable XML fields:
- **Crop type**: change the current crop among available types
- **Planned crop**: set the next crop to be planted
- **Growth stage**: adjust the stage (0 to 10) with slider
- **Ground type**: change the soil condition

Advanced mode treatment sliders:
- Weed state (0-10), stone level (0-5), spray level (0-3), lime level (0-3)
- Plow level (0-3), roller level (0-3), stubble shred level (0-3), water level (0-3)

## Bulk Actions

Users can select multiple fields and apply batch actions:
- **Max growth** — advance all selected fields to ready-to-harvest stage
- **Remove weeds** — clear weeds on all selected fields
- **Remove stones** — clear stones on all selected fields
- **Max lime** — set lime level to maximum on all selected fields
- **Max fertilizer** — set fertilizer level to maximum on all selected fields

When density data is available, batch actions create both XML changes and density edits.

## Land Ownership

### Viewing
- Display of the farmland parcel list with their current owner

### Editing
- Change the owner of a parcel (transfer to the player's farm or release)
- In advanced mode: assign a parcel to any farm (farm ID 2-6)

## Caching

Density data is cached locally in `density-cache.json` via `@tauri-apps/plugin-store`:
- Keyed by savegame folder name (e.g., `savegame1`)
- Includes data, timestamp, and map ID
- On load: display cached data instantly → refresh from backend → update cache
- If refresh fails: keep cached data with "data may not be up to date" warning
- Cache invalidated when map ID changes

## Supported Maps

- **Built-in maps**: MapUS, MapEU, MapAS — read from `{gamePath}/data/maps/`
- **Modded maps**: read from zip archives in `~/Documents/My Games/FarmingSimulator2025/mods/`
- **DLC maps**: not supported (encrypted `.dlc` files)
