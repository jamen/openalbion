use bytemuck::{Pod, Zeroable};
use fable_data::{
    big::{AssetMetadata, ExtraMetadata},
    texture::{BcnEncoding, Texture, TextureError},
};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, Device, Extent3d, FragmentState, IndexFormat, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages,
    TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SkyVertex {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}

impl SkyVertex {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct SkyUniforms {
    view_proj: [[f32; 4]; 4],
}

fn generate_skydome_mesh(segments: u32, rings: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let horizon_color = [1.0, 0.7, 0.5, 0.3];
    let zenith_color = [0.8, 0.85, 1.0, 0.0];

    for ring in 0..=rings {
        let elevation = (ring as f32 / rings as f32) * std::f32::consts::FRAC_PI_2;
        let y = elevation.sin();
        let xz_radius = elevation.cos();
        let v = ring as f32 / rings as f32;
        let t = (v * 2.0).min(1.0);
        let t = t * t;
        let color = [
            horizon_color[0] * (1.0 - t) + zenith_color[0] * t,
            horizon_color[1] * (1.0 - t) + zenith_color[1] * t,
            horizon_color[2] * (1.0 - t) + zenith_color[2] * t,
            horizon_color[3] * (1.0 - t) + zenith_color[3] * t,
        ];

        for seg in 0..=segments {
            let azimuth = (seg as f32 / segments as f32) * std::f32::consts::TAU;
            let x = xz_radius * azimuth.cos();
            let z = xz_radius * azimuth.sin();
            let u = seg as f32 / segments as f32;

            vertices.push(SkyVertex {
                position: [x, y, z],
                color,
                uv: [u, v],
            });
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let current = ring * (segments + 1) + seg;
            let next = current + segments + 1;

            indices.push(current as u16);
            indices.push((current + 1) as u16);
            indices.push(next as u16);

            indices.push((current + 1) as u16);
            indices.push((next + 1) as u16);
            indices.push(next as u16);
        }
    }

    (vertices, indices)
}

pub struct SkyDome {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl SkyDome {
    pub fn new(device: &Device, uniform_layout: &SkyUniformBindGroupLayout) -> Self {
        let (vertices, indices) = generate_skydome_mesh(32, 16);
        let index_count = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let uniforms = SkyUniforms {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sky_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sky_uniform_bind_group"),
            layout: &uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    pub fn update_view_projection(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        let uniforms = SkyUniforms { view_proj };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

pub struct SkyTexture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: BindGroup,
}

impl SkyTexture {
    pub fn new(
        device: &Device,
        queue: &Queue,
        layout: &SkyTextureBindGroupLayout,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<Self, SkyTextureError> {
        use SkyTextureError as E;

        let texture_extras = match &asset_info.extras {
            Some(ExtraMetadata::Texture(extras)) => extras,
            _ => return Err(E::NotATexture),
        };

        tracing::debug!(
            "Texture: {}x{}x{}, dxt_compression={}, top_mip_size={}, top_mip_compressed_size={}, mip_maps={}, asset_data_len={}",
            texture_extras.width,
            texture_extras.height,
            texture_extras.depth,
            texture_extras.dxt_compression,
            texture_extras.top_mip_map_size,
            texture_extras.top_mip_map_compressed_size,
            texture_extras.mip_maps,
            asset_data.len(),
        );

        tracing::debug!(
            "Asset data header: {:02x?}",
            &asset_data[..asset_data.len().min(32)]
        );

        let width = texture_extras.width as u32;
        let height = texture_extras.height as u32;

        let format = match texture_extras.dxt_compression {
            1 | 33 => TextureFormat::Bc1RgbaUnorm,
            3 | 34 => TextureFormat::Bc2RgbaUnorm,
            5 | 35 => TextureFormat::Bc3RgbaUnorm,
            other => return Err(E::UnsupportedDxtFormat(other)),
        };

        let mut input = asset_data;
        let texture_data = Texture::parse(
            &mut input,
            width as usize,
            height as usize,
            texture_extras.depth as usize,
            texture_extras.top_mip_map_size as usize,
            dxt_to_bcn_encoding(texture_extras.dxt_compression),
        )
        .map_err(E::Parse)?;

        let bcn_data = texture_data.get_top_mip_bcn_image().map_err(E::Parse)?;

        tracing::info!(
            "Sky texture loaded: {}x{}, format={:?}, bcn_data_len={}, expected_bcn_len={}",
            width,
            height,
            format,
            bcn_data.len(),
            // Expected size for BCn: (width/4) * (height/4) * block_size
            ((width + 3) / 4)
                * ((height + 3) / 4)
                * match format {
                    TextureFormat::Bc1RgbaUnorm => 8,
                    _ => 16,
                }
        );

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&asset_info.symbol_name),
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

        let block_size = match format {
            TextureFormat::Bc1RgbaUnorm => 8,
            TextureFormat::Bc2RgbaUnorm | TextureFormat::Bc3RgbaUnorm => 16,
            _ => unreachable!(),
        };
        let blocks_wide = (width + 3) / 4;
        let bytes_per_row = blocks_wide * block_size;

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

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("sky_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("sky_bind_group"),
            layout: &layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
        })
    }
}

fn dxt_to_bcn_encoding(dxt: u16) -> BcnEncoding {
    match dxt {
        1 | 33 => BcnEncoding::Bc1,
        3 | 34 => BcnEncoding::Bc2,
        5 | 35 => BcnEncoding::Bc3,
        _ => BcnEncoding::Bc1,
    }
}

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum SkyTextureError {
    NotATexture,
    #[display("UnsupportedDxtFormat({_0})")]
    UnsupportedDxtFormat(#[error(not(source))] u16),
    Parse(TextureError),
}

pub struct SkyUniformBindGroupLayout(BindGroupLayout);

impl SkyUniformBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }))
    }
}

pub struct SkyTextureBindGroupLayout(BindGroupLayout);

impl SkyTextureBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }))
    }
}

pub struct SkyShader(ShaderModule);

impl SkyShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("sky.wgsl")))
    }
}

pub struct SkyPipelineLayout(PipelineLayout);

impl SkyPipelineLayout {
    pub fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        texture_layout: &SkyTextureBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, &texture_layout.0],
            immediate_size: 0,
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
                buffers: &[SkyVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target_format.into())],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        }))
    }
}

pub struct SkyPass {
    uniform_layout: SkyUniformBindGroupLayout,
    texture_layout: SkyTextureBindGroupLayout,
    pipeline: SkyPipeline,
    dome: SkyDome,
    texture: Option<SkyTexture>,
}

impl SkyPass {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        let shader = SkyShader::new(device);
        let uniform_layout = SkyUniformBindGroupLayout::new(device);
        let texture_layout = SkyTextureBindGroupLayout::new(device);
        let layout = SkyPipelineLayout::new(device, &uniform_layout, &texture_layout);
        let pipeline = SkyPipeline::new(device, &layout, &shader, surface_format);
        let dome = SkyDome::new(device, &uniform_layout);

        Self {
            uniform_layout,
            texture_layout,
            pipeline,
            dome,
            texture: None,
        }
    }

    pub fn set_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), SkyTextureError> {
        let texture = SkyTexture::new(device, queue, &self.texture_layout, asset_info, asset_data)?;
        self.texture = Some(texture);
        Ok(())
    }

    pub fn update_camera(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        self.dome.update_view_projection(queue, view_proj);
    }

    pub fn pass(&self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
        let Some(texture) = &self.texture else {
            return;
        };

        let mut rpass = cmd.begin_render_pass(&RenderPassDescriptor {
            label: Some(type_name::<Self>()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_texture_view,
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
            multiview_mask: None,
        });
        rpass.set_pipeline(&self.pipeline.0);
        rpass.set_bind_group(0, &self.dome.uniform_bind_group, &[]);
        rpass.set_bind_group(1, &texture.bind_group, &[]);
        rpass.set_vertex_buffer(0, self.dome.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.dome.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..self.dome.index_count, 0, 0..1);
    }
}
