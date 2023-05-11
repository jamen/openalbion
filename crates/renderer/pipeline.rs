use crate::bind_group::BindGroups;

pub struct Pipelines {}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        bind_groups: &BindGroups,
    ) -> Self {
        Self {}
    }
}
