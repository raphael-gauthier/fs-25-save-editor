// GRLE (Giants Run-Length Encoded) binary format parser
// Adapted from Paint-a-Farm/grleconvert (MIT license)
// https://github.com/Paint-a-Farm/grleconvert

use crate::error::AppError;

/// Decoded GRLE image with pixel values
#[derive(Debug, Clone)]
pub struct GrleImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrleImage {
    /// Get pixel value at (x, y)
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u8 {
        self.pixels[(y * self.width + x) as usize]
    }
}

fn read_u16_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

/// Decode RLE-compressed GRLE data
fn decode_rle(data: &[u8], expected_size: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(expected_size);
    let mut i = 1; // Skip first byte (0x00 padding)

    while i + 1 < data.len() && output.len() < expected_size {
        let prev = data[i];
        let new_val = data[i + 1];
        i += 2;

        if prev == new_val {
            // Run: read extended count with 0xff continuation
            let mut count = 0usize;
            while i < data.len() && data[i] == 0xff {
                count += 255;
                i += 1;
            }
            if i < data.len() {
                count += data[i] as usize;
                i += 1;
            }
            count += 2; // Counts are offset by 2

            let to_emit = count.min(expected_size - output.len());
            output.extend(std::iter::repeat(prev).take(to_emit));
        } else {
            // Transition: emit prev, back up to re-read new_val as next prev
            output.push(prev);
            i -= 1;
        }
    }

    // Handle trailing single byte that couldn't form a pair
    if i < data.len() && output.len() < expected_size {
        output.push(data[i]);
    }

    output.resize(expected_size, 0);
    output
}

/// Parse a GRLE binary file into a GrleImage
///
/// GRLE header (20 bytes):
///   0-3:   Magic "GRLE"
///   4-5:   Version (u16 LE, typically 1)
///   6-7:   Width / 256 (u16 LE)
///   8-9:   Padding
///   10-11: Height / 256 (u16 LE)
///   12-19: Metadata + compressed size
///   20+:   RLE-compressed pixel data
pub fn parse_grle(data: &[u8]) -> Result<GrleImage, AppError> {
    if data.len() < 20 {
        return Err(AppError::DensityMapError {
            message: "GRLE file too small".to_string(),
        });
    }

    if &data[0..4] != b"GRLE" {
        return Err(AppError::DensityMapError {
            message: "Invalid GRLE magic bytes".to_string(),
        });
    }

    let width = (read_u16_le(&data, 6) as u32) * 256;
    let height = (read_u16_le(&data, 10) as u32) * 256;

    if width == 0 || height == 0 {
        return Err(AppError::DensityMapError {
            message: format!("Invalid GRLE dimensions: {}x{}", width, height),
        });
    }

    let compressed_data = &data[20..];
    let expected_size = (width * height) as usize;
    let pixels = decode_rle(compressed_data, expected_size);

    Ok(GrleImage {
        width,
        height,
        pixels,
    })
}

impl GrleImage {
    /// Set pixel value at (x, y)
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, value: u8) {
        self.pixels[(y * self.width + x) as usize] = value;
    }
}

/// Encode pixel data using RLE compression (reverse of decode_rle)
fn encode_rle(pixels: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(pixels.len());
    output.push(0x00); // Initial padding byte

    let mut i = 0;
    while i < pixels.len() {
        let val = pixels[i];
        // Count consecutive identical values
        let mut run_len = 1usize;
        while i + run_len < pixels.len() && pixels[i + run_len] == val {
            run_len += 1;
        }

        if run_len >= 2 {
            // Run: emit val, val, then count-2 with 0xff continuation
            output.push(val);
            output.push(val);
            let mut remaining = run_len - 2;
            while remaining >= 255 {
                output.push(0xff);
                remaining -= 255;
            }
            output.push(remaining as u8);
            i += run_len;
        } else {
            // Single value: emit it as a transition
            output.push(val);
            i += 1;
        }
    }

    output
}

/// Encode a GrleImage back to the GRLE binary format.
/// Uses the original file's header and re-encodes the pixel data.
/// Updates the compressed size field in the header (bytes 17-19 = (size-1) as u24 LE).
pub fn write_grle(image: &GrleImage, original_header: &[u8; 20]) -> Vec<u8> {
    let compressed = encode_rle(&image.pixels);
    let compressed_size_minus_1 = compressed.len().saturating_sub(1) as u32;

    let mut output = Vec::with_capacity(20 + compressed.len());
    output.extend_from_slice(original_header);

    // Update compressed size field: bytes 16-19 = (compressed_size - 1) << 8 as u32 LE
    // i.e., byte 16 = 0x00, bytes 17-19 = (compressed_size - 1) as u24 LE
    output[16] = 0;
    output[17] = (compressed_size_minus_1 & 0xFF) as u8;
    output[18] = ((compressed_size_minus_1 >> 8) & 0xFF) as u8;
    output[19] = ((compressed_size_minus_1 >> 16) & 0xFF) as u8;

    output.extend_from_slice(&compressed);
    output
}

/// Read a GRLE file, returning both the parsed image and the original header bytes
pub fn parse_grle_with_header(data: &[u8]) -> Result<(GrleImage, [u8; 20]), AppError> {
    let image = parse_grle(data)?;
    let mut header = [0u8; 20];
    header.copy_from_slice(&data[..20]);
    Ok((image, header))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_rle_uniform() {
        // All zeros: 0x00 padding, then (0, 0, count)
        let mut data = vec![0x00, 0x00, 0x00];
        // count = 8 means 10 total pixels (count + 2)
        data.push(8);
        let result = decode_rle(&data, 10);
        assert_eq!(result, vec![0u8; 10]);
    }

    #[test]
    fn test_decode_rle_transition() {
        // 0x00 padding, then alternating values: 1, 2, 3
        let data = vec![0x00, 1, 2, 3, 3, 0]; // 1→2 transition, 2→3 transition, then 3,3 run of 2
        let result = decode_rle(&data, 4);
        assert_eq!(result, vec![1, 2, 3, 3]);
    }

    #[test]
    fn test_parse_grle_invalid_magic() {
        let data = vec![0x00; 20];
        assert!(parse_grle(&data).is_err());
    }

    #[test]
    fn test_parse_grle_too_small() {
        let data = vec![0x47, 0x52, 0x4C, 0x45]; // Just "GRLE"
        assert!(parse_grle(&data).is_err());
    }

    #[test]
    fn test_encode_decode_rle_roundtrip() {
        // Mixed data: runs + transitions + trailing single value
        let pixels = vec![5, 5, 5, 3, 2, 1, 7, 7, 4];
        let encoded = encode_rle(&pixels);
        let decoded = decode_rle(&encoded, pixels.len());
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_rle_all_different() {
        let pixels = vec![1, 2, 3, 4, 5];
        let encoded = encode_rle(&pixels);
        let decoded = decode_rle(&encoded, pixels.len());
        assert_eq!(pixels, decoded);
    }

    #[test]
    fn test_encode_decode_rle_uniform() {
        let pixels = vec![42u8; 1000];
        let encoded = encode_rle(&pixels);
        let decoded = decode_rle(&encoded, pixels.len());
        assert_eq!(pixels, decoded);
    }
}
