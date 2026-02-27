# FS25 Save Editor - Functional Specifications

## Table of Contents

| # | Document | Phase | Description |
|---|----------|-------|-------------|
| 01 | [Vision and Scope](01-vision-and-scope.md) | - | Product vision, target audience, scope and phasing |
| 02 | [Savegame Management](02-savegame-management.md) | Phase 1 | Detection, selection, backup and validation of savegames |
| 03 | [Finance Editing](03-finance-editing.md) | Phase 1 | Money, loans, financial statistics |
| 04 | [Vehicle Editing](04-vehicle-editing.md) | Phase 1 | Vehicle fleet, fill levels, condition, bulk actions |
| 05 | [Used Market](05-used-market.md) | Phase 1 | Vehicles for sale, pricing, wear, availability |
| 06 | [Fields and Crops](06-fields-and-crops.md) | Phase 2 | Field status, crops, growth, treatments |
| 07 | [World and Weather](07-world-and-weather.md) | Phase 2 | Playtime, weather forecasts, environment |
| 08 | [Buildings and Structures](08-buildings-and-structures.md) | Phase 2 | Buildings, factories, ongoing construction |
| 09 | [Missions and Collectibles](09-missions-and-collectibles.md) | Phase 2 | Contracts, progression, collectible items |
| 10 | [Cross-Cutting Features](10-cross-cutting-features.md) | All | UI, multilingual support, settings, error handling |

## Decision Summary

| Decision | Choice |
|----------|--------|
| Target audience | Casual AND advanced players (simple mode + advanced mode) |
| Backups | Automatic and mandatory before every modification |
| Save detection | Automatic + manual selection available |
| Languages | French + English, switchable at any time |
| Multiplayer | Solo first, multiplayer support planned for Phase 3 |
| Platforms | Windows, macOS, Linux |

## Analyzed Save Files

The editor works with the following XML files:

| File | Content | Phase |
|------|---------|-------|
| `careerSavegame.xml` | General configuration, money, game settings | 1 |
| `farms.xml` | Farms, finances, statistics, permissions | 1 |
| `vehicles.xml` | Owned vehicles and their full state | 1 |
| `sales.xml` | Used vehicle market | 1 |
| `fields.xml` | Field and crop status | 2 |
| `farmland.xml` | Land ownership | 2 |
| `environment.xml` | Time, weather, seasons | 2 |
| `placeables.xml` | Buildings and structures | 2 |
| `missions.xml` | Missions and contracts | 2 |
| `collectibles.xml` | Collectible items | 2 |
| `economy.xml` | Market prices and supply/demand | 3 |
| `players.xml` | Player appearance and equipment | 3 |
| `npc.xml` | NPC interactions | 3 |
| `handTools.xml` | Hand tools | 3 |
| `r_contracts.xml` | Contract settings | 2 |
