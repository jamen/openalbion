use std::any::type_name;

use crate::{
    pipeline_layout::{ColorPipelineLayout, PipelineLayouts},
    shader::ColorShader,
};

pub struct Pipelines {
    pub color: ColorPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        layouts: &PipelineLayouts,
    ) -> Self {
        Self {
            color: ColorPipeline::new(device, surface_config, &layouts.color),
        }
    }
}

pub struct ColorPipeline(wgpu::RenderPipeline);

impl ColorPipeline {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        layout: &ColorPipelineLayout,
    ) -> Self {
        let shader = ColorShader::new(device);
        let module = shader.as_ref();

        Self(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(type_name::<ColorPipeline>()),
                layout: Some(layout.as_ref()),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: "fs_main",
                    targets: &[Some(surface_config.format.into())],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
            }),
        )
    }
}

impl AsRef<wgpu::RenderPipeline> for ColorPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.0
    }
}
