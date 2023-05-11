use crate::{buffer::Buffers, texture::Textures};

pub struct BindGroups {}

impl BindGroups {
    pub fn new(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        surface_config: &wgpu::SurfaceConfiguration,
        buffers: &Buffers,
        textures: &Textures,
    ) -> Self {
        Self {}
    }
}
