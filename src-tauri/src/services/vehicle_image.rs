use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::AppError;

#[derive(Clone)]
pub struct VehicleImageService {
    cache_dir: PathBuf,
    index_cache: Arc<Mutex<HashMap<String, Option<PathBuf>>>>,
}

impl VehicleImageService {
    pub fn new(cache_dir: PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(&cache_dir)?;
        Ok(Self {
            cache_dir,
            index_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Extract the image path from vehicle XML content.
    /// Handles two formats:
    /// - Direct: `<storeData><image>$data/.../store_x.png</image></storeData>`
    /// - ParentFile set: `<set path="vehicle.storeData.image" value="path/store_x.png"/>`
    fn extract_image_from_xml(xml_content: &str) -> Option<String> {
        let mut reader = Reader::from_str(xml_content);

        let mut in_store_data = false;
        let mut in_image = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = e.name();
                    if tag.as_ref() == b"storeData" {
                        in_store_data = true;
                    } else if in_store_data && tag.as_ref() == b"image" {
                        in_image = true;
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    // Handle <set path="vehicle.storeData.image" value="..."/> in parentFile mods
                    if e.name().as_ref() == b"set" {
                        let mut path_val = String::new();
                        let mut value_val = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"path" => {
                                    path_val =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"value" => {
                                    value_val =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                                _ => {}
                            }
                        }
                        if path_val == "vehicle.storeData.image" && !value_val.is_empty() {
                            return Some(value_val);
                        }
                    }
                }
                Ok(Event::Text(ref text)) => {
                    if in_image {
                        if let Ok(value) = text.unescape() {
                            let trimmed = value.trim().to_string();
                            if !trimmed.is_empty() {
                                return Some(trimmed);
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag = e.name();
                    if tag.as_ref() == b"image" {
                        in_image = false;
                    } else if tag.as_ref() == b"storeData" {
                        in_store_data = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
        None
    }

    /// Try to convert an absolute mod path to $moddir$ format.
    /// e.g. "C:/Users/.../mods/FS25_CaseAxialFlow/axialFlow150.xml"
    ///    → "$moddir$FS25_CaseAxialFlow/axialFlow150.xml"
    fn try_convert_absolute_mod_path(path: &str) -> Option<String> {
        let normalized = path.replace('\\', "/");
        // Look for "/mods/" segment in the path
        let mods_idx = normalized.find("/mods/")?;
        let after_mods = &normalized[mods_idx + 6..]; // skip "/mods/"
        // after_mods is now "ModName/internal/path.xml"
        if after_mods.contains('/') {
            Some(format!("$moddir${}", after_mods))
        } else {
            None
        }
    }

    /// Resolve $data/ prefix to the actual game data path, and fix .png → .dds extension.
    fn resolve_dds_path(game_path: &Path, image_ref: &str) -> PathBuf {
        let resolved = if image_ref.starts_with("$data/") {
            let relative = &image_ref[6..]; // strip "$data/"
            game_path.join("data").join(relative.replace('/', "\\"))
        } else {
            game_path.join(image_ref.replace('/', "\\"))
        };

        if resolved.extension().and_then(|e| e.to_str()) == Some("png") {
            resolved.with_extension("dds")
        } else {
            resolved
        }
    }

    /// Convert raw DDS bytes to a 256×256 PNG file.
    fn convert_dds_bytes_to_png(dds_data: &[u8], png_path: &Path) -> Result<(), AppError> {
        let dds = image_dds::ddsfile::Dds::read(&mut std::io::Cursor::new(dds_data)).map_err(
            |e| AppError::ImageError {
                message: format!("DDS parse error: {}", e),
            },
        )?;
        let img = image_dds::image_from_dds(&dds, 0).map_err(|e| AppError::ImageError {
            message: format!("DDS decode error: {}", e),
        })?;

        let resized =
            image::imageops::resize(&img, 256, 256, image::imageops::FilterType::Lanczos3);

        if let Some(parent) = png_path.parent() {
            fs::create_dir_all(parent)?;
        }

        resized.save(png_path)?;
        Ok(())
    }

    /// Resolve a vehicle image: check cache, parse XML, convert DDS, return PNG path.
    pub fn resolve_image(
        &self,
        game_path: &Path,
        mods_dir: &Path,
        vehicle_filename: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        // Skip encrypted DLC vehicles (no way to extract)
        if vehicle_filename.contains("$pdlcdir$") || vehicle_filename.contains("$dlcdir$") {
            return Ok(None);
        }

        // Skip absolute paths to PDLC/DLC directories
        let lower = vehicle_filename.to_lowercase();
        if lower.contains("/pdlc/") || lower.contains("\\pdlc\\") {
            return Ok(None);
        }

        // Check in-memory cache
        {
            let cache = self.index_cache.lock().unwrap();
            if let Some(cached) = cache.get(vehicle_filename) {
                return Ok(cached.clone());
            }
        }

        let result = if vehicle_filename.contains("$moddir$") {
            self.resolve_mod_image(game_path, mods_dir, vehicle_filename)
        } else if let Some(converted) = Self::try_convert_absolute_mod_path(vehicle_filename) {
            // Sale items use absolute paths like "C:/.../mods/ModName/vehicle.xml"
            // Convert to $moddir$ format so resolve_mod_image can handle it
            self.resolve_mod_image(game_path, mods_dir, &converted)
        } else {
            self.resolve_base_game_image(game_path, vehicle_filename)
        };

        // Store in cache regardless of result
        let cache_value = match &result {
            Ok(path) => path.clone(),
            Err(_) => None,
        };
        {
            let mut cache = self.index_cache.lock().unwrap();
            cache.insert(vehicle_filename.to_string(), cache_value);
        }

        result
    }

    /// Resolve image for a base game vehicle (files on disk).
    fn resolve_base_game_image(
        &self,
        game_path: &Path,
        vehicle_filename: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        let cache_key = vehicle_filename
            .replace(['/', '\\'], "_")
            .replace(".xml", ".png");
        let png_path = self.cache_dir.join(&cache_key);

        if png_path.exists() {
            return Ok(Some(png_path));
        }

        // Savegame filenames are like "data/vehicles/brand/model/model.xml"
        let xml_path = game_path.join(vehicle_filename.replace('/', "\\"));
        if !xml_path.exists() {
            return Ok(None);
        }

        let xml_content = fs::read_to_string(&xml_path).map_err(|_| AppError::ImageError {
            message: format!("Cannot read {}", xml_path.display()),
        })?;

        let image_ref = match Self::extract_image_from_xml(&xml_content) {
            Some(r) => r,
            None => return Ok(None),
        };

        let dds_path = Self::resolve_dds_path(game_path, &image_ref);
        if !dds_path.exists() {
            return Ok(None);
        }

        let dds_data = fs::read(&dds_path)?;
        match Self::convert_dds_bytes_to_png(&dds_data, &png_path) {
            Ok(()) => Ok(Some(png_path)),
            Err(_) => Ok(None),
        }
    }

    /// Resolve image for a mod vehicle (files inside a .zip).
    /// Filename format: `$moddir$ModName/path/to/vehicle.xml`
    fn resolve_mod_image(
        &self,
        game_path: &Path,
        mods_dir: &Path,
        vehicle_filename: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        let cache_key = vehicle_filename
            .replace(['/', '\\', '$'], "_")
            .replace(".xml", ".png");
        let png_path = self.cache_dir.join(&cache_key);

        if png_path.exists() {
            return Ok(Some(png_path));
        }

        // Parse "$moddir$ModName/internal/path.xml"
        let after_prefix = vehicle_filename
            .strip_prefix("$moddir$")
            .unwrap_or(vehicle_filename);
        let (mod_name, internal_xml_path) = match after_prefix.split_once('/') {
            Some((name, path)) => (name, path),
            None => return Ok(None),
        };

        let zip_path = mods_dir.join(format!("{}.zip", mod_name));
        if !zip_path.exists() {
            return Ok(None);
        }

        // Read the vehicle XML from the zip
        let xml_content = match Self::read_file_from_zip(&zip_path, internal_xml_path) {
            Some(data) => String::from_utf8_lossy(&data).to_string(),
            None => return Ok(None),
        };

        let image_ref = match Self::extract_image_from_xml(&xml_content) {
            Some(r) => r,
            None => return Ok(None),
        };

        // If image_ref starts with $data/, read from game directory (not the zip)
        if image_ref.starts_with("$data/") {
            let dds_path = Self::resolve_dds_path(game_path, &image_ref);
            if !dds_path.exists() {
                return Ok(None);
            }
            let dds_data = fs::read(&dds_path)?;
            return match Self::convert_dds_bytes_to_png(&dds_data, &png_path) {
                Ok(()) => Ok(Some(png_path)),
                Err(_) => Ok(None),
            };
        }

        // Image path is relative to the zip root, fix .png → .dds
        let dds_internal_path = if image_ref.ends_with(".png") {
            format!("{}.dds", &image_ref[..image_ref.len() - 4])
        } else {
            image_ref.clone()
        };

        let dds_data = match Self::read_file_from_zip(&zip_path, &dds_internal_path) {
            Some(data) => data,
            None => return Ok(None),
        };

        match Self::convert_dds_bytes_to_png(&dds_data, &png_path) {
            Ok(()) => Ok(Some(png_path)),
            Err(_) => Ok(None),
        }
    }

    /// Read a file from inside a zip archive. Returns None if not found.
    fn read_file_from_zip(zip_path: &Path, internal_path: &str) -> Option<Vec<u8>> {
        let file = fs::File::open(zip_path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;

        let normalized = internal_path.replace('\\', "/");
        let mut entry = match archive.by_name(&normalized) {
            Ok(e) => e,
            Err(_) => return None,
        };

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).ok()?;
        Some(buf)
    }

    /// Resolve images for a batch of vehicles. Errors on individual vehicles return None.
    pub fn resolve_images_batch(
        &self,
        game_path: &Path,
        mods_dir: &Path,
        filenames: &[String],
    ) -> Vec<(String, Option<PathBuf>)> {
        filenames
            .iter()
            .map(|f| {
                let result = self.resolve_image(game_path, mods_dir, f).unwrap_or(None);
                (f.clone(), result)
            })
            .collect()
    }

    /// Clear the disk cache and in-memory index.
    pub fn clear_cache(&self) -> Result<u64, AppError> {
        let size = self.cache_size();
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.path().is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
        }
        let mut cache = self.index_cache.lock().unwrap();
        cache.clear();
        Ok(size)
    }

    /// Get total size of cached images in bytes.
    pub fn cache_size(&self) -> u64 {
        if !self.cache_dir.exists() {
            return 0;
        }
        fs::read_dir(&self.cache_dir)
            .ok()
            .map(|entries| {
                entries
                    .flatten()
                    .filter_map(|e| e.metadata().ok())
                    .filter(|m| m.is_file())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0)
    }
}
