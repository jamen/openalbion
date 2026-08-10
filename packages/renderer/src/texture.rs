//! Uploading decoded [`TextureImage`]s to the GPU.
//!
//! Decoding is the caller's job — by the time bytes reach here they are already pixels or
//! BCN blocks, so upload cannot fail on a format it does not understand.

use crate::image::TextureImage;
use wgpu::{
    Device, Extent3d, Queue, SamplerDescriptor, TexelCopyBufferLayout, TextureDescriptor,
    TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

/// Upload `image` and its mip chain as a 2D texture, returning a view over every level. The
/// underlying `wgpu::Texture` is kept alive by the returned view.
pub fn upload_texture(
    device: &Device,
    queue: &Queue,
    label: &str,
    image: &TextureImage,
) -> TextureView {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some(label),
        size: Extent3d {
            width: image.width,
            height: image.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: image.levels.len().max(1) as u32,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: image.format.wgpu_format(),
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    write_levels(queue, &texture, image);

    // The default view spans every level, which is what a sampler with a mip filter needs.
    texture.create_view(&TextureViewDescriptor::default())
}

/// Write each of `image`'s mip levels into the matching level of `texture`.
pub(crate) fn write_levels(queue: &Queue, texture: &wgpu::Texture, image: &TextureImage) {
    for (level, data) in image.levels.iter().enumerate() {
        let (width, height) = image.level_size(level);
        let mut copy = texture.as_image_copy();
        copy.mip_level = level as u32;

        queue.write_texture(
            copy,
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.bytes_per_row(level)),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// `SetMaxAnisotropy`'s argument, as the game ships it.
///
/// `CEngine::AnisotropicFilteringLevel` is initialised to 2 (`fableengine/engine.cpp:2686`)
/// and pushed into the per-stage `D3DSAMP_MAXANISOTROPY` slot each frame (`:5724`);
/// `NGlobalConsole::ConsoleSetMaxAnisotropy` (`fablelib/global_console.cpp:1371`) writes its
/// argument into that same slot, which is what pins the field as the anisotropy *degree*
/// rather than a filter-mode enum. Retail's `~/Fable/user.ini` opens with
/// `SetMaxAnisotropy(4);`, so 4 is the shipped configuration. AGENTS.md §3.12.
pub const MAX_ANISOTROPY: u16 = 4;

/// A linear-filtered, clamp-to-edge sampler — for textures that are a *lookup*, where a UV
/// outside `0..1` is out of range rather than another tile.
///
/// No mip filter and no anisotropy: a lookup is indexed, not projected onto a surface, so
/// there is no minification to filter and level 0 is the only level these textures have.
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

/// A trilinear, anisotropic, wrapping sampler — what a tiled surface texture needs.
///
/// Wrapping is D3D9's default addressing: 501 of 1500 meshes sampled out of `graphics.big`
/// carry UVs outside `0..1`, so clamping is visibly wrong for a third of the mesh library.
///
/// `mipmap_filter` is `Linear` for two reasons that agree. `TEXTURE_MIPMAP_LINEAR` exists in
/// the engine's own enum (`_misc/e.hpp:770`), and wgpu requires all three filters to be
/// `Linear` when `anisotropy_clamp > 1` — so the constraint and the oracle land on the same
/// configuration. Which value the engine actually sets is behind the render-state cache
/// AGENTS.md §9 records as unreadable, so this is an inference, not a transcription.
pub fn repeat_sampler(device: &Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&SamplerDescriptor {
        label: Some(label),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: MAX_ANISOTROPY,
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        ..Default::default()
    })
}
