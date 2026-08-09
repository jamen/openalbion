//! Decoded texture data, as the renderer accepts it.
//!
//! The renderer never sees a `.big` archive, an asset header or a DXT mode — the caller
//! decodes and hands over pixels. [`ImageFormat`] deliberately restates the subset of
//! `fable_data::texture::BcnEncoding` we can upload rather than sharing that type: four
//! lines of duplication for a crate boundary with no asset formats behind it.

use wgpu::TextureFormat;

/// How the bytes in a [`TextureImage`] are laid out.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    /// DXT1 — 4×4 blocks, 8 bytes each.
    Bc1,
    /// DXT3 — 4×4 blocks, 16 bytes each.
    Bc2,
    /// DXT5 — 4×4 blocks, 16 bytes each.
    Bc3,
    /// Uncompressed 8-bit RGBA.
    Rgba8,
}

impl ImageFormat {
    /// The wgpu format to create the texture with. Every variant is `Unorm`, never
    /// `UnormSrgb`: textures are sampled raw and written to a non-sRGB target, matching
    /// the original's D3D9 pipeline (AGENTS.md §3.5).
    pub fn wgpu_format(self) -> TextureFormat {
        match self {
            Self::Bc1 => TextureFormat::Bc1RgbaUnorm,
            Self::Bc2 => TextureFormat::Bc2RgbaUnorm,
            Self::Bc3 => TextureFormat::Bc3RgbaUnorm,
            Self::Rgba8 => TextureFormat::Rgba8Unorm,
        }
    }

    /// Width and height, in pixels, of one addressable block.
    pub fn block_extent(self) -> u32 {
        match self {
            Self::Bc1 | Self::Bc2 | Self::Bc3 => 4,
            Self::Rgba8 => 1,
        }
    }

    /// Bytes occupied by one block.
    pub fn block_bytes(self) -> u32 {
        match self {
            Self::Bc1 => 8,
            Self::Bc2 | Self::Bc3 => 16,
            Self::Rgba8 => 4,
        }
    }
}

/// One decoded mip level, ready to upload.
///
/// `data` is tightly packed: [`Self::bytes_per_row`] bytes per row of blocks, no padding.
#[derive(Clone, Debug)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
}

impl TextureImage {
    /// Row pitch of `data`, in bytes.
    pub fn bytes_per_row(&self) -> u32 {
        self.width.div_ceil(self.format.block_extent()) * self.format.block_bytes()
    }

    /// Whether `data` is large enough for the declared size and format. Passes check this
    /// before upload so a short buffer is a logged skip rather than a driver-level abort.
    pub fn is_complete(&self) -> bool {
        let rows = self.height.div_ceil(self.format.block_extent()) as usize;
        self.data.len() >= rows * self.bytes_per_row() as usize
    }
}
