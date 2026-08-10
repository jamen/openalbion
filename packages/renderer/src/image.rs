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
    /// Uncompressed single-channel 8-bit. The landscape blend tables are `A8` in the
    /// original; one channel either way, and the shader reads it as `.r`.
    R8,
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
            Self::R8 => TextureFormat::R8Unorm,
        }
    }

    /// Width and height, in pixels, of one addressable block.
    pub fn block_extent(self) -> u32 {
        match self {
            Self::Bc1 | Self::Bc2 | Self::Bc3 => 4,
            Self::Rgba8 | Self::R8 => 1,
        }
    }

    /// Bytes occupied by one block.
    pub fn block_bytes(self) -> u32 {
        match self {
            Self::Bc1 => 8,
            Self::Bc2 | Self::Bc3 => 16,
            Self::Rgba8 => 4,
            Self::R8 => 1,
        }
    }
}

/// A decoded texture and its mip chain, ready to upload.
///
/// Each level is tightly packed: [`Self::bytes_per_row`] bytes per row of blocks, no padding.
/// `width`/`height` describe level 0; level `i` is `max(width >> i, 1)` by
/// `max(height >> i, 1)`, which is how the GPU derives them too.
///
/// The chain is not something the renderer builds — Fable ships one per texture asset and the
/// caller passes it through (AGENTS.md §3.12). A single-level image is the ordinary case for
/// anything generated at runtime; see [`Self::single`].
#[derive(Clone, Debug)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    /// Mip levels, largest first. Never empty for an image that should draw.
    pub levels: Vec<Vec<u8>>,
}

impl TextureImage {
    /// An image with no mip chain — one level, at `width` × `height`.
    pub fn single(width: u32, height: u32, format: ImageFormat, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            format,
            levels: vec![data],
        }
    }

    /// Dimensions of mip `level`, as the GPU derives them.
    pub fn level_size(&self, level: usize) -> (u32, u32) {
        (
            (self.width >> level).max(1),
            (self.height >> level).max(1),
        )
    }

    /// Row pitch of mip `level`, in bytes.
    pub fn bytes_per_row(&self, level: usize) -> u32 {
        let (width, _) = self.level_size(level);
        width.div_ceil(self.format.block_extent()) * self.format.block_bytes()
    }

    /// Whether every level is large enough for its declared size and format. Passes check this
    /// before upload so a short buffer is a logged skip rather than a driver-level abort.
    pub fn is_complete(&self) -> bool {
        !self.levels.is_empty()
            && self.levels.iter().enumerate().all(|(level, data)| {
                let (_, height) = self.level_size(level);
                let rows = height.div_ceil(self.format.block_extent()) as usize;
                data.len() >= rows * self.bytes_per_row(level) as usize
            })
    }
}
