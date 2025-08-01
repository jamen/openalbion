use std::fmt;

use crate::bytes::{take, take_bytes};
use bcndecode::BcnDecoderFormat;
use derive_more::derive::{Display, Error};

pub use bcndecode::BcnEncoding;
pub use minilzo::LzoError;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub top_mip_length: usize,
    pub bcn_encoding: BcnEncoding,
    pub raw_image_data: Vec<u8>,
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

        let small_length = take::<u16>(input).map_err(|_| E::SmallLength)?.to_le();

        let top_mip_compressed_length = if small_length == 0xffff {
            take::<u32>(input).map_err(|_| E::LargeLength)?.to_le()
        } else {
            small_length as u32
        };

        let mut raw_image_data = Vec::new();

        if top_mip_compressed_length > 0 {
            let top_mip_compressed = take_bytes(input, top_mip_compressed_length as usize)
                .map_err(|_| E::TopMipCompressed)?;

            let top_mip = minilzo::decompress(top_mip_compressed, top_mip_length)
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
            .ok_or_else(|| TextureError::TopMipCompressed)
    }

    pub fn get_top_mip_pixel_image(
        &self,
        format: TextureImageFormat,
    ) -> Result<Vec<u8>, TextureError> {
        use TextureError as E;

        let top_mip_compressed = self.get_top_mip_bcn_image()?;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextureImageFormat {
    RGBA,
    BGRA,
    ARGB,
    ABGR,
    LUM,
}

impl Default for TextureImageFormat {
    fn default() -> Self {
        Self::RGBA
    }
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

impl Into<BcnDecoderFormat> for TextureImageFormat {
    fn into(self) -> BcnDecoderFormat {
        use BcnDecoderFormat as F;
        match self {
            Self::RGBA => F::RGBA,
            Self::BGRA => F::BGRA,
            Self::ARGB => F::ARGB,
            Self::ABGR => F::ABGR,
            Self::LUM => F::LUM,
        }
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
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

impl fmt::Display for BcnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::ImageDecodingError => "image decoding error",
            Self::InvalidImageSize => "invalid image size",
            Self::FeatureNotImplemented => "feature not implemented",
            Self::InvalidPixelFormat => "invalid pixel format",
        })
    }
}
