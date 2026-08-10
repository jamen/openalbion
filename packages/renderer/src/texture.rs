//! Uploading decoded [`TextureImage`]s to the GPU.
//!
//! Decoding is the caller's job — by the time bytes reach here they are already pixels or
//! BCN blocks, so upload cannot fail on a format it does not understand.

use crate::image::TextureImage;
use wgpu::{
    Device, Extent3d, Queue, SamplerDescriptor, TexelCopyBufferLayout, TextureDescriptor,
    TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

/// Upload `image` as a single-mip 2D texture and return a view over it. The underlying
/// `wgpu::Texture` is kept alive by the returned view.
pub fn upload_texture(
    device: &Device,
    queue: &Queue,
    label: &str,
    image: &TextureImage,
) -> TextureView {
    let size = Extent3d {
        width: image.width,
        height: image.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: image.format.wgpu_format(),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        &image.data,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(image.bytes_per_row()),
            rows_per_image: None,
        },
        size,
    );

    texture.create_view(&TextureViewDescriptor::default())
}

/// A linear-filtered, clamp-to-edge sampler — for textures that are a *lookup*, where a UV
/// outside `0..1` is out of range rather than another tile.
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

/// A linear-filtered, wrapping sampler — D3D9's default addressing, and what any tiled
/// texture needs. 501 of 1500 meshes sampled out of `graphics.big` carry UVs outside
/// `0..1`, so clamping is visibly wrong for a third of the mesh library.
pub fn repeat_sampler(device: &Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some(label),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        ..Default::default()
    })
}
