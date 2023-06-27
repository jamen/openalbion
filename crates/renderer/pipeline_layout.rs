use std::any::type_name;

use crate::bind_group_layout::{BindGroupLayouts, ColorBindGroupLayout};

pub struct PipelineLayouts {
    pub color: ColorPipelineLayout,
}

impl PipelineLayouts {
    pub fn new(device: &wgpu::Device, bg_layouts: &BindGroupLayouts) -> Self {
        Self {
            color: ColorPipelineLayout::new(&device, &bg_layouts.color),
        }
    }
}

pub struct ColorPipelineLayout(wgpu::PipelineLayout);

impl ColorPipelineLayout {
    pub fn new(device: &wgpu::Device, bind_group_layout: &ColorBindGroupLayout) -> Self {
        Self(
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(type_name::<ColorPipelineLayout>()),
                bind_group_layouts: &[bind_group_layout.as_ref()],
                push_constant_ranges: &[],
            }),
        )
    }
}

impl AsRef<wgpu::PipelineLayout> for ColorPipelineLayout {
    fn as_ref(&self) -> &wgpu::PipelineLayout {
        &self.0
    }
}
