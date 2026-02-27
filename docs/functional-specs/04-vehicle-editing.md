# 04 - Vehicle and Equipment Editing

## Overview

The Vehicles section allows viewing and editing all vehicles and equipment owned by the player. Vehicles include tractors, harvesters, trailers, attached implements, and any other motorized or towed machinery.

## Vehicle List

### Display
Each owned vehicle is displayed with:
- **Vehicle name** (derived from the definition file path, e.g., "Fendt 942 Vario")
- **Type** (tractor, harvester, trailer, implement, etc.)
- **Ownership status** (owned, leased)
- **Purchase price**
- **Age** (in in-game days)
- **Usage time** (in hours, formatted)
- **Owning farm**

### Filtering and Search
- Search bar by vehicle name
- Filters by vehicle type (tractor, harvester, trailer, implement, etc.)
- Filter by status (owned / leased)

## Simple Mode - Per-Vehicle Editing

### Editable Properties
- **Purchase price**: edit the recorded price of the vehicle
- **Age**: edit the vehicle's age in days (resets to new if set to 0)
- **Usage time**: edit the usage hours counter
- **Ownership status**: toggle between owned and leased

### Fill Levels
For each vehicle with tanks or hoppers:
- Display of each fill type (fuel, DEF/AdBlue, seeds, fertilizer, harvest, etc.)
- Readable name of the fill type
- Current level and maximum capacity
- Editing via slider or numeric field
- "Fill All" button to set all tanks to maximum
- "Empty All" button to reset all tanks to zero

## Advanced Mode - Per-Vehicle Editing

### Position and Rotation
- X, Y, Z coordinates of the vehicle in the world
- Rotation on all 3 axes
> Warning displayed: moving a vehicle to invalid coordinates may make it inaccessible in-game

### Configuration
- Display of the vehicle's current configuration (options chosen at purchase)
- Wheel configuration identifiers

### Attachments
- List of equipment currently attached to the vehicle
- Option to detach equipment (break the attachment link)

## Bulk Actions

### On Selection
The user can select multiple vehicles and apply batch actions:
- **Fill all tanks** of all selected vehicles
- **Reset to new** (age and hours to 0) for all selected vehicles
- **Change owner** of all selected vehicles (useful for future multiplayer support)

## Vehicle Deletion

- The user can delete a vehicle from the savegame
- A clear warning is displayed before deletion
- Deletion removes the vehicle from the `vehicles.xml` file

> Note: adding new vehicles is not supported in the MVP because it would require knowledge of all valid configurations and internal identifiers, which is complex and risky for savegame integrity.
