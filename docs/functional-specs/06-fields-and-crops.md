# 06 - Fields and Crops Editing (Phase 2)

## Overview

The Fields section allows viewing and editing the state of each field on the map: crop type, growth stage, soil condition, and various applied treatments.

## Field List

### Display
Each field is displayed with:
- **Field number** (map identifier)
- **Current crop** (wheat, canola, onion, etc.) or "None"
- **Planned crop** (next scheduled crop)
- **Growth stage** (visual representation: progress bar or stage icons)
- **Soil condition** (plowed, cultivated, sown, planted, grass, ready to harvest)
- **Owner** of the associated land

### Filtering
- By crop type
- By growth state (growing, ready to harvest, uncultivated)
- By owner (my fields / all fields)

## Simple Mode - Per-Field Editing

### Crop
- **Crop type**: change the current crop among available types (WHEAT, BARLEY, CANOLA, OAT, CORN, SUNFLOWER, SOYBEAN, POTATO, SUGARBEET, COTTON, SORGHUM, RICE, ONION, CARROT, PARSNIP, BEETROOT, GRASS, etc.)
- **Planned crop**: set the next crop to be planted

### Growth
- **Growth stage**: adjust the stage (0 to 10) with a readable label for each stage
  - 0: Sown
  - 1-3: Growing (early)
  - 4-6: Growing (mid)
  - 7-9: Growing (late)
  - 10: Ready to harvest
- **Quick action "Max growth"**: jump directly to the ready-to-harvest stage

### Soil Condition
- **Soil type**: change the soil condition (PLOWED, CULTIVATED, SOWN, PLANTED, GRASS, HARVEST_READY, etc.)

## Advanced Mode - Per-Field Editing

### Treatment Levels
- **Weed level** (0-9): 0 = no weeds
- **Stone level** (0-3): 0 = no stones
- **Fertilizer/spray level**: amount of product applied
- **Lime level** (0-3): liming status
- **Plowing level**: soil plowing status
- **Rolling level**: soil compaction
- **Stubble shredding level**: shredding status
- **Water level**: irrigation

## Bulk Actions

Users can select multiple fields and apply batch actions:
- **"Grow all"**: advance all selected fields to the ready-to-harvest stage
- **"Weed all"**: set weed level to 0
- **"Remove stones"**: set stone level to 0
- **"Lime all"**: set lime level to maximum
- **"Fertilize all"**: set fertilizer level to maximum
- **"Reset"**: return a field to the uncultivated state

## Land Ownership

### Viewing
- Display of the farmland parcel list with their current owner

### Editing
- Change the owner of a parcel (transfer it to the player's farm or release it)
- In advanced mode: assign a parcel to any farm (farm ID)
