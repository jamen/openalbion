//! The landscape pass.
//!
//! Takes a prebuilt [`TerrainData`] — vertices, indices and one decoded image per theme
//! layer. Building that from a `.lev` heightmap is a data transformation and lives on the
//! other side of the crate boundary, which is also where the real
//! `CEngineLandscapeMeshBuilder` port belongs (AGENTS.md §3.4, step 5.2).

use crate::image::TextureImage;
use bytemuck::{Pod, Zeroable};
use std::any::type_name;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device, Extent3d,
    FilterMode, FragmentState, FrontFace, IndexFormat, MultisampleState, Origin3d, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages, StencilState,
    TexelCopyTextureInfo, TexelCopyBufferLayout, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexState,
    VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Everything the pass needs to draw a landscape, with no asset formats behind it.
pub struct TerrainData {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    /// One image per theme layer. All layers must share a size — the pass uploads them
    /// into a single array texture and skips any that disagree.
    pub layers: Vec<TextureImage>,
    /// LEV heightmap-palette slot → index into `layers`.
    pub palette_to_layer: [u32; 256],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TerrainVertex {
    /// World position, Z-up (AGENTS.md §3.6).
    pub position: [f32; 3],
    pub normal: [f32; 3],
    /// Palette slots of this vertex's ground themes, resolved through
    /// [`TerrainData::palette_to_layer`] in the shader.
    pub theme_indices: [u8; 4],
    pub blend: [u8; 4],
}

impl TerrainVertex {
    const ATTRIBS: [VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Uint8x4,
        3 => Uint8x4,
    ];

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
struct TerrainUniforms {
    view_proj: [[f32; 4]; 4],
    texture_scale: f32,
    _pad: [f32; 7],           // matches WGSL uniform layout (96 bytes total)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct PaletteMap {
    layers: [u32; 256],
}

pub struct TerrainBindGroupLayout(BindGroupLayout);

impl TerrainBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }))
    }
}

pub struct TerrainShader(ShaderModule);

impl TerrainShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("terrain.wgsl")))
    }
}

pub struct TerrainPipelineLayout(PipelineLayout);

impl TerrainPipelineLayout {
    pub fn new(device: &Device, bind_layout: &TerrainBindGroupLayout) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&bind_layout.0],
            immediate_size: 0,
        }))
    }
}

pub struct TerrainPipeline(RenderPipeline);

impl TerrainPipeline {
    pub fn new(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        Self(device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(type_name::<Self>()),
            layout: Some(&layout.0),
            vertex: VertexState {
                module: &shader.0,
                entry_point: Some("vs_main"),
                buffers: &[TerrainVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(target_format.into())],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        }))
    }
}

pub struct TerrainPass {
    bind_layout: TerrainBindGroupLayout,
    pipeline: TerrainPipeline,
    sampler: wgpu::Sampler,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    index_count: u32,
    uniform_buffer: Option<wgpu::Buffer>,
    bind_group: Option<BindGroup>,

    _texture_array: Option<wgpu::Texture>,
    _palette_buffer: Option<wgpu::Buffer>,
}

impl TerrainPass {
    pub fn new(
        device: &Device,
        surface_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        let shader = TerrainShader::new(device);
        let bind_layout = TerrainBindGroupLayout::new(device);
        let layout = TerrainPipelineLayout::new(device, &bind_layout);
        let pipeline = TerrainPipeline::new(device, &layout, &shader, surface_format, depth_format);
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("terrain_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..Default::default()
        });

        Self {
            bind_layout,
            pipeline,
            sampler,
            vertex_buffer: None,
            index_buffer: None,
            index_count: 0,
            uniform_buffer: None,
            bind_group: None,
            _texture_array: None,
            _palette_buffer: None,
        }
    }

    pub fn set_terrain(&mut self, device: &Device, queue: &Queue, terrain: &TerrainData) {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&terrain.vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_index_buffer"),
            contents: bytemuck::cast_slice(&terrain.indices),
            usage: BufferUsages::INDEX,
        });
        let index_count = terrain.indices.len() as u32;

        let uniforms = TerrainUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            texture_scale: 0.0625,
            _pad: [0.0; 7],
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let (_texture_array, texture_view, palette_buffer) = if terrain.layers.is_empty() {
            let tex = device.create_texture(&TextureDescriptor {
                label: Some("terrain_texture_array_empty"),
                size: Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let magenta = vec![255u8, 0, 255, 255].repeat(16);
            queue.write_texture(
                tex.as_image_copy(),
                &magenta,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(16),
                    rows_per_image: Some(4),
                },
                Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
            );
            let view = tex.create_view(&TextureViewDescriptor {
                label: Some("terrain_texture_array_view"),
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });
            let pal = PaletteMap { layers: [0u32; 256] };
            let pal_buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("terrain_palette_map"),
                contents: bytemuck::cast_slice(&[pal]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });
            (Some(tex), view, Some(pal_buf))
        } else {
            // The array's extent and format come from the first layer; every layer has to
            // agree with it. The caller decodes all layers to a common size, so a mismatch
            // here is a data bug worth surfacing rather than silently rescaling.
            let first = &terrain.layers[0];
            let texture_array = device.create_texture(&TextureDescriptor {
                label: Some("terrain_texture_array"),
                size: Extent3d {
                    width: first.width,
                    height: first.height,
                    depth_or_array_layers: terrain.layers.len() as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: first.format.wgpu_format(),
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            for (layer, image) in terrain.layers.iter().enumerate() {
                if image.width != first.width
                    || image.height != first.height
                    || image.format != first.format
                {
                    tracing::warn!(
                        "Terrain layer {layer} is {}x{} {:?}, expected {}x{} {:?} — skipped",
                        image.width,
                        image.height,
                        image.format,
                        first.width,
                        first.height,
                        first.format,
                    );
                    continue;
                }
                if !image.is_complete() {
                    tracing::warn!(
                        "Terrain layer {layer} has {} bytes, too few for {}x{} {:?} — skipped",
                        image.data.len(),
                        image.width,
                        image.height,
                        image.format,
                    );
                    continue;
                }

                queue.write_texture(
                    TexelCopyTextureInfo {
                        texture: &texture_array,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: layer as u32,
                        },
                        aspect: TextureAspect::All,
                    },
                    &image.data,
                    TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(image.bytes_per_row()),
                        rows_per_image: Some(image.height),
                    },
                    Extent3d {
                        width: image.width,
                        height: image.height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            let array_view = texture_array.create_view(&TextureViewDescriptor {
                label: Some("terrain_texture_array_view"),
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });

            let pal = PaletteMap {
                layers: terrain.palette_to_layer,
            };
            let palette_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("terrain_palette_map"),
                contents: bytemuck::cast_slice(&[pal]),
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            });

            (Some(texture_array), array_view, Some(palette_buffer))
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("terrain_bind_group"),
            layout: &self.bind_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: palette_buffer
                        .as_ref()
                        .map(|b| b.as_entire_binding())
                        .unwrap_or(uniform_buffer.as_entire_binding()),
                },
            ],
        });

        tracing::info!(
            "Terrain uploaded: {} vertices, {index_count} indices, {} texture layers",
            terrain.vertices.len(),
            terrain.layers.len(),
        );

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.index_count = index_count;
        self.uniform_buffer = Some(uniform_buffer);
        self.bind_group = Some(bind_group);
    }

    pub fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        let Some(uniform_buffer) = &self.uniform_buffer else {
            return;
        };
        let uniforms = TerrainUniforms {
            view_proj,
            texture_scale: 0.0625,
            _pad: [0.0; 7],
        };
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        let (Some(vertex_buffer), Some(index_buffer), Some(bind_group)) = (
            &self.vertex_buffer,
            &self.index_buffer,
            &self.bind_group,
        ) else {
            return;
        };

        let mut rpass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.pipeline.0);
        rpass.set_bind_group(0, bind_group, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
