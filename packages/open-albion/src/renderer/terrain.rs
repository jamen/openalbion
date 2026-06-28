//! Renders a level's heightmap as a lit terrain mesh.
//!
//! This is a deliberately simple first pass: it builds one mesh from a parsed [`Lev`], shades it
//! with a single directional light, and draws it with depth testing. Ground-theme texturing,
//! multiple levels, and proper lighting are left for later.

use bytemuck::{Pod, Zeroable};
use fable_data::lev::Lev;
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoder,
    CompareFunction, DepthBiasState, DepthStencilState, Device, FragmentState, IndexFormat,
    MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue,
    FrontFace, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilState,
    TextureFormat, TextureView, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Conversion from stored heightmap float to world-space Z.
///
/// Derived from `CMap::LoadFromFile` in the decomp: the file cell's Height field is
/// multiplied by 2048.0 (`___real_40a0000000000000`) to produce the runtime
/// `CHeightMapCell::Height` in world units. The constant is the same one used as the
/// clipping ceiling in `CHeightMap::SetSizeZAt` (max world height ≈ 2048).
///
/// The .lev file stores heights as normalised floats (roughly 0.0–1.0); multiplying by
/// 2048 yields world-space Z, which matches Thing `PositionZ` values (e.g. LookoutPoint
/// heights ~0.013–0.027 → world Z 27–55, Things at Z 27–42).
pub const HEIGHT_SCALE: f32 = 2048.0;
/// World-space spacing between adjacent heightmap grid points.
///
/// One heightmap cell = one world unit horizontally. Evidence: `.tng` Thing PositionX/Y
/// are in [0, width] cell units (Blank.lev 128×128 cells, Things at X≈74.8, Y≈68.8).
const CELL_SIZE: f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TerrainVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl TerrainVertex {
    const ATTRIBS: [VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

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
}

/// Build a triangulated, per-vertex-lit mesh from a level's heightmap grid.
fn build_terrain_mesh(lev: &Lev) -> (Vec<TerrainVertex>, Vec<u32>) {
    let width = lev.header.width as usize + 1;
    let height = lev.header.height as usize + 1;
    let cells = &lev.heightmap_cells;

    let height_at = |col: usize, row: usize| -> f32 {
        cells
            .get(row * width + col)
            .map(|c| c.height * HEIGHT_SCALE)
            .unwrap_or(0.0)
    };

    let mut vertices = Vec::with_capacity(width * height);
    for row in 0..height {
        for col in 0..width {
            let y = height_at(col, row);

            // Central-difference normal from the surrounding heights.
            let left = height_at(col.saturating_sub(1), row);
            let right = height_at((col + 1).min(width - 1), row);
            let down = height_at(col, row.saturating_sub(1));
            let up = height_at(col, (row + 1).min(height - 1));
            let normal = normalize([-(right - left), 2.0 * CELL_SIZE, -(up - down)]);

            vertices.push(TerrainVertex {
                position: [col as f32 * CELL_SIZE, y, row as f32 * CELL_SIZE],
                normal,
            });
        }
    }

    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);
    for row in 0..height.saturating_sub(1) {
        for col in 0..width.saturating_sub(1) {
            let a = (row * width + col) as u32;
            let b = a + 1;
            let c = a + width as u32;
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
        [0.0, 1.0, 0.0]
    }
}

pub struct TerrainUniformBindGroupLayout(BindGroupLayout);

impl TerrainUniformBindGroupLayout {
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

pub struct TerrainShader(ShaderModule);

impl TerrainShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("terrain.wgsl")))
    }
}

pub struct TerrainPipelineLayout(PipelineLayout);

impl TerrainPipelineLayout {
    pub fn new(device: &Device, uniform_layout: &TerrainUniformBindGroupLayout) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0],
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
                // No culling for now — the heightmap is a single surface and the winding hasn't
                // been verified, so this guarantees it's visible from any angle.
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

/// The uploaded terrain mesh and its uniform buffer/bind group.
pub struct TerrainMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl TerrainMesh {
    pub fn new(device: &Device, uniform_layout: &TerrainUniformBindGroupLayout, lev: &Lev) -> Self {
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

        let uniforms = TerrainUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        };

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("terrain_uniform_bind_group"),
            layout: &uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            uniform_buffer,
            uniform_bind_group,
        }
    }

    fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        let uniforms = TerrainUniforms { view_proj };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

pub struct TerrainPass {
    uniform_layout: TerrainUniformBindGroupLayout,
    pipeline: TerrainPipeline,
    mesh: Option<TerrainMesh>,
}

impl TerrainPass {
    pub fn new(device: &Device, surface_format: TextureFormat, depth_format: TextureFormat) -> Self {
        let shader = TerrainShader::new(device);
        let uniform_layout = TerrainUniformBindGroupLayout::new(device);
        let layout = TerrainPipelineLayout::new(device, &uniform_layout);
        let pipeline = TerrainPipeline::new(device, &layout, &shader, surface_format, depth_format);

        Self {
            uniform_layout,
            pipeline,
            mesh: None,
        }
    }

    pub fn set_terrain(&mut self, device: &Device, lev: &Lev) {
        let min_height = lev.heightmap_cells.iter().map(|c| c.height).fold(f32::INFINITY, f32::min);
        let max_height = lev.heightmap_cells.iter().map(|c| c.height).fold(f32::NEG_INFINITY, f32::max);
        tracing::info!(
            "Terrain height range: raw [{:.2}, {:.2}], scaled [{:.2}, {:.2}]",
            min_height, max_height,
            min_height * HEIGHT_SCALE, max_height * HEIGHT_SCALE,
        );
        self.mesh = Some(TerrainMesh::new(device, &self.uniform_layout, lev));
    }

    pub fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        if let Some(mesh) = &self.mesh {
            mesh.update_uniforms(queue, view_proj);
        }
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        let Some(mesh) = &self.mesh else {
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
        rpass.set_bind_group(0, &mesh.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        rpass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}
