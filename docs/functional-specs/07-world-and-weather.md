# 07 - World and Weather Editing (Phase 2)

## Overview

This section allows editing the game world state: the passage of time, weather conditions, and the environment.

## Game Time

### Simple Mode
- **Current day**: display and edit the current game day (e.g., Day 54)
- **Time of day**: display and edit the current in-game time (converted to a readable HH:MM format)
- **Current season**: season inferred from the current day (informational)

### Advanced Mode
- **Days per period**: number of real days per game period
- **Internal timers**: real playtime counters

## Weather

### Weather Forecast
Display of scheduled weather events:
- **Type**: Sun, Rain, Cloudy, Snow, Tornado
- **Associated season**
- **Start day and time**
- **Duration**

### Forecast Editing
- Change the weather type of an event
- Edit the duration of an event
- Delete a weather event
- Add a weather event

### Quick Actions
- **"Force clear weather"**: replace all forecasts with sunshine
- **"Remove tornadoes"**: remove only tornado events

## Environment

### Advanced Mode Only
- **Snow depth**: edit the snow layer thickness
- **Soil moisture**: edit the ground moisture level
- **Fog settings**: coverage, density, visibility
