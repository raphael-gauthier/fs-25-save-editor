# 01 - Vision and Scope

## Product Vision

FS25 Save Editor is a cross-platform native desktop application that allows Farming Simulator 2025 players to view and edit their save files in a simple and safe way, without directly manipulating XML files.

## Target Audience

### Casual Players
- Want to quickly adjust their money, unlock a vehicle, or fix a mistake in their game
- Need a clear, guided interface without technical jargon
- Are unfamiliar with the internal structure of save files

### Advanced Players / Modders
- Want detailed access to all editable parameters
- Are comfortable with precise numerical values and fine-tuned settings
- May want to edit advanced data (economy, weather, field states)

## UX Approach

The application provides a **simple interface by default** exposing the most common features. An **advanced mode** (toggleable globally or per section) unlocks access to all editable parameters.

## Functional Scope

### Phase 1 - MVP (Priority)
- Savegame management (detection, selection, backup)
- Finance editing (money, loans)
- Editing owned vehicles and equipment
- Editing the used vehicle market
- Mandatory automatic backup system

### Phase 2 - Enrichment
- Field and crop editing
- World editing (time, weather, seasons)
- Building and structure editing
- Collectible management
- Mission editing

### Phase 3 - Extensions
- Multiplayer support (multiple farms/players per savegame)
- Market economy editing (commodity prices)
- Player customization (appearance, clothing)
- NPC parameter editing

## Out of Scope
- Editing binary files (.gdm, .grle, .gmss, .dat, .png, .cache)
- Creating new savegames from scratch
- Online features or cloud synchronization
- Editing mods themselves

## Target Platforms
- Windows 10/11
- macOS
- Linux

## Supported Languages
- French
- English
- The interface is multilingual with the ability to switch languages at any time
