# 10 - Cross-Cutting Features

## Disclaimer

### Behavior
- A modal dialog is displayed on application startup before any interaction is possible
- The dialog warns the user that:
  - The application modifies savegame files and may cause corruption
  - The user is solely responsible for any damage to their savegames
  - Creating backups before making changes is strongly recommended
- The user must check a checkbox ("I understand and accept the risks") before the accept button becomes active
- Once accepted, the disclaimer is not shown again (persisted in settings)
- The dialog cannot be dismissed by clicking outside or pressing Escape — the user must explicitly accept

### Persistence
- The acceptance state is saved in `settings.json` via Tauri plugin-store
- If the user has already accepted, the dialog is not shown on subsequent launches

## User Interface

### Navigation
- **Sidebar** with the main sections: Savegames, Finances, Vehicles, Used Market, Fields, World, Buildings, Missions, Collectibles
- Sections not yet available (Phase 2+) are visible but grayed out with a "Coming soon" label
- **Breadcrumb** showing the current position (e.g., Savegame1 > Vehicles > Fendt 942)

### Simple / Advanced Mode
- A global toggle in the navigation bar switches between simple and advanced mode
- In simple mode, only the most common fields and quick actions are visible
- In advanced mode, all editable properties and detailed options are displayed
- The chosen mode is remembered across sessions

### Change Indicators
- Modified but unsaved fields are visually marked (e.g., colored border, edit icon)
- A pending changes counter is always visible
- The user is warned when attempting to quit the application or switch savegames with unsaved changes

## Global Action Bar

- **"Save" button**: applies all pending changes (with an automatic backup beforehand)
- **"Discard changes" button**: resets all values to their original state (before editing)
- **"Reload" button**: re-reads files from disk (useful if files were modified outside the application)

## Multilingual Support

### Supported Languages
- French (FR)
- English (EN)

### Behavior
- Language is auto-detected based on system settings
- The user can change the language at any time via settings
- Language switching is instant, without reloading
- Crop names, vehicle types, and other game terms are translated in both languages

## Application Settings

### General
- **Language**: interface language selection
- **Default mode**: simple or advanced at launch
- **Default path**: custom path to the FarmingSimulator2025 folder

### Backups
- **Number of backups kept**: retention limit (default 10)
- **Backup location**: alongside the savegame (default) or custom folder

### Display
- **Theme**: light / dark / system
- **Currency units**: money display format

## Error Handling

### General Principle
The application must never fail silently. Every error is:
- Clearly displayed to the user in non-technical language
- Accompanied by a suggested action (e.g., "Try reloading the savegame" or "Make sure the game is not running")
- Logged to a file for debugging

### Main Error Cases
- **File locked**: the game is running and blocking file access
- **Corrupted file**: an XML file is malformed or contains unexpected data
- **Insufficient disk space**: unable to create the backup
- **Insufficient permissions**: the application does not have write access to the folder

## Accessibility

- Adjustable font size
- Keyboard navigation support (keyboard shortcuts for main actions)
- Sufficient contrast between text and background in both themes
