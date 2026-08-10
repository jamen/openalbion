//! Fable texture assets → [`TextureImage`].

use renderer::{ImageFormat, TextureImage};
use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, ExtraMetadata},
    texture::{BcnEncoding, Texture, TextureError, TextureImageFormat, bcn_encoding_from_dxt},
};

#[derive(Debug, Display, Error)]
pub enum TextureDecodeError {
    #[display("asset is not a texture")]
    NotATexture,
    #[display("unsupported DXT format {_0}")]
    UnsupportedDxtFormat(#[error(not(source))] u16),
    #[display("volume texture ({_0} slices) — the 2D upload path cannot take it")]
    VolumeTexture(#[error(not(source))] u16),
    #[display("texture parse error: {_0}")]
    Parse(TextureError),
}

/// Decode an asset's top mip, keeping its block compression.
///
/// The GPU samples BCN natively, so this is the cheaper path and the one to prefer.
pub fn decode_texture(
    asset: &AssetMetadata,
    data: &[u8],
) -> Result<TextureImage, TextureDecodeError> {
    let (parsed, width, height, format) = parse(asset, data)?;
    let blocks = parsed
        .get_top_mip_bcn_image()
        .map_err(TextureDecodeError::Parse)?;

    Ok(TextureImage {
        width,
        height,
        format,
        data: blocks.to_vec(),
    })
}

/// Decode an asset's top mip all the way to RGBA8.
///
/// Needed when several assets have to share one GPU texture — the terrain layer array
/// mixes DXT1 and DXT5 sources, and an array texture has a single format.
pub fn decode_texture_rgba(
    asset: &AssetMetadata,
    data: &[u8],
) -> Result<TextureImage, TextureDecodeError> {
    let (parsed, width, height, _) = parse(asset, data)?;
    let rgba = parsed
        .get_top_mip_pixel_image(TextureImageFormat::RGBA)
        .map_err(TextureDecodeError::Parse)?;

    Ok(TextureImage {
        width,
        height,
        format: ImageFormat::Rgba8,
        data: rgba,
    })
}

fn parse(
    asset: &AssetMetadata,
    data: &[u8],
) -> Result<(Texture, u32, u32, ImageFormat), TextureDecodeError> {
    use TextureDecodeError as E;

    let extras = match &asset.extras {
        Some(ExtraMetadata::Texture(extras)) => extras,
        _ => return Err(E::NotATexture),
    };

    // `depth > 1` is a volume texture, whose `top_mip_map_size` covers every slice — taking
    // width/height as the whole asset would upload slice 0's worth of a stack. Two exist
    // (`WEATHER_RAIN`, `MIST_ALPHA`) and nothing draws them; see `texture_test.rs`.
    if extras.depth > 1 {
        return Err(E::VolumeTexture(extras.depth));
    }

    let dxt = extras.dxt_compression;
    let encoding = bcn_encoding_from_dxt(dxt).ok_or(E::UnsupportedDxtFormat(dxt))?;
    let format = image_format(encoding).ok_or(E::UnsupportedDxtFormat(dxt))?;

    let width = extras.width as u32;
    let height = extras.height as u32;

    let mut input = data;
    let parsed = Texture::parse(
        &mut input,
        width as usize,
        height as usize,
        extras.depth as usize,
        extras.top_mip_map_size as usize,
        encoding,
    )
    .map_err(E::Parse)?;

    Ok((parsed, width, height, format))
}

/// The renderer's format for a BCN encoding. Only BC1/2/3 — the encodings
/// [`bcn_encoding_from_dxt`] yields for Fable assets — have a mapping.
fn image_format(encoding: BcnEncoding) -> Option<ImageFormat> {
    match encoding {
        BcnEncoding::Bc1 => Some(ImageFormat::Bc1),
        BcnEncoding::Bc2 => Some(ImageFormat::Bc2),
        BcnEncoding::Bc3 => Some(ImageFormat::Bc3),
        _ => None,
    }
}
