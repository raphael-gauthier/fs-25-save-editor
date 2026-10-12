// Farmland facts for buying and selling: the area and price the game computes
// (verified against real purchases), whether a field lies on the farmland,
// and the farm-built placeables standing on it.

use std::collections::HashMap;
use std::path::Path;

use crate::error::AppError;
use crate::models::field::FarmlandInfo;
use crate::models::placeable::Placeable;
use crate::parsers::placeable::parse_placeables;
use crate::services::density_map::{build_farmland_pixel_map, load_map_data, MapData};

/// Farmland info of the map `map_id`, with the placeables of the savegame.
pub fn load_farmlands_info(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
) -> Result<Vec<FarmlandInfo>, AppError> {
    let map = load_map_data(game_path, map_id)?;
    // Without placeables no farmland is flagged as built on
    let placeables = parse_placeables(savegame_path).unwrap_or_default();
    Ok(farmlands_info(&map, &placeables))
}

/// Area and price of every farmland the map defines. As Farmland.lua and
/// FarmlandManager.lua do, the area counts the farmland's pixels in the
/// farmland map, and the price is the fixed price or area x price per ha x
/// price scale. Buying and selling both move that price.
fn farmlands_info(map: &MapData, placeables: &[Placeable]) -> Vec<FarmlandInfo> {
    let farmlands = &map.farmlands;
    let map_size = map.fields.map_size as f64;
    let pixels = build_farmland_pixel_map(farmlands);
    let sqm_per_pixel = (map_size / farmlands.width as f64).powi(2);

    let farmland_at = |x: f64, z: f64| -> Option<u8> {
        let px = ((x / map_size + 0.5) * farmlands.width as f64).floor();
        let pz = ((z / map_size + 0.5) * farmlands.height as f64).floor();
        let inside = px >= 0.0 && pz >= 0.0 && px < farmlands.width as f64 && pz < farmlands.height as f64;
        inside.then(|| farmlands.get_pixel(px as u32, pz as u32))
    };
    let mut built: HashMap<u8, u32> = HashMap::new();
    if map_size > 0.0 {
        for placeable in placeables.iter().filter(|p| p.farm_id != 0 && !p.is_pre_placed) {
            if let Some(id) = placeable.position.as_ref().and_then(|p| farmland_at(p.x, p.z)) {
                *built.entry(id).or_default() += 1;
            }
        }
    }

    let defs = &map.farmland_defs;
    defs.farmlands
        .iter()
        .map(|def| {
            let count = u8::try_from(def.id)
                .ok()
                .and_then(|id| pixels.get(&id))
                .map_or(0, |p| p.len());
            let area_ha = count as f64 * sqm_per_pixel / 10_000.0;
            FarmlandInfo {
                id: def.id,
                area_ha,
                price: def
                    .fixed_price
                    .unwrap_or(area_ha * defs.price_per_ha * def.price_scale),
                npc_name: def.npc_name.clone(),
                default_farm_property: def.default_farm_property,
                show_on_farmlands_screen: def.show_on_farmlands_screen,
                built_placeables: u8::try_from(def.id)
                    .ok()
                    .and_then(|id| built.get(&id).copied())
                    .unwrap_or(0),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::Position;
    use crate::parsers::farmlands_config::{FarmlandDef, FarmlandsConfig};
    use crate::parsers::grle::GrleImage;

    fn def(id: u32, price_scale: f64, fixed_price: Option<f64>) -> FarmlandDef {
        FarmlandDef {
            id,
            price_scale,
            fixed_price,
            default_farm_property: false,
            show_on_farmlands_screen: true,
            npc_name: None,
        }
    }

    /// A 2048 m map with a 1024 farmland grid (2 m pixels, 4 m² each):
    /// farmland 1 covers 100 pixels at the north-west corner, farmland 2 a
    /// 10-pixel strip next to it.
    fn map() -> MapData {
        let mut pixels = vec![0u8; 1024 * 1024];
        for y in 0..10 {
            for x in 0..10 {
                pixels[y * 1024 + x] = 1;
            }
            pixels[y * 1024 + 10] = 2;
        }
        MapData::for_farmlands(
            GrleImage { width: 1024, height: 1024, pixels },
            2048.0,
            FarmlandsConfig {
                price_per_ha: 60000.0,
                farmlands: vec![def(1, 0.5, None), def(2, 1.0, Some(12000.0))],
            },
        )
    }

    fn placeable(farm_id: u8, x: f64, z: f64, is_pre_placed: bool) -> Placeable {
        Placeable {
            farm_id,
            position: Some(Position { x, y: 0.0, z }),
            is_pre_placed,
            ..Default::default()
        }
    }

    #[test]
    fn test_area_and_price_from_pixels_and_config() {
        let info = farmlands_info(&map(), &[]);
        assert_eq!(info.len(), 2);
        // 100 pixels x 4 m² = 0.04 ha; x 60000 x 0.5
        assert_eq!(info[0].id, 1);
        assert!((info[0].area_ha - 0.04).abs() < 1e-9);
        assert!((info[0].price - 1200.0).abs() < 1e-6);
        // A fixed price ignores the area
        assert_eq!(info[1].price, 12000.0);
    }

    /// Prices of the base and mod maps the user played, each matching a
    /// purchase or sale recorded in a real farms.xml (fieldPurchase /
    /// fieldSelling), on both EASY and NORMAL economic difficulty.
    #[test]
    #[ignore = "needs FS25_GAME pointing at the game install, and the Pallegney mod"]
    fn test_real_map_prices_match_recorded_purchases() {
        let game = std::path::PathBuf::from(std::env::var("FS25_GAME").expect("FS25_GAME"));
        let none = Path::new("");
        let price = |map_id: &str, id: u32| {
            let info = load_farmlands_info(none, &game, map_id).unwrap();
            info.iter().find(|f| f.id == id).unwrap().price
        };
        assert_eq!(price("MapUS", 23), 152_856.0);
        let pallegney = "FS25_Pallegney.FS25_Pallegney";
        assert_eq!(price(pallegney, 18), 156_660.0);
        assert_eq!(price(pallegney, 32), 98_436.0);
        assert_eq!(price(pallegney, 25), 62_676.0);
        assert_eq!(price(pallegney, 123), 25_728.0);
    }

    /// Copies the real savegame named by `FS25_SAVE` (never written to).
    fn real_save_copy(name: &str) -> std::path::PathBuf {
        let src = std::path::PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let root = std::env::temp_dir().join(format!("fs25_real_farmland_{}", name));
        let _ = std::fs::remove_dir_all(&root);
        let save = root.join("savegame1");
        std::fs::create_dir_all(&save).unwrap();
        for entry in std::fs::read_dir(&src).unwrap().flatten() {
            if entry.file_type().unwrap().is_file() {
                std::fs::copy(entry.path(), save.join(entry.file_name())).unwrap();
            }
        }
        save
    }

    /// What the editor saves when buying a farmland: its owner and the farm's
    /// money, through the same save_changes command.
    #[test]
    #[ignore = "needs FS25_SAVE / FS25_GAME pointing at a real savegame and install"]
    fn test_real_save_buying_a_farmland_saves_owner_and_money() {
        use crate::models::changes::{FarmlandChange, FinanceChanges, SavegameChanges};
        use crate::parsers::{farm::parse_farms, field::parse_farmlands};

        let save = real_save_copy("buy");
        let game = std::path::PathBuf::from(std::env::var("FS25_GAME").expect("FS25_GAME"));
        let map_id = std::env::var("FS25_MAP").unwrap_or_else(|_| "MapUS".to_string());
        let farmlands = parse_farmlands(&save).unwrap();
        let free = farmlands.iter().find(|f| f.farm_id == 0).expect("a free farmland").id;
        let price = load_farmlands_info(&save, &game, &map_id)
            .unwrap()
            .into_iter()
            .find(|f| f.id == free)
            .unwrap()
            .price;
        let money = parse_farms(&save).unwrap().into_iter().find(|f| f.farm_id == 1).unwrap().money;

        let changes = SavegameChanges {
            finance: Some(FinanceChanges { money: Some(money - price), loan: None }),
            farmlands: Some(vec![FarmlandChange { id: free, farm_id: 1 }]),
            ..Default::default()
        };
        let result = crate::commands::savegame::save_changes(save.display().to_string(), changes).unwrap();
        assert!(result.errors.is_empty(), "{:?}", result.errors.len());

        let after = parse_farmlands(&save).unwrap();
        assert_eq!(after.iter().find(|f| f.id == free).unwrap().farm_id, 1);
        assert_eq!(after.iter().filter(|f| f.farm_id != 0).count(), farmlands.iter().filter(|f| f.farm_id != 0).count() + 1);
        let money_after = parse_farms(&save).unwrap().into_iter().find(|f| f.farm_id == 1).unwrap().money;
        assert!((money_after - (money - price)).abs() < 0.01);
        println!("bought farmland {} for {} ({} -> {})", free, price, money, money_after);
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }

    #[test]
    fn test_counts_farm_built_placeables_on_each_farmland() {
        // World (-1020, -1020) is farmland pixel (2, 2)
        let placeables = [
            placeable(1, -1020.0, -1020.0, false),
            // Map buildings follow the farmland's owner on their own
            placeable(1, -1020.0, -1020.0, true),
            // Nobody's
            placeable(0, -1020.0, -1020.0, false),
            // Farmland 2's strip, world x -1003
            placeable(1, -1003.0, -1020.0, false),
        ];
        let info = farmlands_info(&map(), &placeables);
        assert_eq!(info[0].built_placeables, 1);
        assert_eq!(info[1].built_placeables, 1);
    }
}
