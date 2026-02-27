# 02 - Savegame Management

## Automatic Savegame Detection

### Default Behavior
On launch, the application automatically scans the default Farming Simulator 2025 path based on the operating system:
- **Windows:** `Documents\My Games\FarmingSimulator2025\`
- **macOS:** `~/Library/Application Support/FarmingSimulator2025/`
- **Linux:** `~/.local/share/FarmingSimulator2025/` (or equivalent Steam/Proton path)

The application detects all `savegame*` folders present and displays them as a list.

### Information Displayed Per Savegame
For each detected savegame, the application displays:
- **Savegame name** (e.g., "My Game")
- **Map played** (e.g., "Riverbend Springs")
- **Available money**
- **Playtime** (formatted in hours and minutes)
- **Last modified date** of the folder
- **Economic difficulty**

### Manual Selection
The user can also choose a savegame folder manually via a file picker. This supports:
- Non-standard installations
- Copied or moved savegames
- Custom paths (dedicated servers, etc.)

## Backup System

### Principle
Before any modification to a savegame, a full backup is created **automatically and mandatorily**. The user cannot disable this feature.

### How It Works
- The backup is a complete copy of the savegame folder
- It is stored in a dedicated subfolder (e.g., `savegame1_backups/`) at the same level as the savegame
- Each backup is timestamped (e.g., `backup_2025-01-15_14h30m22s`)
- A clear message confirms the backup creation before applying modifications

### Backup Management
- The user can view the list of existing backups for a savegame
- They can restore a previous backup (replacing the current savegame with a backup)
- They can manually delete old backups
- The application displays the disk space used by backups

### Retention Limit
- By default, the last 10 backups are kept
- This number is configurable in the application settings
- Backups beyond the limit are flagged but not automatically deleted (the user decides)

## File Validation

### On Open
When loading a savegame, the application checks:
- The presence of essential XML files (careerSavegame.xml, farms.xml, vehicles.xml, etc.)
- That the XML files are well-formed (parsable)
- Basic data consistency (e.g., the money in `careerSavegame.xml` matches the one in `farms.xml`)

### In Case of Issues
- If files are missing or corrupted, the application displays a clear warning
- Sections corresponding to missing or unreadable files are disabled
- The application never blocks entirely: functional sections remain accessible

## Saving Modifications

### Process
1. The user makes their modifications in the interface
2. They click "Apply" / "Save"
3. The automatic backup is created
4. The modifications are written to the XML files
5. A success confirmation is displayed

### Data Consistency
Some values are duplicated across multiple files (e.g., money appears in both `careerSavegame.xml` AND `farms.xml`). The application ensures that these values stay synchronized when saving.
