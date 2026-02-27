# 08 - Buildings and Structures Editing (Phase 2)

## Overview

This section allows viewing and editing buildings, factories, and structures placed on the map by the player.

## Building List

### Display
Each building is displayed with:
- **Name** (inferred from the definition path)
- **Type** (farm building, factory, silo, wind turbine, etc.)
- **Owner** (associated farm)
- **Pre-placed or not** (distinguishes map buildings from player constructions)
- **Age** (in game days)
- **Price**

## Simple Mode

### Editable Properties
- **Owner**: change the owning farm
- **Price**: edit the recorded price

### Buildings Under Construction
For buildings currently being constructed:
- Display of the current construction step
- Display of required and remaining materials for each step
- **"Complete construction" action**: sets all remaining materials to 0, which instantly finishes the construction

## Advanced Mode

### Selling Stations
For selling points (selling silos, etc.):
- Per-product-type statistics: quantity received, amount paid
- Recorded price curves

### Factories and Productions
For production buildings:
- Input raw material stock levels
- Output finished product stock levels

### Position
- Display of building position (coordinates)
> Modifying building positions is not recommended and is not exposed even in advanced mode, as buildings have complex placement constraints.
