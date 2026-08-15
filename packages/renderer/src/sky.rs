//! Outer sky and base band.
//!
//! Geometry follows `CEngineSkyRenderer::BuildOuterSkyMesh` /`BuildBaseBandMesh`
//! (engine_sky_renderer.cpp:545 / :666). The gradient colours that drive it are *not*
//! wired yet: they come from the environment colour LUT at
//! `(ColourLookupColumn + keyframe, SkyGradient*LookupRow)` as an unfiltered integer
//! texel fetch, which lands with the environment layer (AGENTS.md §3.1, step 2).
//! Until then the caller passes zeroed gradients, so the sky shows the raw sky
//! texture — the unimplemented half is visible rather than faked.

use crate::bindless::{BindlessFrame, BindlessIndex, BindlessTextures, TextureKey};
use crate::image::TextureImage;
use crate::TargetFormats;
use crate::texture::upload_texture;
use bytemuck::{Pod, Zeroable};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, Device, FragmentState, IndexFormat, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, TextureView, VertexAttribute, VertexBufferLayout,
    VertexState, VertexStepMode,
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
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub(crate) struct SkyUniforms {
    pub view_proj: [[f32; 4]; 4],
    /// `c92` in `VSHADER_OUTER_SKY` — SkyGradientTop colour, alpha from the
    /// SkyGradientTopAlpha row's red channel.
    pub gradient_top: [f32; 4],
    /// `c93` in `VSHADER_OUTER_SKY` — SkyGradientBottom, alpha likewise.
    pub gradient_bottom: [f32; 4],
    /// `c0.w` in `PSHADER_OUTER_SKY` — blend between sky texture 0 and 1.
    pub texture_blend: f32,
    pub _pad: [f32; 3],
}

fn build_outer_sky_mesh(segments: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // engine_sky_renderer.cpp:545 BuildOuterSkyMesh — literals 7e3f, -5e2f, 6.5e3.
    // Z is up, as in the original.
    let dome_top_z: f32 = 7000.0;
    let dome_bottom_z: f32 = -500.0;
    let dome_radius: f32 = 6500.0;

    // Center vertex at the top (zenith cap).
    vertices.push(SkyVertex {
        position: [0.0, 0.0, dome_top_z],
        color: [0.0, 0.0, 0.0, 0.0],
        uv: [0.5, 0.0],
    });

    // Top ring: diffuse=black, V=0.
    let top_base = 1u16;
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * dome_radius;
        let y = angle.sin() * dome_radius;
        let u = i as f32 / segments as f32;
        vertices.push(SkyVertex {
            position: [x, y, dome_top_z],
            color: [0.0, 0.0, 0.0, 0.0],
            uv: [u, 0.0],
        });
    }

    // Bottom ring: diffuse=white, V=1.
    let bottom_base = top_base + segments as u16 + 1;
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * dome_radius;
        let y = angle.sin() * dome_radius;
        let u = i as f32 / segments as f32;
        vertices.push(SkyVertex {
            position: [x, y, dome_bottom_z],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [u, 1.0],
        });
    }

    // Triangle fan: center to top ring (zenith cap).
    for i in 0..segments {
        let t0 = top_base + i as u16;
        let t1 = top_base + i as u16 + 1;
        indices.extend_from_slice(&[0, t0, t1]);
    }

    // Cylinder wall.
    for i in 0..segments {
        let t0 = top_base + i as u16;
        let b0 = bottom_base + i as u16;
        let t1 = top_base + i as u16 + 1;
        let b1 = bottom_base + i as u16 + 1;
        indices.extend_from_slice(&[t0, b0, t1, t1, b0, b1]);
    }

    (vertices, indices)
}

fn build_base_band_mesh(segments: u32) -> (Vec<SkyVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // engine_sky_renderer.cpp:666 BuildBaseBandMesh — centre at Z = -1e4f, ring at
    // radius 6.5e3 and Z = -5e2f.
    let dome_radius: f32 = 6500.0;
    let dome_bottom_z: f32 = -500.0;
    let base_center_z: f32 = -10000.0;

    vertices.push(SkyVertex {
        position: [0.0, 0.0, base_center_z],
        color: [0.0, 0.0, 0.0, 0.0],
        uv: [0.5, 0.5],
    });

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * dome_radius;
        let y = angle.sin() * dome_radius;
        vertices.push(SkyVertex {
            position: [x, y, dome_bottom_z],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0],
        });
    }

    for i in 0..segments {
        indices.extend_from_slice(&[0, 1 + i as u16, 2 + i as u16]);
    }

    (vertices, indices)
}

/// A sky mesh with its own uniform buffer, so dome and base band can be drawn with
/// different constants from the same pipeline.
struct SkyMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
}

impl SkyMesh {
    fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        label: &str,
        vertices: &[SkyVertex],
        indices: &[u16],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&[SkyUniforms::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
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

    fn update_uniforms(&self, queue: &Queue, uniforms: &SkyUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    fn draw(&self, rpass: &mut wgpu::RenderPass<'_>) {
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

pub struct SkyUniformBindGroupLayout(BindGroupLayout);

impl SkyUniformBindGroupLayout {
    pub fn new(device: &Device) -> Self {
        Self(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(type_name::<Self>()),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
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

/// Which of `PSHADER_OUTER_SKY`'s two texture stages a slot in the bindless array is in.
///
/// The sky is the one pass whose textures change *mid-run* rather than at scene load: the
/// keyframe pair advances with the clock, and `refresh_sky` re-uploads only when the pair
/// actually changes. So this is where `rebuild_if_dirty` earns its name.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
struct DrawConstants {
    /// t0 — the primary sky texture.
    texture0_index: BindlessIndex,
    /// t1 — the one it blends toward, by the pixel shader's `c0.w`.
    texture1_index: BindlessIndex,
    _pad: [u32; 2],
}

pub struct OuterSkyShader(ShaderModule);

impl OuterSkyShader {
    pub fn new(device: &Device, bindless: &BindlessTextures) -> Self {
        Self(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("outer_sky.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{}\n{}",
                    bindless.wgsl_prelude(),
                    include_str!("sky/outer_sky.wgsl"),
                )
                .into(),
            ),
        }))
    }
}

pub struct OuterSkyPipelineLayout(PipelineLayout);

impl OuterSkyPipelineLayout {
    pub fn new(
        device: &Device,
        uniform_layout: &SkyUniformBindGroupLayout,
        bindless: &BindlessTextures,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, bindless.layout()],
            immediate_size: size_of::<DrawConstants>() as u32,
        }))
    }
}

pub struct OuterSkyPipeline(RenderPipeline);

impl OuterSkyPipeline {
    pub fn new(
        device: &Device,
        layout: &OuterSkyPipelineLayout,
        shader: &OuterSkyShader,
        targets: TargetFormats,
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
                targets: &[Some(targets.colour.into())],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: targets.multisample(),
            multiview_mask: None,
            cache: None,
        }))
    }
}

pub struct OuterSkyPass {
    pipeline: OuterSkyPipeline,
    dome: SkyMesh,
    base_band: SkyMesh,
    /// `None` until the first texture is set — the sky is optional, and a level without one
    /// should draw the rest of the world rather than a wrong colour.
    draw: Option<DrawConstants>,
    /// The registry generation `draw`'s indices belong to. The sky registers on the clock
    /// rather than at scene load, so it is the one pass that can be mid-registration when a
    /// scene is cleared (AGENTS.md §12.4).
    generation: u32,
}

impl OuterSkyPass {
    pub fn new(device: &Device, targets: TargetFormats, bindless: &BindlessTextures) -> Self {
        let shader = OuterSkyShader::new(device, bindless);
        let uniform_layout = SkyUniformBindGroupLayout::new(device);
        let layout = OuterSkyPipelineLayout::new(device, &uniform_layout, bindless);
        let pipeline = OuterSkyPipeline::new(device, &layout, &shader, targets);

        // 36 segments: engine_sky_renderer.cpp:616 loops `while (uVar13 < 0x24)`.
        let (dome_vertices, dome_indices) = build_outer_sky_mesh(36);
        let dome = SkyMesh::new(
            device,
            &uniform_layout,
            "sky_dome",
            &dome_vertices,
            &dome_indices,
        );

        // 36 segments: engine_sky_renderer.cpp:711 loops `while (uVar7 < 0x24)`.
        let (band_vertices, band_indices) = build_base_band_mesh(36);
        let base_band = SkyMesh::new(
            device,
            &uniform_layout,
            "sky_base_band",
            &band_vertices,
            &band_indices,
        );

        Self {
            pipeline,
            dome,
            base_band,
            draw: None,
            generation: 0,
        }
    }

    /// Forget which slots the sky was drawing with — its registrations went with the scene.
    pub fn clear(&mut self) {
        self.draw = None;
    }

    /// Set one of the two sky texture stages.
    ///
    /// `secondary` picks `t1`, the one `PSHADER_OUTER_SKY` blends toward by `c0.w`; `t0` is
    /// the primary. Registered by asset id like everything else, so a texture shared between
    /// two keyframes — or with anything else on screen — is one upload.
    pub fn set_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        bindless: &mut BindlessTextures,
        secondary: bool,
        asset_id: u32,
        image: &TextureImage,
    ) {
        let key = TextureKey::Asset(asset_id);
        let index = match bindless.index_of(key) {
            Some(index) => index,
            None => {
                let view = upload_texture(device, queue, "sky_texture", image);
                match bindless.register(key, view) {
                    Ok(index) => index,
                    Err(error) => {
                        tracing::warn!("Sky texture {asset_id}: {error} — sky unchanged");
                        return;
                    }
                }
            }
        };

        // A generation change means the scene was cleared under us and the other stage's
        // index is stale, so start the pair afresh rather than pairing old with new.
        let mut draw = match self.draw {
            Some(draw) if self.generation == bindless.generation() => draw,
            _ => DrawConstants {
                texture0_index: index,
                texture1_index: index,
                _pad: [0; 2],
            },
        };
        self.generation = bindless.generation();

        if secondary {
            draw.texture1_index = index;
        } else {
            draw.texture0_index = index;
            // Until a second texture arrives, t1 falls back to t0 — the blend then does
            // nothing rather than sampling a slot nobody filled.
            if self.draw.is_none() {
                draw.texture1_index = index;
            }
        }
        self.draw = Some(draw);
    }

    /// `gradient_top`/`gradient_bottom` are `c92`/`c93`; `texture_blend` is the pixel
    /// shader's `c0.w`. All three are supplied by the caller — this pass does no data
    /// lookup of its own.
    pub fn update_uniforms(
        &self,
        queue: &Queue,
        view_proj: [[f32; 4]; 4],
        gradient_top: [f32; 4],
        gradient_bottom: [f32; 4],
        texture_blend: f32,
    ) {
        let uniforms = SkyUniforms {
            view_proj,
            gradient_top,
            gradient_bottom,
            texture_blend,
            _pad: [0.0; 3],
        };
        self.dome.update_uniforms(queue, &uniforms);
        self.base_band.update_uniforms(queue, &uniforms);
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        bindless: BindlessFrame<'_>,
        target_texture_view: &TextureView,
    ) {
        let Some(draw) = self.draw else {
            tracing::debug!("Sky pass: no textures registered — sky skipped");
            return;
        };
        debug_assert_eq!(
            self.generation, bindless.generation,
            "sky's bindless indices are from a cleared generation",
        );

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
        rpass.set_bind_group(1, bindless.bind_group, &[]);
        rpass.set_immediates(0, bytemuck::bytes_of(&draw));

        self.dome.draw(&mut rpass);

        // The base band wants its own shader (`VSHADER_SKY_BASE_BAND` is `mov oD0, c92`
        // — flat gradient colour, no texture). Drawn through the outer-sky pipeline for
        // now; corrected in AGENTS.md step 4.1.
        self.base_band.draw(&mut rpass);
    }
}
