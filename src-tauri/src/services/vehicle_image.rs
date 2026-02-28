use std::collections::HashMap;
use std::fs;
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

    /// Extract the image path from a vehicle XML's <storeData><image> text content.
    /// FS25 format: `<storeData><image>$data/vehicles/brand/model/store_model.png</image></storeData>`
    fn extract_image_tag(xml_path: &Path) -> Option<String> {
        let content = fs::read_to_string(xml_path).ok()?;
        let mut reader = Reader::from_str(&content);

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
                Ok(Event::Text(ref text)) => {
                    if in_image {
                        let value = text.unescape().ok()?.trim().to_string();
                        if !value.is_empty() {
                            return Some(value);
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

    /// Resolve $data/ prefix to the actual game data path, and fix extension.
    fn resolve_dds_path(game_path: &Path, image_ref: &str) -> PathBuf {
        let resolved = if image_ref.starts_with("$data/") {
            let relative = &image_ref[6..]; // strip "$data/"
            game_path.join("data").join(relative.replace('/', "\\"))
        } else {
            game_path.join(image_ref.replace('/', "\\"))
        };

        // FS25 references .png in XML but actual files are .dds
        if resolved.extension().and_then(|e| e.to_str()) == Some("png") {
            resolved.with_extension("dds")
        } else {
            resolved
        }
    }

    /// Convert a DDS file to a 256x256 PNG.
    fn convert_dds_to_png(dds_path: &Path, png_path: &Path) -> Result<(), AppError> {
        let dds_data = fs::read(dds_path)?;
        let dds = image_dds::ddsfile::Dds::read(&mut std::io::Cursor::new(&dds_data)).map_err(
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
        vehicle_filename: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        // Skip modded vehicles, DLC vehicles, and non-vehicle objects
        if vehicle_filename.starts_with("mods/")
            || vehicle_filename.starts_with("mods\\")
            || vehicle_filename.contains("$moddir$")
            || vehicle_filename.contains("$pdlcdir$")
            || vehicle_filename.contains("$dlcdir$")
        {
            return Ok(None);
        }

        // Check in-memory cache
        {
            let cache = self.index_cache.lock().unwrap();
            if let Some(cached) = cache.get(vehicle_filename) {
                return Ok(cached.clone());
            }
        }

        let result = self.resolve_image_inner(game_path, vehicle_filename);

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

    fn resolve_image_inner(
        &self,
        game_path: &Path,
        vehicle_filename: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        // Build a cache key from the filename (replace path separators)
        let cache_key = vehicle_filename
            .replace(['/', '\\'], "_")
            .replace(".xml", ".png");
        let png_path = self.cache_dir.join(&cache_key);

        // If PNG already exists in disk cache, return it
        if png_path.exists() {
            return Ok(Some(png_path));
        }

        // Find the vehicle XML in the game folder
        // Savegame filenames are like "data/vehicles/brand/model/model.xml"
        let xml_path = game_path.join(vehicle_filename.replace('/', "\\"));
        if !xml_path.exists() {
            return Ok(None);
        }

        // Extract image reference from XML
        let image_ref = match Self::extract_image_tag(&xml_path) {
            Some(r) => r,
            None => return Ok(None),
        };

        // Resolve to actual DDS file
        let dds_path = Self::resolve_dds_path(game_path, &image_ref);
        if !dds_path.exists() {
            return Ok(None);
        }

        // Convert DDS to PNG
        match Self::convert_dds_to_png(&dds_path, &png_path) {
            Ok(()) => Ok(Some(png_path)),
            Err(_) => Ok(None),
        }
    }

    /// Resolve images for a batch of vehicles. Errors on individual vehicles return None.
    pub fn resolve_images_batch(
        &self,
        game_path: &Path,
        filenames: &[String],
    ) -> Vec<(String, Option<PathBuf>)> {
        filenames
            .iter()
            .map(|f| {
                let result = self.resolve_image(game_path, f).unwrap_or(None);
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
