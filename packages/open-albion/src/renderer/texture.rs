//! Shared helpers for uploading Fable's BCN-compressed texture assets to the GPU.
//!
//! The model and sky passes both take a texture asset (metadata + raw bytes), decode it with
//! [`fable_data::texture::Texture`], and upload the top mip as a block-compressed `wgpu::Texture`.
//! That logic lives here once so the passes only differ in how they bind the result.

use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, ExtraMetadata},
    texture::{BcnEncoding, Texture, TextureError, bcn_block_bytes, bcn_encoding_from_dxt},
};
use wgpu::{
    Device, Extent3d, Queue, SamplerDescriptor, TexelCopyBufferLayout, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

#[derive(Debug, Display, Error)]
pub enum TextureUploadError {
    #[display("asset is not a texture")]
    NotATexture,
    #[display("unsupported DXT format {_0}")]
    UnsupportedDxtFormat(#[error(not(source))] u16),
    #[display("texture parse error: {_0}")]
    Parse(TextureError),
}

/// wgpu's block-compressed format corresponding to a BCN encoding. Only the BC1/2/3 encodings that
/// [`bcn_encoding_from_dxt`] produces are mapped; other modes are unreachable for Fable assets.
fn bcn_texture_format(encoding: BcnEncoding) -> TextureFormat {
    match encoding {
        BcnEncoding::Bc1 => TextureFormat::Bc1RgbaUnorm,
        BcnEncoding::Bc2 => TextureFormat::Bc2RgbaUnorm,
        BcnEncoding::Bc3 => TextureFormat::Bc3RgbaUnorm,
        _ => unreachable!("bcn_encoding_from_dxt only yields BC1/BC2/BC3"),
    }
}

/// Decode a Fable texture asset and upload its top mip as a block-compressed `wgpu::Texture`,
/// returning a view over it. The underlying `wgpu::Texture` is kept alive by the returned view.
pub fn upload_texture(
    device: &Device,
    queue: &Queue,
    asset: &AssetMetadata,
    data: &[u8],
) -> Result<TextureView, TextureUploadError> {
    use TextureUploadError as E;

    let extras = match &asset.extras {
        Some(ExtraMetadata::Texture(extras)) => extras,
        _ => return Err(E::NotATexture),
    };

    let dxt = extras.dxt_compression;
    let encoding = bcn_encoding_from_dxt(dxt).ok_or(E::UnsupportedDxtFormat(dxt))?;
    let format = bcn_texture_format(encoding);

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

    let bcn_data = parsed.get_top_mip_bcn_image().map_err(E::Parse)?;
    let bytes_per_row = width.div_ceil(4) * bcn_block_bytes(encoding);

    let texture = device.create_texture(&TextureDescriptor {
        label: Some(&asset.symbol_name),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        bcn_data,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: None,
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    Ok(texture.create_view(&TextureViewDescriptor::default()))
}

/// A linear-filtered, clamp-to-edge sampler — the common case for both texture passes.
pub fn linear_clamp_sampler(device: &Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some(label),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    })
}
