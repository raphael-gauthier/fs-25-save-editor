// GDM (Giants Density Map) binary format parser
// Adapted from Paint-a-Farm/grleconvert (MIT license)
// https://github.com/Paint-a-Farm/grleconvert

use crate::error::AppError;

/// Decoded GDM image with combined channel values per pixel
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GdmImage {
    pub width: u32,
    pub height: u32,
    pub total_channels: u8,
    /// Raw combined pixel values. For fruits GDM (10 channels, split at 5):
    /// bits 0-4 = fruit type index, bits 5-9 = growth state
    pub pixels: Vec<u16>,
}

impl GdmImage {
    /// Get raw combined pixel value at (x, y)
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u16 {
        self.pixels[(y * self.width + x) as usize]
    }

    /// Extract a range of bits from a pixel value
    #[inline]
    pub fn extract_bits(value: u16, start_bit: u8, num_bits: u8) -> u16 {
        (value >> start_bit) & ((1u16 << num_bits) - 1)
    }
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Decode a single GDM chunk (32x32 pixels typically)
fn decode_block(data: &[u8], pos: usize, chunk_size: usize) -> Result<(Vec<u16>, usize), AppError> {
    if pos + 2 > data.len() {
        return Err(AppError::DensityMapError {
            message: "Unexpected end of GDM block data".to_string(),
        });
    }

    let bit_depth = data[pos] as usize;
    let palette_count = data[pos + 1] as usize;
    let palette_size = 2 * palette_count;
    let bitmap_size = if bit_depth > 0 {
        bit_depth * 128
    } else {
        0
    };
    let block_size = 2 + palette_size + bitmap_size;

    if pos + block_size > data.len() {
        return Err(AppError::DensityMapError {
            message: "GDM block extends beyond data".to_string(),
        });
    }

    let palette: Vec<u16> = (0..palette_count)
        .map(|i| u16::from_le_bytes([data[pos + 2 + i * 2], data[pos + 3 + i * 2]]))
        .collect();

    let total_pixels = chunk_size * chunk_size;
    let mut pixels = Vec::with_capacity(total_pixels);

    if bit_depth == 0 {
        // Uniform chunk
        let value = *palette.first().unwrap_or(&0);
        pixels.resize(total_pixels, value);
    } else {
        let bitmap = &data[pos + 2 + palette_size..pos + 2 + palette_size + bitmap_size];
        let mask = (1u16 << bit_depth) - 1;

        for pixel_idx in 0..total_pixels {
            let bit_pos = pixel_idx * bit_depth;
            let byte_idx = bit_pos / 8;
            let bit_offset = bit_pos % 8;

            // Read up to 3 bytes to support bit_depth > 8 (e.g. ground GDM with 11 channels)
            let mut raw_value = bitmap[byte_idx] as u32;
            if byte_idx + 1 < bitmap.len() {
                raw_value |= (bitmap[byte_idx + 1] as u32) << 8;
            }
            if byte_idx + 2 < bitmap.len() {
                raw_value |= (bitmap[byte_idx + 2] as u32) << 16;
            }

            let idx_or_value = ((raw_value >> bit_offset) & (mask as u32)) as usize;

            let pixel_value = if bit_depth <= 2 && !palette.is_empty() {
                *palette.get(idx_or_value).unwrap_or(&0)
            } else {
                idx_or_value as u16
            };

            pixels.push(pixel_value);
        }
    }

    Ok((pixels, block_size))
}

/// Parse a GDM binary file into a GdmImage
///
/// GDM has two variants:
/// - "MDF (0x22): Extended format with 16-byte header
/// - !MDF (0x21): Legacy format with 9-byte header
///
/// Multiple compression ranges are combined by bit-shifting
pub fn parse_gdm(data: &[u8]) -> Result<GdmImage, AppError> {
    if data.len() < 16 {
        return Err(AppError::DensityMapError {
            message: "GDM file too small".to_string(),
        });
    }

    let magic = &data[0..4];
    if magic != b"\"MDF" && magic != b"!MDF" {
        return Err(AppError::DensityMapError {
            message: "Invalid GDM magic bytes".to_string(),
        });
    }

    let (dimension, num_channels, chunk_size, num_compression_ranges, header_size) =
        if magic == b"\"MDF" {
            let version = read_u32_le(&data, 4);
            if version != 0 {
                return Err(AppError::DensityMapError {
                    message: format!("Unsupported GDM version: {}", version),
                });
            }

            let dim_log2 = data[8] as usize;
            let chunk_log2 = data[9] as usize;
            let num_channels = data[11] as usize;
            let num_compression_ranges = data[12] as usize;

            let dimension = 1usize << (dim_log2 + 5);
            let chunk_size = 1usize << chunk_log2;

            (dimension, num_channels, chunk_size, num_compression_ranges, 16usize)
        } else {
            let dim_log2 = data[4] as usize;
            let chunk_log2 = data[5] as usize;
            let num_channels = data[7] as usize;
            let num_compression_ranges = data[8] as usize;

            let dimension = 1usize << (dim_log2 + 5);
            let chunk_size = 1usize << chunk_log2;

            (dimension, num_channels, chunk_size, num_compression_ranges, 9usize)
        };

    if num_compression_ranges == 0 || num_channels == 0 {
        return Err(AppError::DensityMapError {
            message: "Invalid GDM: zero channels or compression ranges".to_string(),
        });
    }

    // Read compression range boundaries
    let mut compression_boundaries = vec![0u8];
    for i in 0..(num_compression_ranges.saturating_sub(1)) {
        compression_boundaries.push(data[header_size + i]);
    }
    compression_boundaries.push(num_channels as u8);

    let mut bits_per_range = Vec::new();
    for i in 0..num_compression_ranges {
        let start_ch = compression_boundaries[i] as usize;
        let end_ch = compression_boundaries[i + 1] as usize;
        bits_per_range.push(end_ch - start_ch);
    }

    let chunks_per_dim = dimension / chunk_size;
    let total_chunks = chunks_per_dim * chunks_per_dim;

    let compression_boundaries_size = if num_compression_ranges > 1 {
        num_compression_ranges - 1
    } else {
        0
    };
    let data_start = header_size + compression_boundaries_size;

    // Decode all chunks
    let mut image = vec![0u16; dimension * dimension];
    let mut pos = data_start;

    for chunk_idx in 0..total_chunks {
        let mut range_values: Vec<Vec<u16>> = Vec::new();

        for _range_idx in 0..num_compression_ranges {
            let (pixels, block_size) = decode_block(&data, pos, chunk_size)?;
            range_values.push(pixels);
            pos += block_size;
        }

        let chunk_row = chunk_idx / chunks_per_dim;
        let chunk_col = chunk_idx % chunks_per_dim;
        let base_y = chunk_row * chunk_size;
        let base_x = chunk_col * chunk_size;

        for pixel_idx in 0..(chunk_size * chunk_size) {
            // Combine values from all compression ranges
            let mut combined: u32 = 0;
            let mut shift = 0;
            for (range_idx, pixels) in range_values.iter().enumerate() {
                let val = pixels[pixel_idx] as u32;
                combined |= val << shift;
                shift += bits_per_range[range_idx];
            }

            let py = pixel_idx / chunk_size;
            let px = pixel_idx % chunk_size;
            let img_x = base_x + px;
            let img_y = base_y + py;
            let img_idx = img_y * dimension + img_x;

            image[img_idx] = combined as u16;
        }
    }

    Ok(GdmImage {
        width: dimension as u32,
        height: dimension as u32,
        total_channels: num_channels as u8,
        pixels: image,
    })
}

impl GdmImage {
    /// Set raw combined pixel value at (x, y)
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u16) {
        self.pixels[(y * self.width + x) as usize] = value;
    }
}

/// Encode a single GDM chunk.
/// Matches the game's encoding strategy:
/// - 1 unique value: uniform (bit_depth=0, palette_count=1)
/// - 2-4 unique values: palette encoding (bit_depth=1-2, palette lookup)
/// - >4 unique values: direct encoding (bit_depth = range_bits, no palette)
///
/// `range_bits` is the number of channels in this compression range.
/// The game always uses range_bits as bit_depth for direct encoding.
fn encode_block(pixels: &[u16], chunk_size: usize, range_bits: usize) -> Vec<u8> {
    let total_pixels = chunk_size * chunk_size;
    assert!(pixels.len() == total_pixels);

    // Collect unique values (stop early if >4)
    let mut unique: Vec<u16> = Vec::new();
    for &p in pixels {
        if !unique.contains(&p) {
            unique.push(p);
            if unique.len() > 4 {
                break;
            }
        }
    }

    if unique.len() == 1 {
        // Uniform chunk: bit_depth=0, palette_count=1, palette=[value]
        let mut out = Vec::with_capacity(4);
        out.push(0); // bit_depth
        out.push(1); // palette_count
        out.extend_from_slice(&unique[0].to_le_bytes());
        return out;
    }

    if unique.len() <= 4 {
        // Palette encoding: bit_depth=1 or 2, bitmap stores palette indices
        // Do NOT sort — preserve insertion order to match game's encoder
        let palette_count = unique.len();
        let bit_depth = if palette_count <= 2 { 1 } else { 2 };
        let bitmap_size = bit_depth * 128;
        let mut bitmap = vec![0u8; bitmap_size];

        for (pixel_idx, &val) in pixels.iter().enumerate() {
            let idx = unique.iter().position(|&v| v == val).unwrap() as u32;
            let bit_pos = pixel_idx * bit_depth;
            let byte_idx = bit_pos / 8;
            let bit_offset = bit_pos % 8;

            bitmap[byte_idx] |= (idx << bit_offset) as u8;
            if byte_idx + 1 < bitmap.len() {
                bitmap[byte_idx + 1] |= (idx >> (8 - bit_offset)) as u8;
            }
        }

        let mut out = Vec::with_capacity(2 + palette_count * 2 + bitmap_size);
        out.push(bit_depth as u8);
        out.push(palette_count as u8);
        for &val in &unique {
            out.extend_from_slice(&val.to_le_bytes());
        }
        out.extend_from_slice(&bitmap);
        return out;
    }

    // Direct encoding: use range_bits as bit_depth (matches game behavior)
    let bit_depth = range_bits;

    let bitmap_size = bit_depth * 128; // bit_depth * total_pixels / 8 = bit_depth * 1024 / 8
    let mut bitmap = vec![0u8; bitmap_size];

    for (pixel_idx, &val) in pixels.iter().enumerate() {
        let bit_pos = pixel_idx * bit_depth;
        let byte_idx = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let shifted = (val as u32) << bit_offset;
        bitmap[byte_idx] |= shifted as u8;
        if byte_idx + 1 < bitmap.len() {
            bitmap[byte_idx + 1] |= (shifted >> 8) as u8;
        }
        if byte_idx + 2 < bitmap.len() && bit_offset + bit_depth > 16 {
            bitmap[byte_idx + 2] |= (shifted >> 16) as u8;
        }
    }

    let mut out = Vec::with_capacity(2 + bitmap_size);
    out.push(bit_depth as u8);
    out.push(0); // palette_count = 0 for direct encoding
    out.extend_from_slice(&bitmap);
    out
}

/// Calculate the byte size of a GDM block from its header
fn block_byte_size(data: &[u8], pos: usize, chunk_size: usize) -> usize {
    let bit_depth = data[pos] as usize;
    let palette_count = data[pos + 1] as usize;
    let total_pixels = chunk_size * chunk_size;
    let palette_size = 2 * palette_count;
    let bitmap_size = if bit_depth > 0 {
        (bit_depth * total_pixels + 7) / 8
    } else {
        0
    };
    2 + palette_size + bitmap_size
}

/// Encode a GdmImage back to the GDM binary format.
/// Only re-encodes chunks where pixels actually changed; copies original bytes
/// for unchanged chunks to avoid palette reordering artifacts.
pub fn write_gdm(image: &GdmImage, original_data: &[u8]) -> Result<Vec<u8>, AppError> {
    if original_data.len() < 16 {
        return Err(AppError::DensityMapError {
            message: "Original GDM data too small for re-encoding".to_string(),
        });
    }

    // Also decode the original to compare pixels
    let original_image = parse_gdm(original_data)?;

    let magic = &original_data[0..4];
    let (dimension, num_channels, chunk_size, num_compression_ranges, header_size) =
        if magic == b"\"MDF" {
            let dim_log2 = original_data[8] as usize;
            let chunk_log2 = original_data[9] as usize;
            let num_channels = original_data[11] as usize;
            let num_compression_ranges = original_data[12] as usize;
            let dimension = 1usize << (dim_log2 + 5);
            let chunk_size = 1usize << chunk_log2;
            (dimension, num_channels, chunk_size, num_compression_ranges, 16usize)
        } else if magic == b"!MDF" {
            let dim_log2 = original_data[4] as usize;
            let chunk_log2 = original_data[5] as usize;
            let num_channels = original_data[7] as usize;
            let num_compression_ranges = original_data[8] as usize;
            let dimension = 1usize << (dim_log2 + 5);
            let chunk_size = 1usize << chunk_log2;
            (dimension, num_channels, chunk_size, num_compression_ranges, 9usize)
        } else {
            return Err(AppError::DensityMapError {
                message: "Invalid GDM magic for re-encoding".to_string(),
            });
        };

    // Read compression range boundaries
    let mut compression_boundaries = vec![0u8];
    let compression_boundaries_size = if num_compression_ranges > 1 {
        num_compression_ranges - 1
    } else {
        0
    };
    for i in 0..compression_boundaries_size {
        compression_boundaries.push(original_data[header_size + i]);
    }
    compression_boundaries.push(num_channels as u8);

    let mut bits_per_range = Vec::new();
    for i in 0..num_compression_ranges {
        let start_ch = compression_boundaries[i] as usize;
        let end_ch = compression_boundaries[i + 1] as usize;
        bits_per_range.push(end_ch - start_ch);
    }

    let chunks_per_dim = dimension / chunk_size;
    let total_chunks = chunks_per_dim * chunks_per_dim;
    let data_start = header_size + compression_boundaries_size;

    // Write header + compression boundaries (copy from original)
    let mut output = Vec::with_capacity(original_data.len());
    output.extend_from_slice(&original_data[..data_start]);

    // Track position in original data for copying unchanged blocks
    let mut orig_pos = data_start;

    // Encode each chunk
    for chunk_idx in 0..total_chunks {
        let chunk_row = chunk_idx / chunks_per_dim;
        let chunk_col = chunk_idx % chunks_per_dim;
        let base_y = chunk_row * chunk_size;
        let base_x = chunk_col * chunk_size;

        // Check if ANY pixel in this chunk changed
        let mut chunk_changed = false;
        'outer: for py in 0..chunk_size {
            for px in 0..chunk_size {
                let img_x = base_x + px;
                let img_y = base_y + py;
                let idx = img_y * dimension + img_x;
                if image.pixels[idx] != original_image.pixels[idx] {
                    chunk_changed = true;
                    break 'outer;
                }
            }
        }

        if chunk_changed {
            // Re-encode all ranges for this chunk
            for (range_idx, &bits) in bits_per_range.iter().enumerate() {
                let shift: usize = bits_per_range[..range_idx].iter().sum();
                let mask = (1u16 << bits) - 1;

                let mut chunk_pixels = Vec::with_capacity(chunk_size * chunk_size);
                for py in 0..chunk_size {
                    for px in 0..chunk_size {
                        let img_x = base_x + px;
                        let img_y = base_y + py;
                        let combined = image.pixels[img_y * dimension + img_x];
                        let range_val = (combined >> shift) & mask;
                        chunk_pixels.push(range_val);
                    }
                }

                // Skip past original block
                let orig_block_size = block_byte_size(original_data, orig_pos, chunk_size);
                orig_pos += orig_block_size;

                let block = encode_block(&chunk_pixels, chunk_size, bits);
                output.extend_from_slice(&block);
            }
        } else {
            // Copy original bytes for all ranges in this chunk
            for _ in 0..num_compression_ranges {
                let orig_block_size = block_byte_size(original_data, orig_pos, chunk_size);
                output.extend_from_slice(&original_data[orig_pos..orig_pos + orig_block_size]);
                orig_pos += orig_block_size;
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits() {
        // Value: 0b_01010_00011 = fruit=3, growth=10
        let value = 0b0101000011u16;
        assert_eq!(GdmImage::extract_bits(value, 0, 5), 3);  // fruit type
        assert_eq!(GdmImage::extract_bits(value, 5, 5), 10); // growth state
    }

    #[test]
    fn test_decode_block_uniform() {
        // bit_depth=0, palette_count=1, palette=[42]
        let data = vec![0, 1, 42, 0]; // 42 as u16 LE
        let (pixels, size) = decode_block(&data, 0, 32).unwrap();
        assert_eq!(size, 4); // 2 header + 2 palette + 0 bitmap
        assert_eq!(pixels.len(), 32 * 32);
        assert!(pixels.iter().all(|&p| p == 42));
    }

    #[test]
    fn test_parse_gdm_invalid_magic() {
        let data = vec![0x00; 20];
        assert!(parse_gdm(&data).is_err());
    }

    #[test]
    fn test_parse_gdm_too_small() {
        let data = vec![0x22, 0x4D, 0x44, 0x46]; // Just magic
        assert!(parse_gdm(&data).is_err());
    }

    #[test]
    fn test_encode_decode_block_roundtrip() {
        // Test with bit_depth > 8 (simulates ground GDM with 11 channels)
        let chunk_size = 32;
        let total = chunk_size * chunk_size;
        let mut pixels: Vec<u16> = Vec::with_capacity(total);
        for i in 0..total {
            // Values up to 2047 (11 bits) to simulate ground GDM
            pixels.push((i % 2048) as u16);
        }

        let encoded = encode_block(&pixels, chunk_size, 11);
        assert_eq!(encoded[0], 11); // bit_depth matches range_bits
        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_block_uniform_roundtrip() {
        let chunk_size = 32;
        let pixels = vec![1234u16; chunk_size * chunk_size];
        let encoded = encode_block(&pixels, chunk_size, 11);
        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_block_palette_2_values() {
        // Simulates ground GDM block with 2 ground types (e.g., 0 and 4)
        let chunk_size = 32;
        let total = chunk_size * chunk_size;
        let mut pixels = vec![0u16; total];
        for i in (0..total).step_by(2) {
            pixels[i] = 4;
        }

        let encoded = encode_block(&pixels, chunk_size, 11);
        // Should use palette encoding: bit_depth=1, palette_count=2
        assert_eq!(encoded[0], 1); // bit_depth
        assert_eq!(encoded[1], 2); // palette_count

        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_block_palette_3_values() {
        let chunk_size = 32;
        let total = chunk_size * chunk_size;
        let mut pixels = vec![0u16; total];
        for i in (0..total).step_by(3) {
            pixels[i] = 100;
        }
        for i in (1..total).step_by(3) {
            pixels[i] = 500;
        }

        let encoded = encode_block(&pixels, chunk_size, 11);
        assert_eq!(encoded[0], 2); // bit_depth
        assert_eq!(encoded[1], 3); // palette_count

        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_block_palette_4_values() {
        let chunk_size = 32;
        let total = chunk_size * chunk_size;
        let mut pixels = vec![0u16; total];
        for i in 0..total {
            pixels[i] = [0, 4, 13, 2047][i % 4];
        }

        let encoded = encode_block(&pixels, chunk_size, 11);
        assert_eq!(encoded[0], 2); // bit_depth
        assert_eq!(encoded[1], 4); // palette_count

        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_block_5_values_direct() {
        // 5 unique values: direct encoding uses range_bits as bit_depth
        let chunk_size = 32;
        let total = chunk_size * chunk_size;
        let mut pixels = vec![0u16; total];
        for i in 0..total {
            pixels[i] = [0, 1, 2, 3, 4][i % 5];
        }

        let encoded = encode_block(&pixels, chunk_size, 5);
        assert_eq!(encoded[0], 5); // bit_depth = range_bits
        assert_eq!(encoded[1], 0); // palette_count = 0

        let (decoded, _size) = decode_block(&encoded, 0, chunk_size).unwrap();
        assert_eq!(pixels, decoded);
    }
}
