//! The landscape foreground pass.
//!
//! The original draws the landscape as N alpha-blended passes per patch, one per theme
//! layer: the layer's ground texture supplies colour through a planar projection along the
//! layer's mapping direction, and a blend table indexed by the vertex normal supplies the
//! alpha (AGENTS.md §3.4). [`TerrainData`] arrives with that already resolved — one
//! [`TerrainDraw`] per distinct (texture, mapping direction), over a shared vertex and index
//! buffer. Building it from a `.lev` lives on the other side of the crate boundary.

use crate::TargetFormats;
use crate::bindless::{BindlessFrame, BindlessIndex, BindlessTextures, TextureKey};
use crate::image::TextureImage;
use crate::lighting::{LIGHTING_WGSL, LightingUniforms};
use bytemuck::{Pod, Zeroable};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    BufferUsages, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device,
    Extent3d, FragmentState, FrontFace, IndexFormat, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilState,
    TextureDescriptor, TextureDimension,
    TextureUsages, TextureView, TextureViewDescriptor, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
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
    /// `c3`/`c19`/`c20`/`c35`, shared with the static meshes and the repeated meshes.
    lighting: LightingUniforms,
    /// `c42`
    fade_transform: [f32; 4],
}

impl FrameUniforms {
    fn new(view_proj: [[f32; 4]; 4], camera_pos: [f32; 3]) -> FrameUniforms {
        FrameUniforms {
            view_proj,
            camera_pos: [camera_pos[0], camera_pos[1], camera_pos[2], 0.0],
            lighting: LightingUniforms::NEUTRAL,
            // Fade disabled, expressed in the real mechanism rather than bypassed:
            // `dot(dist, 0) + 1` saturates to 1 at every distance.
            // `ForegroundFadeStart`/`End` are set by a console command whose defaults are
            // not recoverable from the decomp (AGENTS.md §9).
            fade_transform: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Everything one layer pass needs, as immediate data — replacing a bind group *and* a
/// uniform buffer per draw (AGENTS.md §12.5).
///
/// Bigger than the other passes' 16 bytes because a layer pass binds a *pair* of textures
/// (§3.4) and carries the planar projection for its mapping direction. 48 bytes, against
/// Vulkan's guaranteed 128.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct DrawConstants {
    /// `c40`
    uv_transform_u: [f32; 4],
    /// `c41`
    uv_transform_v: [f32; 4],
    /// The layer's ground texture, sampled through `repeat_sampler`.
    ground_index: BindlessIndex,
    /// The mapping direction's blend table, sampled through `clamp_sampler` — it is a lookup
    /// indexed by the packed vertex normal, and must neither wrap nor mip (§3.4, step 7).
    blend_index: BindlessIndex,
    _pad: [u32; 2],
}

pub struct TerrainBindGroupLayouts {
    frame: BindGroupLayout,
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
        Self {
            frame: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("terrain_frame"),
                entries: &[uniform(
                    0,
                    ShaderStages::VERTEX_FRAGMENT,
                )],
            }),
        }
    }
}

pub struct TerrainShader(ShaderModule);

impl TerrainShader {
    pub fn new(device: &Device, bindless: &BindlessTextures) -> Self {
        Self(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{LIGHTING_WGSL}\n{}\n{}",
                    bindless.wgsl_prelude(),
                    include_str!("terrain.wgsl"),
                )
                .into(),
            ),
        }))
    }
}

pub struct TerrainPipelineLayout(PipelineLayout);

impl TerrainPipelineLayout {
    pub fn new(
        device: &Device,
        layouts: &TerrainBindGroupLayouts,
        bindless: &BindlessTextures,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&layouts.frame, bindless.layout()],
            immediate_size: size_of::<DrawConstants>() as u32,
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
        targets: TargetFormats,
    ) -> Self {
        Self(Self::build(
            device,
            layout,
            shader,
            targets,
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
            CompareFunction::GreaterEqual,
        ))
    }

    /// `VSHADER_LANDSCAPE_FOREGROUND_BLACKOUT_PASS`: the same meshes in solid black, opaque,
    /// establishing depth and an origin for the additive layers.
    pub fn new_blackout(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        targets: TargetFormats,
    ) -> Self {
        Self(Self::build(
            device,
            layout,
            shader,
            targets,
            "vs_blackout",
            "fs_blackout",
            None,
            true,
            CompareFunction::Greater,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn build(
        device: &Device,
        layout: &TerrainPipelineLayout,
        shader: &TerrainShader,
        targets: TargetFormats,
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
                    format: targets.colour,
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
                format: targets.depth,
                depth_write_enabled,
                depth_compare,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: targets.multisample(),
            multiview_mask: None,
            cache: None,
        })
    }
}

/// A draw's bind group, holding its own `c40`/`c41` and its two textures.
struct PreparedDraw {
    constants: DrawConstants,
    first_index: u32,
    index_count: u32,
}

pub struct TerrainPass {
    pipeline: TerrainPipeline,
    blackout_pipeline: TerrainPipeline,

    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,

    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    draws: Vec<PreparedDraw>,
    /// The registry generation `draws`' indices were registered under. Checked at draw time:
    /// terrain registers in `set_terrain`, which runs *before* the scene's model loading, so
    /// a clear landing between the two would leave every layer pass sampling whatever took
    /// its slot next (AGENTS.md §12.4).
    generation: u32,
}

impl TerrainPass {
    pub fn new(device: &Device, targets: TargetFormats, bindless: &BindlessTextures) -> Self {
        let shader = TerrainShader::new(device, bindless);
        let layouts = TerrainBindGroupLayouts::new(device);
        let layout = TerrainPipelineLayout::new(device, &layouts, bindless);
        let pipeline = TerrainPipeline::new(device, &layout, &shader, targets);
        let blackout_pipeline =
            TerrainPipeline::new_blackout(device, &layout, &shader, targets);

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
            pipeline,
            blackout_pipeline,
            frame_buffer,
            frame_bind_group,
            vertex_buffer: None,
            index_buffer: None,
            draws: Vec::new(),
            generation: 0,
        }
    }

    pub fn set_terrain(
        &mut self,
        device: &Device,
        queue: &Queue,
        bindless: &mut BindlessTextures,
        terrain: &TerrainData,
    ) {
        self.draws.clear();
        self.generation = bindless.generation();

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

        // Ground textures are keyed by slot, not asset id: the scene layer has already
        // deduped them by id on the way in, and a failed load falls back to a deliberately
        // visible magenta placeholder that has no asset behind it.
        let ground: Vec<BindlessIndex> = terrain
            .textures
            .iter()
            .enumerate()
            .map(|(i, image)| {
                let view = self.upload(device, queue, image, &format!("terrain_ground_{i}"));
                bindless.register(TextureKey::Ground(i as u32), view)
            })
            .collect::<Result<_, _>>()
            .unwrap_or_else(|error| {
                tracing::error!("Terrain ground textures: {error}");
                Vec::new()
            });
        let blend: Vec<BindlessIndex> = terrain
            .blend_tables
            .iter()
            .enumerate()
            .map(|(i, image)| {
                let view = self.upload(device, queue, image, &format!("terrain_blend_{i}"));
                bindless.register(TextureKey::BlendTable(i as u32), view)
            })
            .collect::<Result<_, _>>()
            .unwrap_or_else(|error| {
                tracing::error!("Terrain blend tables: {error}");
                Vec::new()
            });

        for draw in &terrain.draws {
            let (Some(&ground_index), Some(&blend_index)) = (
                ground.get(draw.texture as usize),
                blend.get(draw.blend_table as usize),
            ) else {
                tracing::warn!(
                    "Terrain draw references texture {} / blend table {}, which do not exist \
                     — skipped",
                    draw.texture,
                    draw.blend_table,
                );
                continue;
            };

            self.draws.push(PreparedDraw {
                constants: DrawConstants {
                    uv_transform_u: draw.uv_transform_u,
                    uv_transform_v: draw.uv_transform_v,
                    ground_index,
                    blend_index,
                    _pad: [0; 2],
                },
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
        &self,
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

        // The view keeps the texture alive, and the bindless registry keeps the view — so
        // there is no separate list of textures to hold onto any more.
        texture.create_view(&TextureViewDescriptor::default())
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
        bindless: BindlessFrame<'_>,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        let (Some(vertex_buffer), Some(index_buffer)) = (&self.vertex_buffer, &self.index_buffer)
        else {
            return;
        };
        // Terrain registers its textures in `set_terrain`, which runs before the scene's
        // models load. A clear between the two would leave every layer pass sampling whatever
        // took its slot next — silently, and looking almost right (AGENTS.md §12.4).
        //
        // **After the early return, not before.** A pass with no terrain has no indices to be
        // stale, and asserting first fired on any `clear_scene` that was not followed by a
        // `set_terrain` — which the game always does, but a test or a renderer used for
        // something other than a level does not. Matches the sky pass, which had it this way
        // round already.
        debug_assert_eq!(
            self.generation, bindless.generation,
            "terrain's bindless indices are from a cleared generation — \
             set_terrain must run after the scene is cleared, not before",
        );

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
                // `ClearPass` clears depth, not this pass — see its doc comment. This one only
                // ever loads, like every other drawing pass.
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        rpass.set_bind_group(1, bindless.bind_group, &[]);

        rpass.set_pipeline(&self.blackout_pipeline.0);
        for draw in &self.draws {
            rpass.set_immediates(0, bytemuck::bytes_of(&draw.constants));
            rpass.draw_indexed(
                draw.first_index..draw.first_index + draw.index_count,
                0,
                0..1,
            );
        }

        rpass.set_pipeline(&self.pipeline.0);
        for draw in &self.draws {
            rpass.set_immediates(0, bytemuck::bytes_of(&draw.constants));
            rpass.draw_indexed(
                draw.first_index..draw.first_index + draw.index_count,
                0,
                0..1,
            );
        }
    }
}
