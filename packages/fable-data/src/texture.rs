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
    pub bcn_encoding: BcnEncoding,
    pub raw_image_data: Vec<u8>,
}

/// Map Fable's `dxt_compression` tag (from a texture asset's metadata) to a BCN encoding.
///
/// The tag is the D3D `D3DFORMAT` family with Fable-specific aliases: `1/31/33` are DXT1 (BC1),
/// `3/32/34` DXT3 (BC2), and `5/35` DXT5 (BC3). Returns `None` for tags we don't decode.
pub fn bcn_encoding_from_dxt(dxt: u16) -> Option<BcnEncoding> {
    match dxt {
        1 | 31 | 33 => Some(BcnEncoding::Bc1),
        3 | 32 | 34 => Some(BcnEncoding::Bc2),
        5 | 35 => Some(BcnEncoding::Bc3),
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

        // The rest of the input is image data. It could be in several different formats

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
            bcn_encoding,
            raw_image_data,
        })
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
