use crate::bytes::{take, take_bytes};
use bcndecode::BcnDecoderFormat;
use derive_more::derive::{Display, Error};

pub use bcndecode::BcnEncoding;
pub use lzo::LzoError;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub top_mip_length: usize,
    /// Number of mip levels the asset declares (`TextureMetadata::mip_maps`). All of them are
    /// in [`Self::raw_image_data`]; see [`Self::mip_levels`].
    pub mip_map_count: usize,
    pub bcn_encoding: BcnEncoding,
    pub raw_image_data: Vec<u8>,
}

/// Map Fable's `dxt_compression` tag (from a texture asset's metadata) to a BCN encoding.
///
/// Only three tags are block-compressed, and every one of them occurs in `textures.big`.
/// Measured over all 6,324 texture assets, taking bits-per-pixel as `top_mip_map_size` over
/// the block-padded pixel count (AGENTS.md §3.12):
///
/// ```text
/// dxt=31  bpp=4    3683  DXT1 (BC1)     dxt=1   bpp=32        6  uncompressed 32bpp
/// dxt=32  bpp=8    2558  DXT3 (BC2)     dxt=1   bpp=256/2048  2  degenerate headers
/// dxt=35  bpp=8       4  DXT5 (BC3)     dxt=24  bpp=16        1  D3DFMT_X1R5G5B5
/// ```
///
/// **Tag 1 is not DXT1**, despite reading like `D3DFMT_DXT1`'s ordinal:
/// `ITEMS_EXPRESSIONS_CONTAINMENT_RIGHT_ON` is 64×64 with `top_mip_map_size == 16384`, which
/// is `w * h * 4` — uncompressed 32bpp — and its chain is 21,824 = 16384+4096+1024+256+64.
/// Decoding it as BC1 yields noise, so it returns `None` and the caller skips it. Tags `3`,
/// `5`, `33` and `34` were mapped here once but match no asset in the archive.
pub fn bcn_encoding_from_dxt(dxt: u16) -> Option<BcnEncoding> {
    match dxt {
        31 => Some(BcnEncoding::Bc1),
        32 => Some(BcnEncoding::Bc2),
        35 => Some(BcnEncoding::Bc3),
        _ => None,
    }
}

/// Number of bytes one 4x4 block occupies in a BCN encoding (8 for BC1, 16 otherwise).
pub fn bcn_block_bytes(encoding: BcnEncoding) -> u32 {
    match encoding {
        BcnEncoding::Bc1 => 8,
        _ => 16,
    }
}

impl Texture {
    pub fn parse(
        input: &mut &[u8],
        width: usize,
        height: usize,
        depth: usize,
        top_mip_length: usize,
        mip_map_count: usize,
        bcn_encoding: BcnEncoding,
    ) -> Result<Self, TextureError> {
        use TextureError as E;

        // The texture is prefixed with a variable length integer that tells us how long the first
        // mip is.

        let small_length = take::<u16>(input).map_err(|_| E::SmallLength)?.to_le();

        let top_mip_compressed_length = if small_length == 0xffff {
            take::<u32>(input).map_err(|_| E::LargeLength)?.to_le()
        } else {
            small_length as u32
        };

        // The rest of the input is image data: level 0 is LZO-compressed, every level below it
        // is stored raw and follows immediately. Measured over `textures.big` — 3,978 of 4,000
        // assets have a `raw_image_data` length equal to the exact chain sum, which is only
        // possible if the sub-levels are uncompressed. AGENTS.md §3.12.

        let mut raw_image_data = Vec::new();

        if top_mip_compressed_length > 0 {
            let top_mip_compressed = take_bytes(input, top_mip_compressed_length as usize)
                .map_err(|_| E::TopMipCompressed)?;

            let top_mip = lzo::decompress(top_mip_compressed, top_mip_length)
                .map_err(E::TopMipLzoDecompress)?;

            raw_image_data.extend_from_slice(&top_mip);
        }

        raw_image_data.extend_from_slice(input);

        Ok(Self {
            width,
            height,
            depth,
            top_mip_length,
            mip_map_count,
            bcn_encoding,
            raw_image_data,
        })
    }

    /// Every usable mip level, largest first, as `(width, height, block bytes)`.
    ///
    /// Level sizes are `CTextureManager::CalculateTextureSize`
    /// (`bbblibrary/lib_texture_manager_2.cpp:1976`): each level halves both dimensions, and
    /// each dimension clamps to a 4-pixel minimum for the block size. Level 0's length comes
    /// from `top_mip_length` rather than the formula, because that is the length LZO actually
    /// decompressed to and so is what keeps every subsequent offset aligned with the bytes.
    ///
    /// Enumeration stops at the first level whose dimensions fall below 4, and at the first
    /// level the data is too short for. Both matter: a handful of assets pack their sub-4×4
    /// levels unclamped (the 512×512 sky textures are 27 bytes short of a block-clamped
    /// chain), so a level's offset is only trustworthy while every level above it was full
    /// size. AGENTS.md §3.12.
    pub fn mip_levels(&self) -> Vec<(usize, usize, &[u8])> {
        let block_bytes = bcn_block_bytes(self.bcn_encoding) as usize;
        let mut levels = Vec::new();
        let mut offset = 0usize;

        for level in 0..self.mip_map_count.max(1) {
            let width = (self.width >> level).max(1);
            let height = (self.height >> level).max(1);
            if width < 4 || height < 4 {
                break;
            }

            let length = if level == 0 {
                self.top_mip_length
            } else {
                width.div_ceil(4) * height.div_ceil(4) * block_bytes
            };

            let Some(bytes) = self.raw_image_data.get(offset..offset + length) else {
                break;
            };
            levels.push((width, height, bytes));
            offset += length;
        }

        levels
    }

    pub fn get_top_mip_bcn_image(&self) -> Result<&[u8], TextureError> {
        self.raw_image_data
            .get(..self.top_mip_length)
            .ok_or(TextureError::TopMipCompressed)
    }

    pub fn get_top_mip_pixel_image(
        &self,
        format: TextureImageFormat,
    ) -> Result<Vec<u8>, TextureError> {
        use TextureError as E;

        let top_mip_compressed = self.get_top_mip_bcn_image()?;

        if top_mip_compressed.is_empty() {
            return Ok(Vec::new());
        }

        let top_mip = bcndecode::decode(
            top_mip_compressed,
            self.width,
            self.height,
            self.bcn_encoding,
            format.into(),
        )
        .map_err(BcnError::from)
        .map_err(E::TopMipBcnDecompress)?;

        Ok(top_mip)
    }
}

#[derive(Error, Display, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextureError {
    SmallLength,
    LargeLength,
    TopMipCompressed,
    TopMipLzoDecompress(LzoError),
    TopMipBcnDecompress(BcnError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TextureImageFormat {
    #[default]
    RGBA,
    BGRA,
    ARGB,
    ABGR,
    LUM,
}

impl From<BcnDecoderFormat> for TextureImageFormat {
    fn from(x: BcnDecoderFormat) -> Self {
        use BcnDecoderFormat as F;
        match x {
            F::RGBA => Self::RGBA,
            F::BGRA => Self::BGRA,
            F::ARGB => Self::ARGB,
            F::ABGR => Self::ABGR,
            F::LUM => Self::LUM,
        }
    }
}

impl From<TextureImageFormat> for BcnDecoderFormat {
    fn from(val: TextureImageFormat) -> Self {
        use BcnDecoderFormat as F;
        match val {
            TextureImageFormat::RGBA => F::RGBA,
            TextureImageFormat::BGRA => F::BGRA,
            TextureImageFormat::ARGB => F::ARGB,
            TextureImageFormat::ABGR => F::ABGR,
            TextureImageFormat::LUM => F::LUM,
        }
    }
}

#[derive(Error, Display, Debug, Copy, Clone, PartialEq, Eq)]
pub enum BcnError {
    ImageDecodingError,
    InvalidImageSize,
    FeatureNotImplemented,
    InvalidPixelFormat,
}

impl From<bcndecode::Error> for BcnError {
    fn from(x: bcndecode::Error) -> Self {
        use bcndecode::Error as F;
        match x {
            F::ImageDecodingError => Self::ImageDecodingError,
            F::InvalidImageSize => Self::InvalidImageSize,
            F::FeatureNotImplemented => Self::FeatureNotImplemented,
            F::InvalidPixelFormat => Self::InvalidPixelFormat,
        }
    }
}
