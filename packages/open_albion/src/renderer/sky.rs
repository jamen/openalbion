use std::any::type_name;
use wgpu::{
    CommandEncoder, Device, FragmentState, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, TextureFormat, TextureView, VertexState, include_wgsl,
};

pub struct SkyShader(ShaderModule);

impl SkyShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("sky.wgsl")))
    }
}

pub struct SkyPipelineLayout(PipelineLayout);

impl SkyPipelineLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        }))
    }
}

pub struct SkyPipeline(RenderPipeline);

impl SkyPipeline {
    pub fn new(
        device: &Device,
        layout: &SkyPipelineLayout,
        shader: &SkyShader,
        target_format: TextureFormat,
    ) -> Self {
        Self(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(type_name::<Self>()),
            layout: Some(&layout.0),
            vertex: VertexState {
                module: &shader.0,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target_format.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        }))
    }
}

pub struct SkyPass {
    pipeline: SkyPipeline,
}

impl SkyPass {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let shader = SkyShader::new(device);
        let layout = SkyPipelineLayout::new(device);
        let pipeline = SkyPipeline::new(device, &layout, &shader, surface_format);
        Self { pipeline }
    }

    pub fn pass(&mut self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
        let mut rpass = cmd.begin_render_pass(&RenderPassDescriptor {
            label: Some(type_name::<Self>()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.pipeline.0);
        rpass.draw(0..3, 0..1);
    }
}
