//! Depth buffer shared by the world (terrain/model) passes.

use std::any::type_name;
use wgpu::{
    Device, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

pub struct DepthTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: TextureView,
}

impl DepthTexture {
    pub const FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(device: &Device, size: [u32; 2]) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(type_name::<Self>()),
            size: Extent3d {
                width: size[0].max(1),
                height: size[1].max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self { texture, view }
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }
}
