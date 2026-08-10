//! The landscape foreground pass.
//!
//! The original draws the landscape as N alpha-blended passes per patch, one per theme
//! layer: the layer's ground texture supplies colour through a planar projection along the
//! layer's mapping direction, and a blend table indexed by the vertex normal supplies the
//! alpha (AGENTS.md §3.4). [`TerrainData`] arrives with that already resolved — one
//! [`TerrainDraw`] per distinct (texture, mapping direction), over a shared vertex and index
//! buffer. Building it from a `.lev` lives on the other side of the crate boundary.

use crate::image::TextureImage;
use bytemuck::{Pod, Zeroable};
use std::any::type_name;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    BufferUsages, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device,
    Extent3d, FilterMode, FragmentState, FrontFace, IndexFormat, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages, StencilState,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Everything the pass needs to draw a landscape, with no asset formats behind it.
pub struct TerrainData {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    /// One per distinct (ground texture, mapping direction), in draw order.
    pub draws: Vec<TerrainDraw>,
    /// Ground textures, referenced by [`TerrainDraw::texture`].
    pub textures: Vec<TextureImage>,
    /// The five mapping directions' blend tables, referenced by
    /// [`TerrainDraw::blend_table`]. Single-channel; the original's are `A8`.
    pub blend_tables: Vec<TextureImage>,
}

/// One alpha-blended layer pass.
pub struct TerrainDraw {
    pub first_index: u32,
    pub index_count: u32,
    /// Index into [`TerrainData::textures`].
    pub texture: u32,
    /// Index into [`TerrainData::blend_tables`] — the layer's mapping direction.
    pub blend_table: u32,
    /// `c40` / `c41`: the planar projection for this layer's mapping direction. `w` is the
    /// per-patch UV localisation offset, which is always a whole number of texture tiles and
    /// so samples identically at zero (see `tools/landscape-statics.md`).
    pub uv_transform_u: [f32; 4],
    pub uv_transform_v: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TerrainVertex {
    /// World position, Z-up (AGENTS.md §3.6).
    pub position: [f32; 3],
    /// Lighting normal, from `CEngineMap::PeekMapNormal`.
    pub normal: [f32; 3],
    /// This layer's theme weight here, `0..=1`.
    pub blend: f32,
    /// The blend table lookup — the vertex normal's packed horizontal part, *not* a ground
    /// texture coordinate.
    pub cliff_uv: [f32; 2],
}

impl TerrainVertex {
    const ATTRIBS: [VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32,
        3 => Float32x2,
    ];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// The per-frame shader constants, by their register names in the Lights layout (§3.8).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct FrameUniforms {
    /// `c5..c8`
    view_proj: [[f32; 4]; 4],
    /// `c4`
    camera_pos: [f32; 4],
    /// `c3`
    ambient: [f32; 4],
    /// `c19`
    light_dir: [f32; 4],
    /// `c20`
    diffuse: [f32; 4],
    /// `c35`
    backlight: [f32; 4],
    /// `c42`
    fade_transform: [f32; 4],
}

/// The lighting the pass runs with until the environment layer lands (AGENTS.md step 2).
///
/// UNVERIFIED, and deliberately inert rather than plausible: `Ambient` of 0.5 cancels the
/// pixel shader's `mul_x2` exactly, so the landscape shows its textures at their authored
/// colour with no directional term at all. The mechanism is fully wired — when step 2
/// supplies rows 1/0/3 of the environment LUT, only these four values change.
impl FrameUniforms {
    fn new(view_proj: [[f32; 4]; 4], camera_pos: [f32; 3]) -> FrameUniforms {
        FrameUniforms {
            view_proj,
            camera_pos: [camera_pos[0], camera_pos[1], camera_pos[2], 0.0],
            // UNVERIFIED: neutral stand-in — see above.
            ambient: [0.5, 0.5, 0.5, 1.0],
            light_dir: [0.0, 0.0, -1.0, 0.0],
            diffuse: [0.0, 0.0, 0.0, 0.0],
            backlight: [0.0, 0.0, 0.0, 0.0],
            // Fade disabled, expressed in the real mechanism rather than bypassed:
            // `dot(dist, 0) + 1` saturates to 1 at every distance.
            // `ForegroundFadeStart`/`End` are set by a console command whose defaults are
            // not recoverable from the decomp (AGENTS.md §9).
            fade_transform: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// `c40` / `c41` for one draw.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct DrawUniforms {
    uv_transform_u: [f32; 4],
    uv_transform_v: [f32; 4],
}

pub struct TerrainBindGroupLayouts {
    frame: BindGroupLayout,
    draw: BindGroupLayout,
}

impl TerrainBindGroupLayouts {
    pub fn new(device: &Device) -> Self {
        let uniform = |binding, visibility| BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture = |binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler = |binding| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };

        Self {
            frame: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("terrain_frame"),
                entries: &[uniform(
                    0,
                    ShaderStages::VERTEX_FRAGMENT,
                )],
            }),
            draw: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("terrain_draw"),
                entries: &[
                    uniform(0, ShaderStages::VERTEX),
                    texture(1),
                    sampler(2),
                    texture(3),
                    sampler(4),
                ],
            }),
        }
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
    pub fn new(device: &Device, layouts: &TerrainBindGroupLayouts) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&layouts.frame, &layouts.draw],
            immediate_size: 0,
        }))
    }
}

pub struct TerrainPipeline(RenderPipeline);

impl TerrainPipeline {
    /// The layer pass: additive over the blackout pass, depth-tested but not depth-writing.
    ///
    /// Additive is forced by the alphas: they sum to exactly 1 at every point, so
    /// `sum(alpha_i * colour_i)` over black is the weighted average the data describes.
    /// Source-alpha-over would give `1 - prod(1 - alpha_i)` and leak the background between
    /// themes. The `D3DRS_*` values themselves are not recoverable from the decomp -- see
    /// `SetupForegroundStates` -- so this is derived from the blend maths rather than
    /// transcribed (AGENTS.md §9).
    pub fn new(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        Self(Self::build(
            device,
            layout,
            shader,
            target_format,
            depth_format,
            "vs_main",
            "fs_main",
            Some(BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            // Depth was written by the blackout pass over the same geometry, so every layer
            // sits exactly on it.
            false,
            CompareFunction::LessEqual,
        ))
    }

    /// `VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS`: the same meshes in solid black, opaque,
    /// establishing depth and an origin for the additive layers.
    pub fn new_blackout(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        Self(Self::build(
            device,
            layout,
            shader,
            target_format,
            depth_format,
            "vs_blackout",
            "fs_blackout",
            None,
            true,
            CompareFunction::Less,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
        vs: &str,
        fs: &str,
        blend: Option<BlendState>,
        depth_write_enabled: bool,
        depth_compare: CompareFunction,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(vs),
            layout: Some(&layout.0),
            vertex: VertexState {
                module: &shader.0,
                entry_point: Some(vs),
                buffers: &[TerrainVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.0,
                entry_point: Some(fs),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: depth_format,
                depth_write_enabled,
                depth_compare,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    }
}

/// A draw's bind group, holding its own `c40`/`c41` and its two textures.
struct PreparedDraw {
    bind_group: BindGroup,
    first_index: u32,
    index_count: u32,
}

pub struct TerrainPass {
    layouts: TerrainBindGroupLayouts,
    pipeline: TerrainPipeline,
    blackout_pipeline: TerrainPipeline,
    ground_sampler: wgpu::Sampler,
    blend_sampler: wgpu::Sampler,

    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    draws: Vec<PreparedDraw>,

    /// Kept alive for as long as the bind groups reference them.
    _textures: Vec<wgpu::Texture>,
    _draw_buffers: Vec<wgpu::Buffer>,
}

impl TerrainPass {
    pub fn new(device: &Device, surface_format: TextureFormat, depth_format: TextureFormat) -> Self {
        let shader = TerrainShader::new(device);
        let layouts = TerrainBindGroupLayouts::new(device);
        let layout = TerrainPipelineLayout::new(device, &layouts);
        let pipeline = TerrainPipeline::new(device, &layout, &shader, surface_format, depth_format);
        let blackout_pipeline =
            TerrainPipeline::new_blackout(device, &layout, &shader, surface_format, depth_format);

        // The ground texture tiles, and is the one thing here that is projected onto a
        // surface: trilinear and anisotropic, at the shipped `SetMaxAnisotropy(4)`
        // (`crate::texture::MAX_ANISOTROPY`, AGENTS.md §3.12). One tile spans 8 world cells,
        // so grazing views minify hard and this is exactly the case anisotropy is for.
        let ground_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("terrain_ground_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: crate::texture::MAX_ANISOTROPY,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..Default::default()
        });
        // The blend table is a lookup indexed by the packed vertex normal — it must not wrap,
        // and it must not mip: §3.4's additive compositing holds only because the five
        // directions' blends partition unity at every texel, and a filtered-down level would
        // not. It is uploaded single-mip to match.
        let blend_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("terrain_blend_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..Default::default()
        });

        let frame_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_frame_uniforms"),
            contents: bytemuck::cast_slice(&[FrameUniforms::new(
                glam::Mat4::IDENTITY.to_cols_array_2d(),
                [0.0; 3],
            )]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("terrain_frame"),
            layout: &layouts.frame,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });

        Self {
            layouts,
            pipeline,
            blackout_pipeline,
            ground_sampler,
            blend_sampler,
            frame_buffer,
            frame_bind_group,
            vertex_buffer: None,
            index_buffer: None,
            draws: Vec::new(),
            _textures: Vec::new(),
            _draw_buffers: Vec::new(),
        }
    }

    pub fn set_terrain(&mut self, device: &Device, queue: &Queue, terrain: &TerrainData) {
        self.draws.clear();
        self._textures.clear();
        self._draw_buffers.clear();

        if terrain.vertices.is_empty() || terrain.indices.is_empty() {
            self.vertex_buffer = None;
            self.index_buffer = None;
            tracing::warn!("Terrain has no geometry");
            return;
        }

        self.vertex_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&terrain.vertices),
            usage: BufferUsages::VERTEX,
        }));
        self.index_buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_index_buffer"),
            contents: bytemuck::cast_slice(&terrain.indices),
            usage: BufferUsages::INDEX,
        }));

        let ground_views: Vec<_> = terrain
            .textures
            .iter()
            .enumerate()
            .map(|(i, image)| self.upload(device, queue, image, &format!("terrain_ground_{i}")))
            .collect();
        let blend_views: Vec<_> = terrain
            .blend_tables
            .iter()
            .enumerate()
            .map(|(i, image)| self.upload(device, queue, image, &format!("terrain_blend_{i}")))
            .collect();

        for draw in &terrain.draws {
            let (Some(ground), Some(blend)) = (
                ground_views.get(draw.texture as usize),
                blend_views.get(draw.blend_table as usize),
            ) else {
                tracing::warn!(
                    "Terrain draw references texture {} / blend table {}, which do not exist \
                     — skipped",
                    draw.texture,
                    draw.blend_table,
                );
                continue;
            };

            let uniforms = DrawUniforms {
                uv_transform_u: draw.uv_transform_u,
                uv_transform_v: draw.uv_transform_v,
            };
            let buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("terrain_draw_uniforms"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: BufferUsages::UNIFORM,
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("terrain_draw"),
                layout: &self.layouts.draw,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(ground),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.ground_sampler),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(blend),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.blend_sampler),
                    },
                ],
            });

            self._draw_buffers.push(buffer);
            self.draws.push(PreparedDraw {
                bind_group,
                first_index: draw.first_index,
                index_count: draw.index_count,
            });
        }

        tracing::info!(
            "Terrain uploaded: {} vertices, {} indices, {} layer passes, {} ground textures",
            terrain.vertices.len(),
            terrain.indices.len(),
            self.draws.len(),
            terrain.textures.len(),
        );
    }

    fn upload(
        &mut self,
        device: &Device,
        queue: &Queue,
        image: &TextureImage,
        label: &str,
    ) -> TextureView {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: image.width.max(1),
                height: image.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: image.levels.len().max(1) as u32,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: image.format.wgpu_format(),
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        if image.is_complete() {
            crate::texture::write_levels(queue, &texture, image);
        } else {
            tracing::warn!(
                "{label}: {} levels totalling {} bytes are too few for {}x{} {:?} — left blank",
                image.levels.len(),
                image.levels.iter().map(Vec::len).sum::<usize>(),
                image.width,
                image.height,
                image.format,
            );
        }

        let view = texture.create_view(&TextureViewDescriptor::default());
        self._textures.push(texture);
        view
    }

    pub fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4], camera_pos: [f32; 3]) {
        queue.write_buffer(
            &self.frame_buffer,
            0,
            bytemuck::cast_slice(&[FrameUniforms::new(view_proj, camera_pos)]),
        );
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer)
        else {
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

        rpass.set_bind_group(0, &self.frame_bind_group, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);

        // The blackout pass over the same meshes, laying down depth and the black the layers
        // accumulate onto. Overlapping layers black each other out harmlessly.
        rpass.set_pipeline(&self.blackout_pipeline.0);
        for draw in &self.draws {
            rpass.set_bind_group(1, &draw.bind_group, &[]);
            rpass.draw_indexed(
                draw.first_index..draw.first_index + draw.index_count,
                0,
                0..1,
            );
        }

        rpass.set_pipeline(&self.pipeline.0);
        for draw in &self.draws {
            rpass.set_bind_group(1, &draw.bind_group, &[]);
            rpass.draw_indexed(
                draw.first_index..draw.first_index + draw.index_count,
                0,
                0..1,
            );
        }
    }
}
