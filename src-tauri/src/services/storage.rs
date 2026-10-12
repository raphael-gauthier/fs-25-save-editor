// Capacities and accepted fill types of the savegame's storages, from the
// XML of each placeable (game data, a mod, or a map's pre-placed placeable).

use std::collections::HashMap;
use std::path::Path;

use crate::error::AppError;
use crate::models::placeable::{
    FillTypeCapacity, ObjectStorageCapacity, Placeable, SiloCapacity, SiloStorageKind, StorageCapacities,
};
use crate::parsers::placeable::parse_placeables;
use crate::parsers::storage_defs::{parse_storage_defs, StorageDef, StorageDefs};
use crate::services::density_map::{game_data_file, get_mods_dir, load_map_data, read_file_from_zip};

/// Capacities of the storages in the savegame's placeables.
pub fn load_storage_capacities(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
) -> Result<StorageCapacities, AppError> {
    let map = load_map_data(game_path, map_id)?;
    let placeables = parse_placeables(savegame_path)?;
    let mut cache: HashMap<String, Option<StorageDefs>> = HashMap::new();
    let defs_of = |p: &Placeable| -> Option<StorageDefs> {
        let file = if p.filename.is_empty() {
            map.preplaced_files.get(&p.unique_id)?.clone()
        } else {
            p.filename.clone()
        };
        cache
            .entry(file.clone())
            .or_insert_with(|| placeable_xml(game_path, &file).map(|xml| parse_storage_defs(&xml)))
            .clone()
    };
    Ok(storage_capacities(&placeables, defs_of, &map.fill_type_categories))
}

/// Reads a placeable XML: "$data/..." or "data/..." from the game install,
/// "$moddir$MOD/..." from the mod's zip or folder.
fn placeable_xml(game_path: &Path, filename: &str) -> Option<String> {
    let bytes = if let Some(path) = game_data_file(game_path, filename) {
        std::fs::read(path).ok()?
    } else {
        let (mod_name, inner) = filename.strip_prefix("$moddir$")?.split_once('/')?;
        let mods = get_mods_dir().ok()?;
        read_file_from_zip(&mods.join(format!("{}.zip", mod_name)), inner)
            .ok()
            .or_else(|| std::fs::read(mods.join(mod_name).join(inner)).ok())?
    };
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Matches each saved storage with its definition: a silo's `<storage index>`
/// is its 1-based position in the XML's `<storages>`.
fn storage_capacities(
    placeables: &[Placeable],
    mut defs_of: impl FnMut(&Placeable) -> Option<StorageDefs>,
    categories: &HashMap<String, Vec<String>>,
) -> StorageCapacities {
    let mut capacities = StorageCapacities::default();
    for placeable in placeables {
        let Some(defs) = defs_of(placeable) else { continue };
        for storage in &placeable.silo_storages {
            let def = match storage.kind {
                SiloStorageKind::Silo => storage
                    .index
                    .checked_sub(1)
                    .and_then(|i| defs.silo_storages.get(i as usize)),
                SiloStorageKind::Extension => defs.extension.as_ref(),
            };
            if let Some(def) = def {
                capacities.silos.push(SiloCapacity {
                    placeable_index: placeable.index,
                    kind: storage.kind,
                    index: storage.index,
                    capacity: def.capacity,
                    fill_type_capacities: def
                        .capacities
                        .iter()
                        .map(|(fill_type, capacity)| FillTypeCapacity { fill_type: fill_type.clone(), capacity: *capacity })
                        .collect(),
                    fill_types: accepted_fill_types(def, categories),
                    shared: def.supports_multiple_fill_types,
                });
            }
        }
        if let Some(capacity) = defs.object_storage_capacity {
            capacities.object_storages.push(ObjectStorageCapacity { placeable_index: placeable.index, capacity });
        }
    }
    capacities
}

/// Fill types of the storage's categories, then its own fill types.
fn accepted_fill_types(def: &StorageDef, categories: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut fill_types: Vec<String> = Vec::new();
    let from_categories = def.fill_type_categories.iter().filter_map(|c| categories.get(c)).flatten();
    for fill_type in from_categories.chain(&def.fill_types) {
        if !fill_types.contains(fill_type) {
            fill_types.push(fill_type.clone());
        }
    }
    fill_types
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::placeable::{ProductionStock, SiloStorage};

    fn def(capacity: f64, categories: &[&str], fill_types: &[&str]) -> StorageDef {
        StorageDef {
            capacity,
            capacities: Vec::new(),
            fill_type_categories: categories.iter().map(|s| s.to_string()).collect(),
            fill_types: fill_types.iter().map(|s| s.to_string()).collect(),
            supports_multiple_fill_types: true,
        }
    }

    fn placeable(index: usize, filename: &str, storages: Vec<SiloStorage>) -> Placeable {
        Placeable { index, filename: filename.into(), silo_storages: storages, ..Default::default() }
    }

    fn storage(kind: SiloStorageKind, index: u32) -> SiloStorage {
        SiloStorage { kind, index, farm_id: 1, stocks: vec![ProductionStock { fill_type: "WHEAT".into(), amount: 1.0 }] }
    }

    /// Capacities of the real savegame `FS25_SAVE` on its map `FS25_MAP`.
    #[test]
    #[ignore = "needs FS25_SAVE / FS25_GAME pointing at a real savegame and install"]
    fn test_real_save_storage_capacities() {
        let save = std::path::PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let game = std::path::PathBuf::from(std::env::var("FS25_GAME").expect("FS25_GAME"));
        let map = std::env::var("FS25_MAP").unwrap_or_else(|_| "MapUS".to_string());
        let placeables = parse_placeables(&save).unwrap();
        let capacities = load_storage_capacities(&save, &game, &map).unwrap();
        let storages: usize = placeables.iter().map(|p| p.silo_storages.len()).sum();
        for c in &capacities.silos {
            println!("{} {:?} {} cap {} shared {} types {}", placeables[c.placeable_index].display_name, c.kind, c.index, c.capacity, c.shared, c.fill_types.join(" "));
            assert!(c.capacity > 0.0 && !c.fill_types.is_empty());
        }
        for o in &capacities.object_storages {
            println!("{} objects {}", placeables[o.placeable_index].display_name, o.capacity);
        }
        // Every silo storage of the save finds its definition
        assert_eq!(capacities.silos.len(), storages);
    }

    #[test]
    fn test_matches_saved_storages_with_their_definitions() {
        let placeables = vec![
            placeable(0, "silo.xml", vec![storage(SiloStorageKind::Silo, 2)]),
            placeable(1, "extension.xml", vec![storage(SiloStorageKind::Extension, 0)]),
            placeable(2, "barn.xml", vec![]),
            placeable(3, "unknown.xml", vec![storage(SiloStorageKind::Silo, 1)]),
        ];
        let mut second = def(5000.0, &[], &["DIESEL"]);
        second.capacities = vec![("DIESEL".into(), 4000.0)];
        second.supports_multiple_fill_types = false;
        let defs_of = |p: &Placeable| match p.filename.as_str() {
            "silo.xml" => Some(StorageDefs {
                silo_storages: vec![def(200000.0, &["FARMSILO"], &[]), second.clone()],
                ..Default::default()
            }),
            "extension.xml" => Some(StorageDefs {
                extension: Some(def(250000.0, &["FARMSILO"], &["OAT", "WHEAT"])),
                ..Default::default()
            }),
            "barn.xml" => Some(StorageDefs { object_storage_capacity: Some(250), ..Default::default() }),
            _ => None,
        };
        let categories = HashMap::from([("FARMSILO".to_string(), vec!["WHEAT".to_string(), "BARLEY".to_string()])]);

        let capacities = storage_capacities(&placeables, defs_of, &categories);

        assert_eq!(
            capacities.silos,
            vec![
                SiloCapacity {
                    placeable_index: 0,
                    kind: SiloStorageKind::Silo,
                    index: 2,
                    capacity: 5000.0,
                    fill_type_capacities: vec![FillTypeCapacity { fill_type: "DIESEL".into(), capacity: 4000.0 }],
                    fill_types: vec!["DIESEL".into()],
                    shared: false,
                },
                SiloCapacity {
                    placeable_index: 1,
                    kind: SiloStorageKind::Extension,
                    index: 0,
                    capacity: 250000.0,
                    fill_type_capacities: vec![],
                    fill_types: vec!["WHEAT".into(), "BARLEY".into(), "OAT".into()],
                    shared: true,
                },
            ]
        );
        assert_eq!(capacities.object_storages, vec![ObjectStorageCapacity { placeable_index: 2, capacity: 250 }]);
    }
}
