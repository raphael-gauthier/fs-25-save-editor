use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::density::{
    DensityEditPayload, FieldDensityData, FruitCoverage, GroundCoverage, LevelDistribution,
};
use crate::parsers::density_map_config::{self, GROUND_TYPES};
use crate::parsers::gdm::{self, GdmImage};
use crate::parsers::grle::{self, GrleImage};

/// Main entry point: aggregate all density map data per farmland
pub fn aggregate_field_data(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
) -> Result<Vec<FieldDensityData>, AppError> {
    // 1. Resolve map data and read farmlands
    let (farmlands_grle, fruit_types) = load_map_data(game_path, map_id)?;

    // 2. Build farmland pixel map (farmland_id → list of pixel coords)
    let farmland_pixels = build_farmland_pixel_map(&farmlands_grle);

    if farmland_pixels.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Read savegame density maps
    let fruits_gdm = read_gdm_file(&savegame_path.join("densityMap_fruits.gdm"))?;
    let ground_gdm = read_gdm_file(&savegame_path.join("densityMap_ground.gdm")).ok();
    let weed_gdm = read_gdm_file(&savegame_path.join("densityMap_weed.gdm")).ok();
    let stones_gdm = read_gdm_file(&savegame_path.join("densityMap_stones.gdm")).ok();

    // 4. Read info layer GRLEs
    let lime_grle = read_grle_file(&savegame_path.join("infoLayer_limeLevel.grle")).ok();
    let plow_grle = read_grle_file(&savegame_path.join("infoLayer_plowLevel.grle")).ok();
    let spray_grle = read_grle_file(&savegame_path.join("infoLayer_sprayLevel.grle")).ok();
    let roller_grle = read_grle_file(&savegame_path.join("infoLayer_rollerLevel.grle")).ok();
    let stubble_grle =
        read_grle_file(&savegame_path.join("infoLayer_stubbleShredLevel.grle")).ok();

    // 5. Compute scale factor (farmlands may be lower resolution)
    let scale = fruits_gdm.width / farmlands_grle.width;

    // 6. Aggregate per farmland
    let mut results = Vec::new();

    for (&farmland_id, pixels) in &farmland_pixels {
        if farmland_id == 0 {
            continue; // Skip non-farmland pixels
        }

        let density = aggregate_single_farmland(
            farmland_id as u32,
            pixels,
            scale,
            &fruits_gdm,
            &ground_gdm,
            &weed_gdm,
            &stones_gdm,
            &lime_grle,
            &plow_grle,
            &spray_grle,
            &roller_grle,
            &stubble_grle,
            &fruit_types,
        );

        results.push(density);
    }

    results.sort_by_key(|d| d.farmland_id);
    Ok(results)
}

/// Load map-specific data: farmlands GRLE + fruit type mapping
fn load_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, Vec<String>), AppError> {
    if map_id.contains('.') {
        // Modded map: e.g. "FS25_Pallegney.FS25_Pallegney"
        load_mod_map_data(game_path, map_id)
    } else {
        // Built-in map: "MapUS", "MapEU", "MapAS"
        load_builtin_map_data(game_path, map_id)
    }
}

fn load_builtin_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, Vec<String>), AppError> {
    let map_dir = match map_id {
        "MapUS" => "mapUS",
        "MapEU" => "mapEU",
        "MapAS" => "mapAS",
        other => {
            return Err(AppError::DensityMapError {
                message: format!("Unknown built-in map: {}. DLC maps are not yet supported.", other),
            });
        }
    };

    let farmlands_path = game_path
        .join("data")
        .join("maps")
        .join(map_dir)
        .join("data")
        .join("infoLayer_farmlands.grle");

    let farmlands_grle = read_grle_file(&farmlands_path)?;

    // 1. Base shared fruit types
    let fruit_types_path = game_path
        .join("data")
        .join("maps")
        .join("maps_fruitTypes.xml");

    let mut fruit_types = density_map_config::parse_fruit_types_from_file(&fruit_types_path)?;

    // 2. Map-specific fruit types (e.g. MEADOW from mapUS.xml)
    let map_xml_path = game_path
        .join("data")
        .join("maps")
        .join(map_dir)
        .join(format!("{}.xml", map_dir));

    if let Ok(map_xml_data) = std::fs::read(&map_xml_path) {
        if let Ok(extra_types) = density_map_config::parse_map_xml_fruit_types(&map_xml_data) {
            fruit_types.extend(extra_types);
        }
    }

    // 3. DLC fruit types from game log (captures types not in any XML)
    append_dlc_fruit_types_from_log(&mut fruit_types);

    Ok((farmlands_grle, fruit_types))
}

fn load_mod_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, Vec<String>), AppError> {
    // Extract mod name from map_id like "FS25_Pallegney.FS25_Pallegney"
    let mod_name = map_id
        .split('.')
        .next()
        .ok_or_else(|| AppError::DensityMapError {
            message: format!("Invalid mod map ID: {}", map_id),
        })?;

    // Mods are in the user's FarmingSimulator2025/mods/ directory
    let mods_path = get_mods_dir()?;
    let zip_path = mods_path.join(format!("{}.zip", mod_name));

    if !zip_path.exists() {
        return Err(AppError::DensityMapError {
            message: format!(
                "Mod zip not found: {}. Expected at {}",
                mod_name,
                zip_path.display()
            ),
        });
    }

    // Read farmlands GRLE from zip
    let farmlands_data =
        read_file_from_zip(&zip_path, "maps/data/infoLayer_farmlands.grle")?;
    let farmlands_grle = grle::parse_grle(&farmlands_data)?;

    // Read fruit types from zip
    let mut fruit_types =
        density_map_config::parse_fruit_types_from_zip(&zip_path, "maps/config/fruitTypes.xml")
            .or_else(|_| {
                // Fallback: try base game fruit types
                let base_path = game_path
                    .join("data")
                    .join("maps")
                    .join("maps_fruitTypes.xml");
                density_map_config::parse_fruit_types_from_file(&base_path)
            })?;

    // DLC fruit types from game log
    append_dlc_fruit_types_from_log(&mut fruit_types);

    Ok((farmlands_grle, fruit_types))
}

/// Append DLC fruit types from the game log that aren't already in the list.
/// The game log records the exact fruit type registration order including DLC types.
/// Also appends known extra fruit types as a final fallback.
fn append_dlc_fruit_types_from_log(fruit_types: &mut Vec<String>) {
    let log_path = match dirs::document_dir() {
        Some(docs) => docs
            .join("My Games")
            .join("FarmingSimulator2025")
            .join("log.txt"),
        None => {
            append_known_extra_types(fruit_types);
            return;
        }
    };

    if let Ok(log_types) = density_map_config::parse_game_log_fruit_types(&log_path) {
        for log_type in &log_types {
            let normalized = log_type.to_uppercase();
            if !fruit_types.iter().any(|t| t.to_uppercase() == normalized) {
                fruit_types.push(normalized);
            }
        }
    }

    // Append known extra types that may not appear in game log
    append_known_extra_types(fruit_types);
}

/// Append known FS25 fruit types that aren't already resolved.
fn append_known_extra_types(fruit_types: &mut Vec<String>) {
    for &name in density_map_config::KNOWN_EXTRA_FRUIT_TYPES {
        if !fruit_types.iter().any(|t| t.eq_ignore_ascii_case(name)) {
            fruit_types.push(name.to_string());
        }
    }
}

/// Get the FarmingSimulator2025 mods directory
fn get_mods_dir() -> Result<PathBuf, AppError> {
    let docs = dirs::document_dir().ok_or_else(|| AppError::DensityMapError {
        message: "Could not find user documents directory".to_string(),
    })?;
    Ok(docs.join("My Games").join("FarmingSimulator2025").join("mods"))
}

/// Build map of farmland_id → Vec<(x, y)> coordinates at farmland resolution
fn build_farmland_pixel_map(farmlands: &GrleImage) -> HashMap<u8, Vec<(u16, u16)>> {
    let mut map: HashMap<u8, Vec<(u16, u16)>> = HashMap::new();

    for y in 0..farmlands.height {
        for x in 0..farmlands.width {
            let id = farmlands.get_pixel(x, y);
            if id > 0 {
                map.entry(id).or_default().push((x as u16, y as u16));
            }
        }
    }

    map
}

fn read_grle_file(path: &Path) -> Result<GrleImage, AppError> {
    let data = std::fs::read(path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read {}: {}", path.display(), e),
    })?;
    grle::parse_grle(&data)
}

fn read_gdm_file(path: &Path) -> Result<GdmImage, AppError> {
    let data = std::fs::read(path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read {}: {}", path.display(), e),
    })?;
    gdm::parse_gdm(&data)
}

fn read_file_from_zip(zip_path: &Path, inner_path: &str) -> Result<Vec<u8>, AppError> {
    let file = std::fs::File::open(zip_path).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to open zip {}: {}", zip_path.display(), e),
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read zip: {}", e),
    })?;
    let mut entry = archive.by_name(inner_path).map_err(|e| AppError::DensityMapError {
        message: format!("{} not found in zip: {}", inner_path, e),
    })?;
    let mut data = Vec::new();
    entry.read_to_end(&mut data).map_err(|e| AppError::DensityMapError {
        message: format!("Failed to read {} from zip: {}", inner_path, e),
    })?;
    Ok(data)
}

/// Aggregate density data for a single farmland
#[allow(clippy::too_many_arguments)]
fn aggregate_single_farmland(
    farmland_id: u32,
    farmland_pixels: &[(u16, u16)],
    scale: u32,
    fruits_gdm: &GdmImage,
    ground_gdm: &Option<GdmImage>,
    weed_gdm: &Option<GdmImage>,
    stones_gdm: &Option<GdmImage>,
    lime_grle: &Option<GrleImage>,
    plow_grle: &Option<GrleImage>,
    spray_grle: &Option<GrleImage>,
    roller_grle: &Option<GrleImage>,
    stubble_grle: &Option<GrleImage>,
    fruit_types: &[String],
) -> FieldDensityData {
    // Accumulators
    let mut fruit_counts: HashMap<u8, (u32, u32)> = HashMap::new(); // index → (count, growth_sum)
    let mut ground_counts: HashMap<u8, u32> = HashMap::new();
    let mut lime_acc = LevelAccumulator::new();
    let mut plow_acc = LevelAccumulator::new();
    let mut spray_acc = LevelAccumulator::new();
    let mut roller_acc = LevelAccumulator::new();
    let mut stubble_acc = LevelAccumulator::new();
    let mut weed_sum: u32 = 0;
    let mut weed_count: u32 = 0;
    let mut stone_count: u32 = 0;
    let mut total_hi_res_pixels: u32 = 0;
    let mut max_growth: u8 = 0;

    for &(fx, fy) in farmland_pixels {
        // Each farmland pixel maps to a scale×scale block in high-res layers
        for dy in 0..scale {
            for dx in 0..scale {
                let hx = (fx as u32) * scale + dx;
                let hy = (fy as u32) * scale + dy;

                if hx >= fruits_gdm.width || hy >= fruits_gdm.height {
                    continue;
                }

                total_hi_res_pixels += 1;

                // Fruits: bits 0-4 = type index, bits 5-9 = growth state
                let fruit_pixel = fruits_gdm.get_pixel(hx, hy);
                let fruit_idx = GdmImage::extract_bits(fruit_pixel, 0, 5) as u8;
                let growth = GdmImage::extract_bits(fruit_pixel, 5, 5) as u8;

                let entry = fruit_counts.entry(fruit_idx).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += growth as u32;
                if growth > max_growth {
                    max_growth = growth;
                }

                // Ground type: bits 0-3
                if let Some(ref gdm) = ground_gdm {
                    if hx < gdm.width && hy < gdm.height {
                        let ground_pixel = gdm.get_pixel(hx, hy);
                        let ground_type = GdmImage::extract_bits(ground_pixel, 0, 4) as u8;
                        *ground_counts.entry(ground_type).or_insert(0) += 1;
                    }
                }

                // Weeds
                if let Some(ref gdm) = weed_gdm {
                    if hx < gdm.width && hy < gdm.height {
                        let weed_val = gdm.get_pixel(hx, hy) as u8;
                        if weed_val > 0 {
                            weed_count += 1;
                        }
                        weed_sum += weed_val as u32;
                    }
                }

                // Stones
                if let Some(ref gdm) = stones_gdm {
                    if hx < gdm.width && hy < gdm.height {
                        let stone_val = gdm.get_pixel(hx, hy) as u8;
                        if stone_val > 0 {
                            stone_count += 1;
                        }
                    }
                }

                // Info layers (GRLE)
                if let Some(ref grle) = lime_grle {
                    if hx < grle.width && hy < grle.height {
                        lime_acc.add(grle.get_pixel(hx, hy));
                    }
                }
                if let Some(ref grle) = plow_grle {
                    if hx < grle.width && hy < grle.height {
                        plow_acc.add(grle.get_pixel(hx, hy));
                    }
                }
                if let Some(ref grle) = spray_grle {
                    if hx < grle.width && hy < grle.height {
                        spray_acc.add(grle.get_pixel(hx, hy));
                    }
                }
                if let Some(ref grle) = roller_grle {
                    if hx < grle.width && hy < grle.height {
                        roller_acc.add(grle.get_pixel(hx, hy));
                    }
                }
                if let Some(ref grle) = stubble_grle {
                    if hx < grle.width && hy < grle.height {
                        stubble_acc.add(grle.get_pixel(hx, hy));
                    }
                }
            }
        }
    }

    let total = total_hi_res_pixels.max(1) as f32;

    // Build fruit distribution (skip index 0 = no fruit)
    let mut fruit_distribution: Vec<FruitCoverage> = fruit_counts
        .iter()
        .filter(|(&idx, _)| idx > 0)
        .map(|(&idx, &(count, growth_sum))| {
            let name = fruit_types
                .get((idx as usize).wrapping_sub(1))
                .cloned()
                .unwrap_or_else(|| format!("UNKNOWN_{}", idx));
            FruitCoverage {
                fruit_type: name,
                percentage: (count as f32 / total) * 100.0,
                avg_growth: if count > 0 {
                    growth_sum as f32 / count as f32
                } else {
                    0.0
                },
            }
        })
        .collect();

    fruit_distribution.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    let dominant_fruit = fruit_distribution.first().map(|f| f.fruit_type.clone());
    let fruit_coverage = fruit_distribution
        .first()
        .map(|f| f.percentage)
        .unwrap_or(0.0);

    // Average growth across all non-empty pixels
    let total_growth: u32 = fruit_counts
        .iter()
        .filter(|(&idx, _)| idx > 0)
        .map(|(_, &(_, growth_sum))| growth_sum)
        .sum();
    let total_fruit_pixels: u32 = fruit_counts
        .iter()
        .filter(|(&idx, _)| idx > 0)
        .map(|(_, &(count, _))| count)
        .sum();
    let avg_growth_state = if total_fruit_pixels > 0 {
        total_growth as f32 / total_fruit_pixels as f32
    } else {
        0.0
    };

    // Build ground type distribution
    let mut ground_type_distribution: Vec<GroundCoverage> = ground_counts
        .iter()
        .map(|(&idx, &count)| {
            let name = GROUND_TYPES
                .get(idx as usize)
                .unwrap_or(&"UNKNOWN")
                .to_string();
            GroundCoverage {
                ground_type: name,
                percentage: (count as f32 / total) * 100.0,
            }
        })
        .collect();
    ground_type_distribution.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    FieldDensityData {
        farmland_id,
        pixel_count: total_hi_res_pixels,
        dominant_fruit,
        fruit_coverage,
        fruit_distribution,
        avg_growth_state,
        max_growth_state: max_growth,
        lime_status: lime_acc.to_distribution(3),
        plow_status: plow_acc.to_distribution(1),
        spray_status: spray_acc.to_distribution(2),
        roller_status: roller_acc.to_distribution(1),
        stubble_shred_status: stubble_acc.to_distribution(1),
        weed_coverage: (weed_count as f32 / total) * 100.0,
        avg_weed_level: if total_hi_res_pixels > 0 {
            weed_sum as f32 / total as f32
        } else {
            0.0
        },
        stone_coverage: (stone_count as f32 / total) * 100.0,
        ground_type_distribution,
    }
}

/// Helper to accumulate level statistics
struct LevelAccumulator {
    count: u32,
    sum: u32,
    at_zero: u32,
    max_seen: u8,
    at_max_counts: HashMap<u8, u32>,
}

impl LevelAccumulator {
    fn new() -> Self {
        Self {
            count: 0,
            sum: 0,
            at_zero: 0,
            max_seen: 0,
            at_max_counts: HashMap::new(),
        }
    }

    fn add(&mut self, value: u8) {
        self.count += 1;
        self.sum += value as u32;
        if value == 0 {
            self.at_zero += 1;
        }
        if value > self.max_seen {
            self.max_seen = value;
        }
        *self.at_max_counts.entry(value).or_insert(0) += 1;
    }

    fn to_distribution(&self, expected_max: u8) -> LevelDistribution {
        let total = self.count.max(1) as f32;
        let at_max = self
            .at_max_counts
            .get(&expected_max)
            .copied()
            .unwrap_or(0);

        LevelDistribution {
            avg_level: self.sum as f32 / total,
            max_level: self.max_seen,
            pct_at_zero: (self.at_zero as f32 / total) * 100.0,
            pct_at_max: (at_max as f32 / total) * 100.0,
        }
    }
}

/// Apply density map edits and write modified files back to the savegame directory.
/// Returns the list of files that were modified.
pub fn save_density_edits(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
    edits: &[DensityEditPayload],
) -> Result<Vec<String>, AppError> {
    if edits.is_empty() {
        return Ok(Vec::new());
    }

    // Load map data to get farmland pixels and fruit type mapping
    let (farmlands_grle, fruit_types) = load_map_data(game_path, map_id)?;
    let farmland_pixels = build_farmland_pixel_map(&farmlands_grle);

    // Determine which files need to be modified
    let needs_fruits = edits.iter().any(|e| e.set_fruit_name.is_some());
    let needs_lime = edits.iter().any(|e| e.set_lime_level.is_some());
    let needs_spray = edits.iter().any(|e| e.set_spray_level.is_some());
    let needs_plow = edits.iter().any(|e| e.set_plow_level.is_some());
    let needs_roller = edits.iter().any(|e| e.set_roller_level.is_some());
    let needs_stubble = edits.iter().any(|e| e.set_stubble_shred_level.is_some());
    let needs_weeds = edits.iter().any(|e| e.clear_weeds);
    let needs_stones = edits.iter().any(|e| e.clear_stones);

    let scale = if needs_fruits || needs_weeds || needs_stones {
        let fruits_path = savegame_path.join("densityMap_fruits.gdm");
        let fruits_data = std::fs::read(&fruits_path).map_err(|e| AppError::DensityMapError {
            message: format!("Failed to read fruits GDM: {}", e),
        })?;
        let fruits_gdm = gdm::parse_gdm(&fruits_data)?;
        fruits_gdm.width / farmlands_grle.width
    } else if needs_lime || needs_spray || needs_plow || needs_roller || needs_stubble {
        // Use any available info layer to determine scale
        let lime_path = savegame_path.join("infoLayer_limeLevel.grle");
        let lime_data = std::fs::read(&lime_path).map_err(|e| AppError::DensityMapError {
            message: format!("Failed to read lime GRLE: {}", e),
        })?;
        let lime_grle = grle::parse_grle(&lime_data)?;
        lime_grle.width / farmlands_grle.width
    } else {
        return Ok(Vec::new());
    };

    let mut modified_files = Vec::new();

    // Build a fruit name → index map (1-based)
    let fruit_name_to_idx: HashMap<String, u8> = fruit_types
        .iter()
        .enumerate()
        .map(|(i, name)| (name.to_uppercase(), (i + 1) as u8))
        .collect();

    // Modify fruits GDM
    if needs_fruits {
        let fruits_path = savegame_path.join("densityMap_fruits.gdm");
        let original_data = std::fs::read(&fruits_path)?;
        let mut fruits_gdm = gdm::parse_gdm(&original_data)?;

        for edit in edits {
            if let Some(ref fruit_name) = edit.set_fruit_name {
                let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                    Some(p) => p,
                    None => continue,
                };

                let fruit_idx = if fruit_name.is_empty() || fruit_name == "NONE" {
                    0u8
                } else {
                    *fruit_name_to_idx.get(&fruit_name.to_uppercase()).unwrap_or(&0)
                };
                let growth = edit.set_growth_state.unwrap_or(0).min(31);
                let combined = (fruit_idx as u16) | ((growth as u16) << 5);

                for &(fx, fy) in pixels {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let hx = (fx as u32) * scale + dx;
                            let hy = (fy as u32) * scale + dy;
                            if hx < fruits_gdm.width && hy < fruits_gdm.height {
                                fruits_gdm.set_pixel(hx, hy, combined);
                            }
                        }
                    }
                }
            }
        }

        let encoded = gdm::write_gdm(&fruits_gdm, &original_data)?;
        std::fs::write(&fruits_path, &encoded)?;
        modified_files.push("densityMap_fruits.gdm".to_string());
    }

    // Modify weed GDM
    if needs_weeds {
        let weed_path = savegame_path.join("densityMap_weed.gdm");
        if let Ok(original_data) = std::fs::read(&weed_path) {
            let mut weed_gdm = gdm::parse_gdm(&original_data)?;

            for edit in edits {
                if !edit.clear_weeds {
                    continue;
                }
                let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                    Some(p) => p,
                    None => continue,
                };
                for &(fx, fy) in pixels {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let hx = (fx as u32) * scale + dx;
                            let hy = (fy as u32) * scale + dy;
                            if hx < weed_gdm.width && hy < weed_gdm.height {
                                weed_gdm.set_pixel(hx, hy, 0);
                            }
                        }
                    }
                }
            }

            let encoded = gdm::write_gdm(&weed_gdm, &original_data)?;
            std::fs::write(&weed_path, &encoded)?;
            modified_files.push("densityMap_weed.gdm".to_string());
        }
    }

    // Modify stones GDM
    if needs_stones {
        let stones_path = savegame_path.join("densityMap_stones.gdm");
        if let Ok(original_data) = std::fs::read(&stones_path) {
            let mut stones_gdm = gdm::parse_gdm(&original_data)?;

            for edit in edits {
                if !edit.clear_stones {
                    continue;
                }
                let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                    Some(p) => p,
                    None => continue,
                };
                for &(fx, fy) in pixels {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let hx = (fx as u32) * scale + dx;
                            let hy = (fy as u32) * scale + dy;
                            if hx < stones_gdm.width && hy < stones_gdm.height {
                                stones_gdm.set_pixel(hx, hy, 0);
                            }
                        }
                    }
                }
            }

            let encoded = gdm::write_gdm(&stones_gdm, &original_data)?;
            std::fs::write(&stones_path, &encoded)?;
            modified_files.push("densityMap_stones.gdm".to_string());
        }
    }

    // Helper to modify a GRLE info layer
    let modify_grle = |filename: &str,
                       edits: &[DensityEditPayload],
                       get_value: &dyn Fn(&DensityEditPayload) -> Option<u8>|
     -> Result<Option<String>, AppError> {
        let path = savegame_path.join(filename);
        let original_data = std::fs::read(&path).map_err(|e| AppError::DensityMapError {
            message: format!("Failed to read {}: {}", filename, e),
        })?;
        let (mut image, header) = grle::parse_grle_with_header(&original_data)?;

        let mut modified = false;
        for edit in edits {
            let value = match get_value(edit) {
                Some(v) => v,
                None => continue,
            };
            let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                Some(p) => p,
                None => continue,
            };

            modified = true;
            for &(fx, fy) in pixels {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let hx = (fx as u32) * scale + dx;
                        let hy = (fy as u32) * scale + dy;
                        if hx < image.width && hy < image.height {
                            image.set_pixel(hx, hy, value);
                        }
                    }
                }
            }
        }

        if modified {
            let encoded = grle::write_grle(&image, &header);
            std::fs::write(&path, &encoded)?;
            Ok(Some(filename.to_string()))
        } else {
            Ok(None)
        }
    };

    // Modify GRLE info layers
    if needs_lime {
        if let Some(name) = modify_grle("infoLayer_limeLevel.grle", edits, &|e| e.set_lime_level)?
        {
            modified_files.push(name);
        }
    }
    if needs_spray {
        if let Some(name) =
            modify_grle("infoLayer_sprayLevel.grle", edits, &|e| e.set_spray_level)?
        {
            modified_files.push(name);
        }
    }
    if needs_plow {
        if let Some(name) = modify_grle("infoLayer_plowLevel.grle", edits, &|e| e.set_plow_level)?
        {
            modified_files.push(name);
        }
    }
    if needs_roller {
        if let Some(name) =
            modify_grle("infoLayer_rollerLevel.grle", edits, &|e| e.set_roller_level)?
        {
            modified_files.push(name);
        }
    }
    if needs_stubble {
        if let Some(name) = modify_grle("infoLayer_stubbleShredLevel.grle", edits, &|e| {
            e.set_stubble_shred_level
        })? {
            modified_files.push(name);
        }
    }

    Ok(modified_files)
}
