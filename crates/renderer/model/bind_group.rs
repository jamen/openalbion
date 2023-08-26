use crate::bind_group_layout::{BindGroupLayouts, ColorBindGroupLayout};
use std::any::type_name;

pub struct BindGroups {
    pub color: ColorBindGroup,
}

impl BindGroups {
    pub fn new(device: &wgpu::Device, bg_layouts: &BindGroupLayouts) -> Self {
        Self {
            color: ColorBindGroup::new(device, &bg_layouts.color),
        }
    }
}

pub struct ColorBindGroup(wgpu::BindGroup);

impl ColorBindGroup {
    pub fn new(device: &wgpu::Device, layout: &ColorBindGroupLayout) -> Self {
        Self(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(type_name::<ColorBindGroup>()),
            layout: layout.as_ref(),
            entries: &[],
        }))
    }
}

impl AsRef<wgpu::BindGroup> for ColorBindGroup {
    fn as_ref(&self) -> &wgpu::BindGroup {
        &self.0
    }
}
