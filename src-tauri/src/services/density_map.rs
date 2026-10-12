use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::density::{
    DensityEditPayload, FieldDensityData, FruitCoverage, GroundCoverage, LevelDistribution,
};
use crate::parsers::density_map_config::{self, GROUND_TYPES};
use crate::parsers::foliage::{self, FruitGrowth};
use crate::parsers::gdm::{self, GdmImage};
use crate::parsers::grle::{self, GrleImage};

/// Main entry point: aggregate all density map data per farmland
pub fn aggregate_field_data(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
) -> Result<Vec<FieldDensityData>, AppError> {
    // 1. Resolve map data and read farmlands
    let (farmlands_grle, layout) = load_map_data(game_path, map_id)?;

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
            &layout,
        );

        results.push(density);
    }

    results.sort_by_key(|d| d.farmland_id);
    Ok(results)
}

/// Bit layout of a densityMap_fruits.gdm pixel: the foliage type index in the
/// lowest `type_bits` (the first compression range), then the 4-bit growth
/// state, then the haulm bit of root crops.
#[derive(Debug, Clone, Copy)]
struct FruitBits {
    type_bits: u8,
}

const FRUIT_STATE_BITS: u8 = 4;

impl FruitBits {
    fn of(fruits: &GdmImage) -> Self {
        FruitBits {
            type_bits: fruits.range_channels.first().copied().unwrap_or(5),
        }
    }

    fn type_index(self, pixel: u16) -> usize {
        GdmImage::extract_bits(pixel, 0, self.type_bits) as usize
    }

    fn state(self, pixel: u16) -> u8 {
        GdmImage::extract_bits(pixel, self.type_bits, FRUIT_STATE_BITS) as u8
    }

    /// `pixel` with the given type and growth state and the haulm bit cleared.
    /// Any channel above the haulm bit is kept.
    fn encode(self, pixel: u16, type_index: usize, state: u8) -> u16 {
        let low_bits = (self.type_bits + FRUIT_STATE_BITS + 1) as u32;
        let kept = pixel
            .checked_shr(low_bits)
            .unwrap_or(0)
            .checked_shl(low_bits)
            .unwrap_or(0);
        kept | type_index as u16 | (((state & 0xF) as u16) << self.type_bits)
    }
}

/// How a map's fruit layer is laid out: its foliage type names, and the
/// growth states of every fruit type. Type index 0 is bare ground, so the
/// type at index `i` is `type_names[i - 1]`.
#[derive(Debug, Default)]
pub(crate) struct FruitLayout {
    type_names: Vec<String>,
    fruits: HashMap<String, FruitGrowth>,
}

impl FruitLayout {
    /// The fruit type at `type_index`, None for deco foliage or unknown indices.
    fn fruit_at(&self, type_index: usize) -> Option<&FruitGrowth> {
        self.type_name(type_index).and_then(|name| self.fruits.get(name))
    }

    fn type_name(&self, type_index: usize) -> Option<&String> {
        type_index.checked_sub(1).and_then(|i| self.type_names.get(i))
    }

    fn type_index_of(&self, name: &str) -> Option<usize> {
        self.type_names
            .iter()
            .position(|n| n.eq_ignore_ascii_case(name))
            .map(|i| i + 1)
    }

    /// Registers fruit types read from foliage XML contents. With `append`,
    /// types missing from the map's own layer are added after it, the way the
    /// game adds its extra fruit types.
    fn add_fruits(&mut self, foliage_xmls: impl IntoIterator<Item = Vec<u8>>, append: bool) {
        for xml in foliage_xmls {
            if let Some(growth) = foliage::parse_fruit_growth(&String::from_utf8_lossy(&xml)) {
                if append && !self.type_names.contains(&growth.name) {
                    self.type_names.push(growth.name.clone());
                }
                self.fruits.entry(growth.name.clone()).or_insert(growth);
            }
        }
    }
}

/// Load map-specific data: farmlands GRLE + fruit layer layout
fn load_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, FruitLayout), AppError> {
    if map_id.contains('.') {
        // Modded map: e.g. "FS25_Pallegney.FS25_Pallegney"
        load_mod_map_data(game_path, map_id)
    } else {
        // Built-in map: "MapUS", "MapEU", "MapAS"
        load_builtin_map_data(game_path, map_id)
    }
}

/// Resolves a foliage path from a fruit types list when it points into the
/// game install ("$data/..." or "data/...").
fn game_data_file(game_path: &Path, filename: &str) -> Option<PathBuf> {
    let relative = filename.strip_prefix('$').unwrap_or(filename);
    relative.starts_with("data/").then(|| game_path.join(relative))
}

fn read_game_files(game_path: &Path, filenames: &[String]) -> Vec<Vec<u8>> {
    filenames
        .iter()
        .filter_map(|f| game_data_file(game_path, f))
        .filter_map(|path| std::fs::read(path).ok())
        .collect()
}

fn load_builtin_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, FruitLayout), AppError> {
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
    let maps_dir = game_path.join("data").join("maps");
    let map_path = maps_dir.join(map_dir);

    let farmlands_grle = read_grle_file(&map_path.join("data").join("infoLayer_farmlands.grle"))?;

    let i3d = std::fs::read(map_path.join(format!("{}.i3d", map_dir))).map_err(|e| {
        AppError::DensityMapError {
            message: format!("Failed to read {}.i3d: {}", map_dir, e),
        }
    })?;
    let mut layout = FruitLayout {
        type_names: foliage::parse_fruit_layer_types(&String::from_utf8_lossy(&i3d)),
        fruits: HashMap::new(),
    };

    let mut fruit_files = std::fs::read(maps_dir.join("maps_fruitTypes.xml"))
        .map(|data| density_map_config::parse_fruit_type_filenames(&data))
        .unwrap_or_default();
    if let Ok(map_xml) = std::fs::read(map_path.join(format!("{}.xml", map_dir))) {
        fruit_files.extend(density_map_config::parse_fruit_type_filenames(&map_xml));
    }
    layout.add_fruits(read_game_files(game_path, &fruit_files), false);
    layout.add_fruits(
        read_game_files(game_path, &additional_fruit_type_files(game_path)),
        true,
    );

    Ok((farmlands_grle, layout))
}

fn load_mod_map_data(
    game_path: &Path,
    map_id: &str,
) -> Result<(GrleImage, FruitLayout), AppError> {
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

    let map_config_file = read_file_from_zip(&zip_path, "modDesc.xml")
        .ok()
        .and_then(|data| density_map_config::parse_mod_desc_map_config(&data))
        .ok_or_else(|| AppError::DensityMapError {
            message: format!("No map declared in {}'s modDesc.xml", mod_name),
        })?;
    let map_config =
        density_map_config::parse_map_config(&read_file_from_zip(&zip_path, &map_config_file)?);
    let i3d_file = map_config.i3d_filename.ok_or_else(|| AppError::DensityMapError {
        message: format!("No map i3d declared in {}", map_config_file),
    })?;
    let i3d = read_file_from_zip(&zip_path, &i3d_file)?;
    let mut layout = FruitLayout {
        type_names: foliage::parse_fruit_layer_types(&String::from_utf8_lossy(&i3d)),
        fruits: HashMap::new(),
    };

    let mut fruit_files = map_config.fruit_type_filenames;
    if let Some(file) = map_config.fruit_types_filename {
        if let Ok(data) = read_file_from_zip(&zip_path, &file) {
            fruit_files.extend(density_map_config::parse_fruit_type_filenames(&data));
        }
    }
    // Foliage files are either base-game ($data/...) or inside the mod zip
    let foliage_xmls: Vec<Vec<u8>> = fruit_files
        .iter()
        .filter_map(|file| match game_data_file(game_path, file) {
            Some(path) => std::fs::read(path).ok(),
            None => read_file_from_zip(&zip_path, file).ok(),
        })
        .collect();
    layout.add_fruits(foliage_xmls, false);
    layout.add_fruits(
        read_game_files(game_path, &additional_fruit_type_files(game_path)),
        true,
    );

    Ok((farmlands_grle, layout))
}

/// Foliage files of the fruit types the game appends after a map's own fruit
/// layer, as recorded in the game log. Without a log, the base game's onion is
/// assumed: it ships in data/foliage but in no map's fruit layer, and the game
/// adds it right after the map's own types.
fn additional_fruit_type_files(game_path: &Path) -> Vec<String> {
    let from_log = dirs::document_dir()
        .map(|docs| docs.join("My Games").join("FarmingSimulator2025").join("log.txt"))
        .and_then(|log| std::fs::read(log).ok())
        .map(|log| density_map_config::parse_additional_fruit_types(&String::from_utf8_lossy(&log)))
        .unwrap_or_default();
    if !from_log.is_empty() {
        return from_log;
    }
    let onion = "data/foliage/onion/onion.xml".to_string();
    if game_path.join(&onion).exists() {
        vec![onion]
    } else {
        Vec::new()
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

/// Calls `f` with every high-resolution pixel covered by `farmland_pixels`.
fn for_each_hi_res_pixel(farmland_pixels: &[(u16, u16)], scale: u32, mut f: impl FnMut(u32, u32)) {
    for &(fx, fy) in farmland_pixels {
        for dy in 0..scale {
            for dx in 0..scale {
                f(fx as u32 * scale + dx, fy as u32 * scale + dy);
            }
        }
    }
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
    layout: &FruitLayout,
) -> FieldDensityData {
    let bits = FruitBits::of(fruits_gdm);

    // Accumulators
    // Crops shown on the map only: deco foliage and meadow are left out
    let mut fruit_counts: HashMap<usize, (u32, u32)> = HashMap::new(); // type index → (count, growth_sum)
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

                let fruit_pixel = fruits_gdm.get_pixel(hx, hy);
                let growth = bits.state(fruit_pixel);
                let fruit_idx = bits.type_index(fruit_pixel);
                if growth > 0 && layout.fruit_at(fruit_idx).is_some_and(|f| f.shown_on_map) {
                    let entry = fruit_counts.entry(fruit_idx).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 += growth as u32;
                    if growth > max_growth {
                        max_growth = growth;
                    }
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

    let mut fruit_distribution: Vec<FruitCoverage> = fruit_counts
        .iter()
        .map(|(&idx, &(count, growth_sum))| {
            let name = layout.type_name(idx).cloned().unwrap_or_default();
            let harvest_ready_state = layout.fruit_at(idx).and_then(|f| f.harvest_ready_state);
            FruitCoverage {
                fruit_type: name,
                percentage: (count as f32 / total) * 100.0,
                avg_growth: if count > 0 {
                    growth_sum as f32 / count as f32
                } else {
                    0.0
                },
                harvest_ready_state,
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
    let total_growth: u32 = fruit_counts.values().map(|&(_, growth_sum)| growth_sum).sum();
    let total_fruit_pixels: u32 = fruit_counts.values().map(|&(count, _)| count).sum();
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
/// When `create_backup` is set, the savegame is backed up before any layer is
/// replaced. Returns the list of files that were modified.
pub fn save_density_edits(
    savegame_path: &Path,
    game_path: &Path,
    map_id: &str,
    edits: &[DensityEditPayload],
    create_backup: bool,
) -> Result<Vec<String>, AppError> {
    if edits.is_empty() {
        return Ok(Vec::new());
    }

    // Load map data to get farmland pixels and the fruit layer layout
    let (farmlands_grle, layout) = load_map_data(game_path, map_id)?;
    apply_density_edits(savegame_path, &farmlands_grle, &layout, edits, create_backup)
}

/// Applies `edits` to the savegame's density layers, using `farmlands_grle` to
/// locate each farmland's pixels and `layout` to read and write fruits.
fn apply_density_edits(
    savegame_path: &Path,
    farmlands_grle: &GrleImage,
    layout: &FruitLayout,
    edits: &[DensityEditPayload],
    create_backup: bool,
) -> Result<Vec<String>, AppError> {
    let farmland_pixels = build_farmland_pixel_map(farmlands_grle);

    // Determine which files need to be modified
    let needs_fruits = edits.iter().any(|e| e.set_fruit_name.is_some() || e.set_harvest_ready);
    let needs_ground = edits.iter().any(|e| e.set_ground_type.is_some() || e.set_harvest_ready);
    let needs_lime = edits.iter().any(|e| e.set_lime_level.is_some());
    let needs_spray = edits.iter().any(|e| e.set_spray_level.is_some());
    let needs_plow = edits.iter().any(|e| e.set_plow_level.is_some());
    let needs_roller = edits.iter().any(|e| e.set_roller_level.is_some());
    let needs_stubble = edits.iter().any(|e| e.set_stubble_shred_level.is_some());
    let needs_weeds = edits.iter().any(|e| e.clear_weeds);
    let needs_stones = edits.iter().any(|e| e.clear_stones);
    let any_crop_area_only = edits.iter().any(|e| e.crop_area_only);
    let any_field_area_only = edits.iter().any(|e| e.field_area_only);

    // Load fruits GDM for scale and crop_area_only filtering
    let fruits_gdm_for_filter = if needs_fruits || needs_ground || needs_weeds || needs_stones || any_crop_area_only {
        let fruits_path = savegame_path.join("densityMap_fruits.gdm");
        let fruits_data = std::fs::read(&fruits_path).map_err(|e| AppError::DensityMapError {
            message: format!("Failed to read fruits GDM: {}", e),
        })?;
        Some(gdm::parse_gdm(&fruits_data)?)
    } else {
        None
    };

    let scale = if let Some(ref fg) = fruits_gdm_for_filter {
        fg.width / farmlands_grle.width
    } else if needs_lime || needs_spray || needs_plow || needs_roller || needs_stubble {
        let lime_path = savegame_path.join("infoLayer_limeLevel.grle");
        let lime_data = std::fs::read(&lime_path).map_err(|e| AppError::DensityMapError {
            message: format!("Failed to read lime GRLE: {}", e),
        })?;
        let lime_grle = grle::parse_grle(&lime_data)?;
        lime_grle.width / farmlands_grle.width
    } else {
        return Ok(Vec::new());
    };

    // Load ground GDM for field_area_only filtering
    let ground_gdm_for_filter = if any_field_area_only {
        let ground_path = savegame_path.join("densityMap_ground.gdm");
        std::fs::read(&ground_path).ok().and_then(|data| gdm::parse_gdm(&data).ok())
    } else {
        None
    };

    // Every layer is encoded in memory first, so a failure on any layer
    // leaves the savegame untouched.
    let mut pending: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    let fruit_bits = fruits_gdm_for_filter.as_ref().map(FruitBits::of);

    // Helper: check if a pixel should be modified based on filter mode.
    // - crop_area_only: a growing fruit type (for harvest/mow — only affect crop pixels)
    // - field_area_only: ground_type 1-13 (for plow/cultivate — field ground, not grass/road)
    // - neither: modify all pixels in the farmland boundary
    let should_modify_pixel = |hx: u32, hy: u32, edit: &DensityEditPayload| -> bool {
        if edit.crop_area_only {
            if let Some(ref fg) = fruits_gdm_for_filter {
                if hx < fg.width && hy < fg.height {
                    let bits = FruitBits::of(fg);
                    let fruit_pixel = fg.get_pixel(hx, hy);
                    return bits.state(fruit_pixel) > 0
                        && layout.fruit_at(bits.type_index(fruit_pixel)).is_some();
                }
            }
            return false;
        }
        if edit.field_area_only {
            if let Some(ref gg) = ground_gdm_for_filter {
                if hx < gg.width && hy < gg.height {
                    let ground_pixel = gg.get_pixel(hx, hy);
                    let ground_type = GdmImage::extract_bits(ground_pixel, 0, 4);
                    // Field ground types: 1 (STUBBLE_TILLAGE) through 13 (HARVEST_READY_OTHER)
                    // Excludes: 0 (NONE), 14 (GRASS), 15 (GRASS_CUT)
                    return ground_type >= 1 && ground_type <= 13;
                }
            }
            return false;
        }
        true
    };

    // Ground types fruits show once harvest ready, set alongside them
    let mut harvest_ready_ground: Vec<(u32, u32, u16)> = Vec::new();

    // Modify fruits GDM
    if needs_fruits {
        let fruits_path = savegame_path.join("densityMap_fruits.gdm");
        let original_data = std::fs::read(&fruits_path)?;
        let mut fruits_gdm = gdm::parse_gdm(&original_data)?;
        let bits = fruit_bits.unwrap_or_else(|| FruitBits::of(&fruits_gdm));

        for edit in edits {
            let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                Some(p) => p,
                None => continue,
            };

            if let Some(ref fruit_name) = edit.set_fruit_name {
                let (type_index, growth) = if fruit_name.is_empty() || fruit_name == "NONE" {
                    (0, 0)
                } else {
                    let index = layout.type_index_of(fruit_name).ok_or_else(|| {
                        AppError::DensityMapError {
                            message: format!("Fruit type {} is not part of this map", fruit_name),
                        }
                    })?;
                    (index, edit.set_growth_state.unwrap_or(0))
                };

                for_each_hi_res_pixel(pixels, scale, |hx, hy| {
                    if hx < fruits_gdm.width && hy < fruits_gdm.height
                        && should_modify_pixel(hx, hy, edit)
                    {
                        let current = fruits_gdm.get_pixel(hx, hy);
                        fruits_gdm.set_pixel(hx, hy, bits.encode(current, type_index, growth));
                    }
                });
            }

            if edit.set_harvest_ready {
                // Every growing fruit pixel jumps to its own fruit's harvest-ready state
                for_each_hi_res_pixel(pixels, scale, |hx, hy| {
                    if hx >= fruits_gdm.width || hy >= fruits_gdm.height {
                        return;
                    }
                    let current = fruits_gdm.get_pixel(hx, hy);
                    let type_index = bits.type_index(current);
                    if bits.state(current) == 0 {
                        return;
                    }
                    let Some(fruit) = layout.fruit_at(type_index) else { return };
                    let Some(ready) = fruit.harvest_ready_state else { return };
                    fruits_gdm.set_pixel(hx, hy, bits.encode(current, type_index, ready));
                    if let Some(ground) = fruit
                        .harvest_ready_ground
                        .as_deref()
                        .and_then(|g| GROUND_TYPES.iter().position(|&t| t == g))
                    {
                        harvest_ready_ground.push((hx, hy, ground as u16));
                    }
                });
            }
        }

        pending.push((fruits_path, gdm::write_gdm(&fruits_gdm, &original_data)?));
    }

    // Modify ground GDM
    if needs_ground {
        let ground_path = savegame_path.join("densityMap_ground.gdm");
        if let Ok(original_data) = std::fs::read(&ground_path) {
            let mut ground_gdm = gdm::parse_gdm(&original_data)?;

            for edit in edits {
                let ground_type = match edit.set_ground_type {
                    Some(gt) => gt.min(15),
                    None => continue,
                };
                let pixels = match farmland_pixels.get(&(edit.farmland_id as u8)) {
                    Some(p) => p,
                    None => continue,
                };
                for &(fx, fy) in pixels {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let hx = (fx as u32) * scale + dx;
                            let hy = (fy as u32) * scale + dy;
                            if hx < ground_gdm.width && hy < ground_gdm.height
                                && should_modify_pixel(hx, hy, edit)
                            {
                                let current = ground_gdm.get_pixel(hx, hy);
                                // Replace only bits 0-3 (ground type), preserve upper bits
                                let new_val = (current & !0xF) | (ground_type as u16);
                                ground_gdm.set_pixel(hx, hy, new_val);
                            }
                        }
                    }
                }
            }
            for &(hx, hy, ground_type) in &harvest_ready_ground {
                let current = ground_gdm.get_pixel(hx, hy);
                ground_gdm.set_pixel(hx, hy, (current & !0xF) | ground_type);
            }

            pending.push((ground_path, gdm::write_gdm(&ground_gdm, &original_data)?));
        }
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
                            if hx < weed_gdm.width && hy < weed_gdm.height
                                && should_modify_pixel(hx, hy, edit)
                            {
                                weed_gdm.set_pixel(hx, hy, 0);
                            }
                        }
                    }
                }
            }

            pending.push((weed_path, gdm::write_gdm(&weed_gdm, &original_data)?));
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
                            if hx < stones_gdm.width && hy < stones_gdm.height
                                && should_modify_pixel(hx, hy, edit)
                            {
                                stones_gdm.set_pixel(hx, hy, 0);
                            }
                        }
                    }
                }
            }

            pending.push((stones_path, gdm::write_gdm(&stones_gdm, &original_data)?));
        }
    }

    // Helper to modify a GRLE info layer
    let modify_grle = |filename: &str,
                       edits: &[DensityEditPayload],
                       get_value: &dyn Fn(&DensityEditPayload) -> Option<u8>|
     -> Result<Option<(PathBuf, Vec<u8>)>, AppError> {
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
                        if hx < image.width && hy < image.height
                            && should_modify_pixel(hx, hy, edit)
                        {
                            image.set_pixel(hx, hy, value);
                        }
                    }
                }
            }
        }

        if modified {
            Ok(Some((path, grle::write_grle(&image, &header))))
        } else {
            Ok(None)
        }
    };

    // Modify GRLE info layers
    if needs_lime {
        if let Some(layer) = modify_grle("infoLayer_limeLevel.grle", edits, &|e| e.set_lime_level)?
        {
            pending.push(layer);
        }
    }
    if needs_spray {
        if let Some(layer) =
            modify_grle("infoLayer_sprayLevel.grle", edits, &|e| e.set_spray_level)?
        {
            pending.push(layer);
        }
    }
    if needs_plow {
        if let Some(layer) = modify_grle("infoLayer_plowLevel.grle", edits, &|e| e.set_plow_level)?
        {
            pending.push(layer);
        }
    }
    if needs_roller {
        if let Some(layer) =
            modify_grle("infoLayer_rollerLevel.grle", edits, &|e| e.set_roller_level)?
        {
            pending.push(layer);
        }
    }
    if needs_stubble {
        if let Some(layer) = modify_grle("infoLayer_stubbleShredLevel.grle", edits, &|e| {
            e.set_stubble_shred_level
        })? {
            pending.push(layer);
        }
    }

    if pending.is_empty() {
        return Ok(Vec::new());
    }
    if create_backup {
        crate::backup::manager::create_backup(savegame_path)?;
    }
    replace_files(&pending)
}

/// Replaces every file in `pending`: all contents go to temp files first and
/// only once each one is written are they renamed over the originals.
/// Returns the file names that were replaced.
fn replace_files(pending: &[(PathBuf, Vec<u8>)]) -> Result<Vec<String>, AppError> {
    let tmp_path = |path: &Path| {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(".tmp");
        path.with_file_name(name)
    };

    for (i, (path, bytes)) in pending.iter().enumerate() {
        if let Err(e) = std::fs::write(tmp_path(path), bytes) {
            for (written, _) in &pending[..=i] {
                let _ = std::fs::remove_file(tmp_path(written));
            }
            return Err(e.into());
        }
    }

    let mut replaced = Vec::with_capacity(pending.len());
    for (path, _) in pending {
        std::fs::rename(tmp_path(path), path)?;
        replaced.push(path.file_name().unwrap_or_default().to_string_lossy().into_owned());
    }
    Ok(replaced)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DENSITY_FILES: [&str; 9] = [
        "densityMap_fruits.gdm",
        "densityMap_ground.gdm",
        "densityMap_weed.gdm",
        "densityMap_stones.gdm",
        "infoLayer_limeLevel.grle",
        "infoLayer_plowLevel.grle",
        "infoLayer_sprayLevel.grle",
        "infoLayer_rollerLevel.grle",
        "infoLayer_stubbleShredLevel.grle",
    ];

    /// Copies the reference save's density layers into a fresh `<name>/savegame1`.
    fn reference_density_save(name: &str) -> PathBuf {
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("docs")
            .join("savegames");
        let root = std::env::temp_dir().join(format!("fs25_test_density_{}", name));
        let _ = std::fs::remove_dir_all(&root);
        let save = root.join("savegame1");
        std::fs::create_dir_all(&save).unwrap();
        for file in DENSITY_FILES {
            std::fs::copy(src.join(file), save.join(file)).unwrap();
        }
        save
    }

    /// A 1024x1024 farmland map (a quarter of the 4096 layers) where a
    /// 16x16 block is farmland 1 and everything else is unowned.
    fn farmland_map() -> GrleImage {
        let mut pixels = vec![0u8; 1024 * 1024];
        for y in 500..516 {
            for x in 500..516 {
                pixels[y * 1024 + x] = 1;
            }
        }
        GrleImage { width: 1024, height: 1024, pixels }
    }

    /// Riverbend Springs' fruit layer (data/maps/mapUS/mapUS.i3d) followed by
    /// the onion the game appends: the type order of the reference save.
    const MAP_US_TYPES: [&str; 31] = [
        "DECOFOLIAGE", "DECOBUSHUS", "MEADOW", "FORESTPLANTS", "WATERPLANTS",
        "GRASS", "WHEAT", "CANOLA", "BARLEY", "MAIZE", "POTATO", "SUGARBEET", "SUNFLOWER",
        "SOYBEAN", "OILSEEDRADISH", "POPLAR", "OAT", "SUGARCANE", "COTTON", "SORGHUM",
        "BEETROOT", "CARROT", "PARSNIP", "GRAPE", "OLIVE", "RICELONGGRAIN", "RICE",
        "GREENBEAN", "PEA", "SPINACH", "ONION",
    ];
    const BARLEY: usize = 9;

    fn growth(name: &str, shown_on_map: bool, ready: u8, ground: Option<&str>) -> FruitGrowth {
        FruitGrowth {
            name: name.to_string(),
            shown_on_map,
            harvest_ready_state: Some(ready),
            harvest_ready_ground: ground.map(str::to_string),
        }
    }

    /// Harvest-ready states from data/foliage/<fruit>/<fruit>.xml
    fn map_us_layout() -> FruitLayout {
        let fruits = [
            growth("MEADOW", false, 4, None),
            growth("WHEAT", true, 8, Some("HARVEST_READY")),
            growth("BARLEY", true, 8, Some("HARVEST_READY")),
            growth("CARROT", true, 5, Some("HARVEST_READY_OTHER")),
        ];
        FruitLayout {
            type_names: MAP_US_TYPES.iter().map(|s| s.to_string()).collect(),
            fruits: fruits.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    fn read_gdm(save: &Path, file: &str) -> GdmImage {
        gdm::parse_gdm(&std::fs::read(save.join(file)).unwrap()).unwrap()
    }

    /// The 16 values of `layer` under farmland-resolution pixel (fx, fy)
    /// (1024 grid over 4096 layers), row by row.
    fn block(layer: &GdmImage, fx: u16, fy: u16) -> Vec<u16> {
        (0..16u32)
            .map(|i| layer.get_pixel(fx as u32 * 4 + i % 4, fy as u32 * 4 + i / 4))
            .collect()
    }

    /// First farmland-resolution pixel whose block of `layer` satisfies `pred`.
    fn find_block(layer: &GdmImage, pred: impl Fn(&[u16]) -> bool) -> (u16, u16) {
        for fy in 0..1024u16 {
            for fx in 0..1024u16 {
                if pred(&block(layer, fx, fy)) {
                    return (fx, fy);
                }
            }
        }
        panic!("no matching block in the reference save");
    }

    fn farmland_at(fx: u16, fy: u16) -> GrleImage {
        let mut pixels = vec![0u8; 1024 * 1024];
        pixels[fy as usize * 1024 + fx as usize] = 1;
        GrleImage { width: 1024, height: 1024, pixels }
    }

    #[test]
    fn test_fruit_bits_decode_reference_save_values() {
        // Raw values found in the reference save on fields fields.xml reports as
        // harvested barley (type 9, state 10) and growing onion (type 31, state 6)
        let bits = FruitBits { type_bits: 5 };
        assert_eq!((bits.type_index(329), bits.state(329)), (BARLEY, 10));
        assert_eq!((bits.type_index(223), bits.state(223)), (31, 6));
    }

    #[test]
    fn test_fruit_bits_encode_clears_haulm_and_keeps_upper_channels() {
        let bits = FruitBits { type_bits: 7 };
        let haulm = 1u16 << 11;
        let upper = 1u16 << 13;
        let pixel = bits.encode(upper | haulm | 5 | (3 << 7), 44, 9);
        assert_eq!(bits.type_index(pixel), 44);
        assert_eq!(bits.state(pixel), 9);
        assert_eq!(pixel & haulm, 0);
        assert_eq!(pixel & upper, upper);
    }

    #[test]
    fn test_harvest_ready_moves_harvested_barley_back_to_its_ready_state() {
        let save = reference_density_save("harvest_ready");
        let fruits = read_gdm(&save, "densityMap_fruits.gdm");
        let bits = FruitBits::of(&fruits);
        let (fx, fy) = find_block(&fruits, |b| {
            b.iter().all(|&p| bits.type_index(p) == BARLEY && bits.state(p) == 10)
        });

        let edit = DensityEditPayload {
            farmland_id: 1,
            set_harvest_ready: true,
            ..Default::default()
        };
        apply_density_edits(&save, &farmland_at(fx, fy), &map_us_layout(), &[edit], false).unwrap();

        let fruits = read_gdm(&save, "densityMap_fruits.gdm");
        let ground = read_gdm(&save, "densityMap_ground.gdm");
        let harvest_ready = GROUND_TYPES.iter().position(|&g| g == "HARVEST_READY").unwrap() as u16;
        for dy in 0..4 {
            for dx in 0..4 {
                let (hx, hy) = (fx as u32 * 4 + dx, fy as u32 * 4 + dy);
                let pixel = fruits.get_pixel(hx, hy);
                assert_eq!((bits.type_index(pixel), bits.state(pixel)), (BARLEY, 8));
                assert_eq!(ground.get_pixel(hx, hy) & 0xF, harvest_ready);
            }
        }
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }

    #[test]
    fn test_set_fruit_rejects_a_fruit_type_the_map_lacks() {
        let save = reference_density_save("unknown_fruit");
        let fruits_before = std::fs::read(save.join("densityMap_fruits.gdm")).unwrap();
        let edit = DensityEditPayload {
            farmland_id: 1,
            set_fruit_name: Some("MUSTARD".to_string()),
            set_growth_state: Some(3),
            ..Default::default()
        };

        let result = apply_density_edits(&save, &farmland_map(), &map_us_layout(), &[edit], false);

        assert!(result.is_err());
        assert!(std::fs::read(save.join("densityMap_fruits.gdm")).unwrap() == fruits_before);
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }

    fn fruit_and_lime_edit() -> DensityEditPayload {
        DensityEditPayload {
            farmland_id: 1,
            set_fruit_name: Some("WHEAT".to_string()),
            set_growth_state: Some(8),
            set_lime_level: Some(3),
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_writes_nothing_when_a_later_layer_fails() {
        let save = reference_density_save("partial_failure");
        let fruits_before = std::fs::read(save.join("densityMap_fruits.gdm")).unwrap();
        std::fs::remove_file(save.join("infoLayer_limeLevel.grle")).unwrap();

        let result = apply_density_edits(
            &save,
            &farmland_map(),
            &map_us_layout(),
            &[fruit_and_lime_edit()],
            false,
        );

        assert!(result.is_err());
        let fruits_after = std::fs::read(save.join("densityMap_fruits.gdm")).unwrap();
        assert!(fruits_after == fruits_before, "fruits layer was written before the lime layer failed");
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }

    #[test]
    fn test_apply_backs_up_original_layers_before_writing() {
        let save = reference_density_save("backup");
        let fruits_before = std::fs::read(save.join("densityMap_fruits.gdm")).unwrap();

        let modified = apply_density_edits(
            &save,
            &farmland_map(),
            &map_us_layout(),
            &[fruit_and_lime_edit()],
            true,
        )
        .unwrap();
        assert!(modified.contains(&"densityMap_fruits.gdm".to_string()));
        assert!(std::fs::read(save.join("densityMap_fruits.gdm")).unwrap() != fruits_before);

        let backups = crate::backup::manager::list_backups(&save).unwrap();
        assert_eq!(backups.len(), 1);
        let backed_up = std::fs::read(PathBuf::from(&backups[0].path).join("densityMap_fruits.gdm")).unwrap();
        assert!(backed_up == fruits_before, "backup does not hold the original fruits layer");
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }

    #[test]
    fn test_apply_skips_backup_when_not_requested() {
        let save = reference_density_save("no_backup");

        apply_density_edits(
            &save,
            &farmland_map(),
            &map_us_layout(),
            &[fruit_and_lime_edit()],
            false,
        )
        .unwrap();

        assert!(crate::backup::manager::list_backups(&save).unwrap().is_empty());
        let leftovers: Vec<_> = std::fs::read_dir(&save)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temporary files left behind");
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }
    /// Copies the density layers of the real savegame named by `FS25_SAVE`
    /// (never written to) and returns it with the game install `FS25_GAME`
    /// and map id `FS25_MAP`. Run with `cargo test -- --ignored`.
    fn real_save_copy(name: &str) -> (PathBuf, PathBuf, String) {
        let src = PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let game = PathBuf::from(std::env::var("FS25_GAME").expect("FS25_GAME"));
        let map = std::env::var("FS25_MAP").unwrap_or_else(|_| "MapUS".to_string());
        let root = std::env::temp_dir().join(format!("fs25_real_density_{}", name));
        let _ = std::fs::remove_dir_all(&root);
        let save = root.join("savegame1");
        std::fs::create_dir_all(&save).unwrap();
        for file in DENSITY_FILES {
            std::fs::copy(src.join(file), save.join(file)).unwrap();
        }
        (save, game, map)
    }

    #[test]
    #[ignore = "needs FS25_SAVE / FS25_GAME pointing at a real savegame and install"]
    fn test_real_save_lime_edit_only_touches_its_farmland() {
        let (save, game, map) = real_save_copy("lime");
        let before = aggregate_field_data(&save, &game, &map).unwrap();
        let target = before.iter().find(|d| d.lime_status.avg_level < 3.0).expect("a farmland below max lime");

        let edit = DensityEditPayload {
            farmland_id: target.farmland_id,
            set_lime_level: Some(3),
            ..Default::default()
        };
        let modified = save_density_edits(&save, &game, &map, &[edit], true).unwrap();
        assert_eq!(modified, vec!["infoLayer_limeLevel.grle".to_string()]);
        assert_eq!(crate::backup::manager::list_backups(&save).unwrap().len(), 1);

        let after = aggregate_field_data(&save, &game, &map).unwrap();
        for (b, a) in before.iter().zip(&after) {
            if a.farmland_id == target.farmland_id {
                assert_eq!(a.lime_status.avg_level, 3.0);
            } else {
                assert_eq!(a.lime_status.avg_level, b.lime_status.avg_level, "farmland {} changed", a.farmland_id);
            }
        }
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }
    #[test]
    #[ignore = "needs FS25_SAVE / FS25_GAME pointing at a real savegame and install"]
    fn test_real_save_crops_match_fields_xml() {
        let save = PathBuf::from(std::env::var("FS25_SAVE").expect("FS25_SAVE"));
        let game = PathBuf::from(std::env::var("FS25_GAME").expect("FS25_GAME"));
        let map = std::env::var("FS25_MAP").unwrap_or_else(|_| "MapUS".to_string());
        let density = aggregate_field_data(&save, &game, &map).unwrap();
        let fields = crate::parsers::field::parse_fields(&save).unwrap();

        // fields.xml is the game's own summary of each field's crop
        let mut matched = 0;
        let mut compared = 0;
        for field in fields.iter().filter(|f| f.fruit_type != "UNKNOWN") {
            let Some(d) = density.iter().find(|d| d.farmland_id == field.id) else { continue };
            let crops: Vec<&str> = d.fruit_distribution.iter().map(|f| f.fruit_type.as_str()).collect();
            compared += 1;
            if crops.contains(&field.fruit_type.as_str()) {
                matched += 1;
            } else {
                println!("field {}: fields.xml {} / density {:?}", field.id, field.fruit_type, crops);
            }
        }
        println!("{} of {} fields found their fields.xml crop", matched, compared);
        // Fields and farmlands do not always share an id, so a few miss
        assert!(compared > 0 && matched * 10 >= compared * 8);
    }

    #[test]
    #[ignore = "needs FS25_SAVE / FS25_GAME pointing at a real savegame and install"]
    fn test_real_save_harvest_ready_sets_each_crop_to_its_own_state() {
        let (save, game, map) = real_save_copy("harvest_ready");
        let (_, layout) = load_map_data(&game, &map).unwrap();
        let farmlands: Vec<u32> = aggregate_field_data(&save, &game, &map)
            .unwrap()
            .iter()
            .filter(|d| !d.fruit_distribution.is_empty())
            .map(|d| d.farmland_id)
            .collect();
        let edits: Vec<DensityEditPayload> = farmlands
            .iter()
            .map(|&id| DensityEditPayload {
                farmland_id: id,
                set_harvest_ready: true,
                ..Default::default()
            })
            .collect();

        save_density_edits(&save, &game, &map, &edits, false).unwrap();

        let mut checked = 0;
        for d in aggregate_field_data(&save, &game, &map).unwrap() {
            for crop in &d.fruit_distribution {
                assert_eq!(crop.harvest_ready_state, layout.fruits[&crop.fruit_type].harvest_ready_state);
                let Some(ready) = crop.harvest_ready_state else { continue };
                assert_eq!(crop.avg_growth, ready as f32, "{} on farmland {}", crop.fruit_type, d.farmland_id);
                checked += 1;
            }
        }
        println!("{} crops on {} farmlands checked", checked, farmlands.len());
        let _ = std::fs::remove_dir_all(save.parent().unwrap());
    }
}
