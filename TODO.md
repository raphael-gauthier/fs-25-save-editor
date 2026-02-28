# TODO

## Bug Fixes

### 1. Finance - Advanced mode stats all zero with no history
- [x] In the Finance view, when advanced mode is enabled, all statistics display as zero and no history is shown.

### 2. Vehicles - Multiple selection and filter issues
- [x] **Row selection:** Clicking a single row in the vehicles table selects all rows instead of just the clicked one.
- [x] **Type filter incomplete:** The "Types" filter dropdown does not list all available vehicle types.
- [x] **Condition always "None":** The vehicle condition/state is always displayed as "None" — needs investigation to determine if this is a data issue or a display bug.

### 3. Missions - No data displayed
- [x] The Missions view shows no data. Needs investigation to determine whether this is a parsing issue, a data mapping problem, or expected behavior for certain savegames.

## Features

### 4. Vehicle image previews
- [x] Add vehicle thumbnail/image previews in the **Vehicles** view and the **Used Market (Sales)** view.
- [x] Source images from the game files.

### 5. Add vehicle to used market
- [x] Allow adding a vehicle to the used market (sale point) from the editor.
- [x] Use game file references to populate vehicle data (brand, model, category, price, etc.).
