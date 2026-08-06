use bytemuck::{Pod, Zeroable};
use fable_data::lev::Lev;
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

use crate::files::Files;
use fable_data::big::ExtraMetadata;
use fable_data::texture::{Texture, TextureImageFormat, bcn_encoding_from_dxt};

// UNVERIFIED: neither value is sourced from the game. LEV stores height as a
// normalised f32; the real world-space scale and cell pitch come from the landscape
// map/patch code (engine_landscape*.cpp), not from us. AGENTS.md §9.
pub const HEIGHT_SCALE: f32 = 2048.0;
const CELL_SIZE: f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TerrainVertex {
    position: [f32; 3],
    normal: [f32; 3],
    theme_indices: [u8; 4],
    blend: [u8; 4],
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

fn build_terrain_mesh(lev: &Lev) -> (Vec<TerrainVertex>, Vec<u32>) {
    let w = lev.header.width as usize + 1;
    let h = lev.header.height as usize + 1;
    let cells = &lev.heightmap_cells;

    let height_at = |col: usize, row: usize| -> f32 {
        cells
            .get(row * w + col)
            .map(|c| c.height * HEIGHT_SCALE)
            .unwrap_or(0.0)
    };

    let mut vertices = Vec::with_capacity(w * h);
    for row in 0..h {
        for col in 0..w {
            let z = height_at(col, row);

            let left = height_at(col.saturating_sub(1), row);
            let right = height_at((col + 1).min(w - 1), row);
            let down = height_at(col, row.saturating_sub(1));
            let up = height_at(col, (row + 1).min(h - 1));
            // Z-up: gradient in X and Y, up is +Z.
            let normal = normalize([-(right - left), -(up - down), 2.0 * CELL_SIZE]);

            let cell = &cells[row * w + col];

            // `CliffU`/`CliffV` are per-vertex texture coordinates on
            // `CLandscapeLayerMesh::CVertex` (engine_landscape_layer_mesh.hpp:71), produced by
            // the mesh builder according to the layer's `MappingDirection` — not derived from
            // the height gradient. Left zero until the layer meshes land (AGENTS.md step 5.2).
            vertices.push(TerrainVertex {
                position: [col as f32 * CELL_SIZE, row as f32 * CELL_SIZE, z],
                normal,
                theme_indices: [
                    cell.ground_theme.0,
                    cell.ground_theme.1,
                    cell.ground_theme.2,
                    0,
                ],
                blend: [
                    cell.ground_theme_strength.0,
                    cell.ground_theme_strength.1,
                    0,
                    0,
                ],
            });
        }
    }

    let mut indices = Vec::with_capacity((w - 1) * (h - 1) * 6);
    for row in 0..h.saturating_sub(1) {
        for col in 0..w.saturating_sub(1) {
            let a = (row * w + col) as u32;
            let b = a + 1;
            let c = a + w as u32;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    (vertices, indices)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
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

    pub fn set_terrain(
        &mut self,
        device: &Device,
        queue: &Queue,
        files: &mut Files,
        lev: &Lev,
    ) {
        let bundle = files.resolve_terrain_themes(&lev.header.heightmap_palette);

        let (vertices, indices) = build_terrain_mesh(lev);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        let index_count = indices.len() as u32;

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

        let (_texture_array, texture_view, palette_buffer) = if bundle.texture_ids.is_empty() {
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
            let layer_count = bundle.texture_ids.len() as u32;
            let tex_size = 256u32;
            let texture_array = device.create_texture(&TextureDescriptor {
                label: Some("terrain_texture_array"),
                size: Extent3d {
                    width: tex_size,
                    height: tex_size,
                    depth_or_array_layers: layer_count,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            for (layer, &tex_id) in bundle.texture_ids.iter().enumerate() {
                tracing::debug!("Loading terrain texture layer {layer}: id={tex_id}");
                match files.read_texture_by_id(tex_id as u32) {
                    Ok((asset, data)) => {
                        let extras = match &asset.extras {
                            Some(ExtraMetadata::Texture(e)) => e,
                            _ => {
                                tracing::warn!("Terrain texture {tex_id} is not a texture asset");
                                continue;
                            }
                        };
                        let dxt = extras.dxt_compression;
                        let Some(encoding) = bcn_encoding_from_dxt(dxt) else {
                            tracing::warn!("Unsupported DXT format for texture {tex_id}: {dxt}");
                            continue;
                        };
                        let width = extras.width as u32;
                        let height = extras.height as u32;
                        let parsed = match Texture::parse(
                            &mut data.as_slice(),
                            width as usize,
                            height as usize,
                            extras.depth as usize,
                            extras.top_mip_map_size as usize,
                            encoding,
                        ) {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!("Parse texture {tex_id}: {e:?}");
                                continue;
                            }
                        };
                        let rgba = match parsed.get_top_mip_pixel_image(TextureImageFormat::RGBA) {
                            Ok(d) => d,
                            Err(e) => {
                                tracing::warn!("Decode texture {tex_id}: {e:?}");
                                continue;
                            }
                        };
                        let bytes_per_row = width * 4;

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
                            &rgba,
                            TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(bytes_per_row),
                                rows_per_image: Some(height),
                            },
                            Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load terrain texture id={} for layer {}: {e}",
                            tex_id,
                            layer
                        );
                    }
                }
            }

            let array_view = texture_array.create_view(&TextureViewDescriptor {
                label: Some("terrain_texture_array_view"),
                dimension: Some(TextureViewDimension::D2Array),
                ..Default::default()
            });

            let mut pal = PaletteMap { layers: [0u32; 256] };
            for (pal_idx, &layer) in bundle.palette_to_layer.iter().enumerate() {
                pal.layers[pal_idx] = layer as u32;
            }
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

        let min_height = lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::INFINITY, f32::min);
        let max_height = lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::NEG_INFINITY, f32::max);
        tracing::info!(
            "Terrain height range: raw [{:.2}, {:.2}], scaled [{:.2}, {:.2}]",
            min_height,
            max_height,
            min_height * HEIGHT_SCALE,
            max_height * HEIGHT_SCALE,
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
