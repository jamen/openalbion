//! Parser for TGA (Truevision TGA) image files.
//!
//! This is a minimal implementation that supports the common uncompressed
//! true-color formats used in Fable's data files.

use crate::bytes::{take, take_bytes};
use derive_more::{Display, Error};

/// A parsed TGA image.
#[derive(Clone, Debug)]
pub struct Tga {
    pub header: Header,
    /// Raw pixel data in the format specified by the header.
    /// For 24-bit images: BGR order, bottom-to-top rows.
    /// For 32-bit images: BGRA order, bottom-to-top rows.
    pub pixels: Vec<u8>,
}

impl Tga {
    /// Parse a TGA image from bytes.
    pub fn parse(input: &[u8]) -> Result<Self, TgaError> {
        let mut inp = input;
        let header = Header::parse(&mut inp)?;

        // Skip image ID if present
        if header.id_length > 0 {
            let _ =
                take_bytes(&mut inp, header.id_length as usize).map_err(|_| TgaError::ImageId)?;
        }

        // Skip color map if present
        if header.color_map_type != 0 {
            let color_map_size =
                header.color_map_length as usize * (header.color_map_entry_size as usize / 8);
            let _ = take_bytes(&mut inp, color_map_size).map_err(|_| TgaError::ColorMap)?;
        }

        // Read pixel data
        let bytes_per_pixel = header.pixel_depth as usize / 8;
        let pixel_data_size = header.width as usize * header.height as usize * bytes_per_pixel;

        match header.image_type {
            // Uncompressed true-color
            2 => {
                let pixels =
                    take_bytes(&mut inp, pixel_data_size).map_err(|_| TgaError::PixelData)?;
                Ok(Self {
                    header,
                    pixels: pixels.to_vec(),
                })
            }
            // RLE compressed true-color
            10 => {
                let pixels = Self::decode_rle(&mut inp, pixel_data_size, bytes_per_pixel)?;
                Ok(Self { header, pixels })
            }
            other => Err(TgaError::UnsupportedImageType(other)),
        }
    }

    /// Decode RLE compressed pixel data.
    fn decode_rle(
        inp: &mut &[u8],
        expected_size: usize,
        bytes_per_pixel: usize,
    ) -> Result<Vec<u8>, TgaError> {
        let mut pixels = Vec::with_capacity(expected_size);

        while pixels.len() < expected_size {
            let packet_header = take::<u8>(inp).map_err(|_| TgaError::RlePacket)?;
            let count = (packet_header & 0x7F) as usize + 1;

            if packet_header & 0x80 != 0 {
                // RLE packet: single pixel repeated
                let pixel = take_bytes(inp, bytes_per_pixel).map_err(|_| TgaError::PixelData)?;
                for _ in 0..count {
                    pixels.extend_from_slice(pixel);
                }
            } else {
                // Raw packet: sequence of pixels
                let data =
                    take_bytes(inp, count * bytes_per_pixel).map_err(|_| TgaError::PixelData)?;
                pixels.extend_from_slice(data);
            }
        }

        Ok(pixels)
    }

    /// Get the width of the image.
    pub fn width(&self) -> u32 {
        self.header.width as u32
    }

    /// Get the height of the image.
    pub fn height(&self) -> u32 {
        self.header.height as u32
    }

    /// Get the number of bytes per pixel.
    pub fn bytes_per_pixel(&self) -> usize {
        self.header.pixel_depth as usize / 8
    }

    /// Check if the image origin is at the top-left (vs bottom-left).
    pub fn is_top_origin(&self) -> bool {
        (self.header.image_descriptor & 0x20) != 0
    }

    /// Get pixel data as RGBA, converting from TGA's native format.
    /// Returns data in top-to-bottom row order regardless of original origin.
    pub fn to_rgba(&self) -> Vec<u8> {
        let width = self.header.width as usize;
        let height = self.header.height as usize;
        let bpp = self.bytes_per_pixel();
        let mut rgba = vec![0u8; width * height * 4];

        for y in 0..height {
            // Handle origin: TGA default is bottom-left
            let src_y = if self.is_top_origin() {
                y
            } else {
                height - 1 - y
            };

            for x in 0..width {
                let src_idx = (src_y * width + x) * bpp;
                let dst_idx = (y * width + x) * 4;

                // TGA stores BGR(A), convert to RGBA
                rgba[dst_idx] = self.pixels[src_idx + 2]; // R
                rgba[dst_idx + 1] = self.pixels[src_idx + 1]; // G
                rgba[dst_idx + 2] = self.pixels[src_idx]; // B
                rgba[dst_idx + 3] = if bpp >= 4 {
                    self.pixels[src_idx + 3]
                } else {
                    255
                }; // A
            }
        }

        rgba
    }

    /// Get a pixel value at (x, y) as [R, G, B, A].
    /// Coordinates are in standard top-left origin.
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let width = self.header.width as usize;
        let height = self.header.height as usize;
        let bpp = self.bytes_per_pixel();

        let x = x as usize;
        let y = y as usize;

        // Handle origin: TGA default is bottom-left
        let src_y = if self.is_top_origin() {
            y
        } else {
            height - 1 - y
        };

        let idx = (src_y * width + x) * bpp;

        // TGA stores BGR(A), convert to RGBA
        [
            self.pixels[idx + 2],                              // R
            self.pixels[idx + 1],                              // G
            self.pixels[idx],                                  // B
            if bpp >= 4 { self.pixels[idx + 3] } else { 255 }, // A
        ]
    }
}

/// TGA file header (18 bytes).
#[derive(Clone, Copy, Debug, Default)]
pub struct Header {
    /// Length of image ID field (0 = no ID).
    pub id_length: u8,
    /// Color map type (0 = no color map, 1 = color map present).
    pub color_map_type: u8,
    /// Image type:
    /// - 0: No image data
    /// - 1: Uncompressed color-mapped
    /// - 2: Uncompressed true-color
    /// - 3: Uncompressed grayscale
    /// - 9: RLE color-mapped
    /// - 10: RLE true-color
    /// - 11: RLE grayscale
    pub image_type: u8,
    /// Color map first entry index.
    pub color_map_first_index: u16,
    /// Color map length (number of entries).
    pub color_map_length: u16,
    /// Color map entry size in bits.
    pub color_map_entry_size: u8,
    /// X origin of image.
    pub x_origin: u16,
    /// Y origin of image.
    pub y_origin: u16,
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// Pixel depth (bits per pixel: 8, 16, 24, or 32).
    pub pixel_depth: u8,
    /// Image descriptor:
    /// - Bits 0-3: Alpha channel depth
    /// - Bit 4: Reserved
    /// - Bit 5: Screen origin (0 = lower-left, 1 = upper-left)
    /// - Bits 6-7: Interleaving
    pub image_descriptor: u8,
}

impl Header {
    pub const BYTE_SIZE: usize = 18;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, TgaError> {
        use TgaError as E;

        let id_length = take::<u8>(inp).map_err(|_| E::Header)?;
        let color_map_type = take::<u8>(inp).map_err(|_| E::Header)?;
        let image_type = take::<u8>(inp).map_err(|_| E::Header)?;
        let color_map_first_index = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let color_map_length = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let color_map_entry_size = take::<u8>(inp).map_err(|_| E::Header)?;
        let x_origin = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let y_origin = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let width = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let height = take::<u16>(inp).map_err(|_| E::Header)?.to_le();
        let pixel_depth = take::<u8>(inp).map_err(|_| E::Header)?;
        let image_descriptor = take::<u8>(inp).map_err(|_| E::Header)?;

        Ok(Self {
            id_length,
            color_map_type,
            image_type,
            color_map_first_index,
            color_map_length,
            color_map_entry_size,
            x_origin,
            y_origin,
            width,
            height,
            pixel_depth,
            image_descriptor,
        })
    }
}

#[derive(Debug, Display, Error)]
pub enum TgaError {
    #[display("failed to parse header")]
    Header,
    #[display("failed to read image ID")]
    ImageId,
    #[display("failed to read color map")]
    ColorMap,
    #[display("failed to read pixel data")]
    PixelData,
    #[display("failed to read RLE packet")]
    RlePacket,
    #[display("unsupported image type: {_0}")]
    UnsupportedImageType(#[error(not(source))] u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        assert_eq!(Header::BYTE_SIZE, 18);
    }
}
