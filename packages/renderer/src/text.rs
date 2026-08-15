//! Draws screen-space text: one instanced quad per glyph, after the resolve.
//!
//! **The pass that is not multisampled.** Every other pass draws into the 4× MSAA colour
//! texture; this one runs after [`crate::ResolvePass`] has folded that into the presentable
//! texture, so it is built for a sample count of 1 — see [`TargetFormats::post_resolve`]. That
//! is not a compromise: coverage-antialiased glyphs are already antialiased, and running them
//! through MSAA would cost samples to change nothing.
//!
//! **The pass that indexes non-uniformly.** Each instance carries its own bindless slot, so
//! neighbouring fragments in one draw sample different textures. `TEXTURE_BINDING_ARRAY` alone
//! does not cover that — this is the first and only user of
//! `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING`, requested since §12.8 step
//! 1 and unexercised until now (AGENTS.md §12.9, §13.4).
//!
//! Rasterizing is *not* here. Glyph bitmaps arrive as `TextureImage`s from `openalbion::text`,
//! the same way meshes arrive as `Model`s (§11.1).

use crate::TargetFormats;
use crate::bindless::{BindlessFrame, BindlessIndex, BindlessTextures, TextureKey};
use crate::texture::upload_texture;
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, TextureView, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// One glyph, positioned in physical pixels with the origin at the target's top left.
///
/// There is no UV rect. A glyph is its own texture and is sampled across the whole `0..1`
/// range, so a rect would be `(0, 0, 1, 1)` on every instance — 16 bytes of nothing per glyph
/// and a mechanism with nothing to check it. It is what an atlas would add, and §13.2 chose not
/// to have one.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlyphInstance {
    /// `(x, y, width, height)` of the glyph's bitmap — the pen position plus the glyph's own
    /// bearings, which the text layer has already applied.
    pub rect: [f32; 4],
    /// Straight (non-premultiplied) RGBA. Alpha is scaled by the glyph's coverage.
    pub colour: [f32; 4],
    /// The slot [`crate::Renderer::add_glyph`] returned. **Valid only for the bindless
    /// generation it was registered under** — see the `debug_assert` in [`TextPass::pass`].
    pub texture_index: BindlessIndex,
    pub _pad: [u32; 3],
}

impl GlyphInstance {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Uint32];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Matches `struct FrameUniforms` in `text.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct FrameUniforms {
    viewport: [f32; 4],
}

#[derive(Debug, Display, Error)]
pub enum AddGlyphError {
    #[display("glyph bitmap is short for its declared size")]
    Incomplete,
    #[display("no room in the bindless texture array: {_0}")]
    BindlessFull(crate::bindless::BindlessFull),
}

pub struct TextPass {
    pipeline: RenderPipeline,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,
    /// Rebuilt whenever the text changes, which for a console is whenever a key is pressed —
    /// not every frame. `None` when there is nothing to draw, which is the default and the
    /// state `--screenshot` must stay in (§13.6).
    instances: Option<(wgpu::Buffer, u32)>,
    /// The registry generation the current instances' indices were resolved under.
    generation: u32,
}

impl TextPass {
    pub fn new(device: &Device, targets: TargetFormats, bindless: &BindlessTextures) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{}\n{}", bindless.wgsl_prelude(), include_str!("text.wgsl")).into(),
            ),
        });

        let frame_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("text_frame_layout"),
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
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            // Group 1 is the shared bindless group, at the index every pass uses (§12.4).
            bind_group_layouts: &[&frame_layout, bindless.layout()],
            // No immediates: the texture slot is per *instance* here, not per draw, which is
            // the whole point of this pass (§13.4).
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(type_name::<Self>()),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                // Instances only. The quad's corners come from `vertex_index` (see the shader).
                buffers: &[GlyphInstance::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: targets.colour,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    // Colour only. The target is the presentable texture, and writing its
                    // alpha would hand the compositor a half-transparent frame under some
                    // `CompositeAlphaMode`s.
                    write_mask: ColorWrites::COLOR,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                // A screen-space quad has no meaningful facing, and its winding depends only on
                // how `vertex_index` unpacks — so culling here could only ever be a bug.
                cull_mode: None,
                ..Default::default()
            },
            // No depth attachment: this draws over a finished frame, in instance order.
            depth_stencil: None,
            multisample: targets.post_resolve_multisample(),
            multiview_mask: None,
            cache: None,
        });

        let frame_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("text_frame_buffer"),
            contents: bytemuck::bytes_of(&FrameUniforms {
                viewport: [1.0, 1.0, 0.0, 0.0],
            }),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("text_frame_bind_group"),
            layout: &frame_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            frame_buffer,
            frame_bind_group,
            instances: None,
            generation: 0,
        }
    }

    /// The target's size in physical pixels — what [`GlyphInstance::rect`] is measured in.
    pub fn set_viewport(&self, queue: &Queue, size: [u32; 2]) {
        queue.write_buffer(
            &self.frame_buffer,
            0,
            bytemuck::bytes_of(&FrameUniforms {
                viewport: [size[0].max(1) as f32, size[1].max(1) as f32, 0.0, 0.0],
            }),
        );
    }

    /// Register one rasterized glyph, or return the slot it already holds.
    ///
    /// Keyed by the text layer's opaque `key` rather than by anything the renderer understands
    /// — it must not know what a font or a character is (§11.1).
    pub fn add_glyph(
        &mut self,
        device: &Device,
        queue: &Queue,
        bindless: &mut BindlessTextures,
        key: u64,
        image: &crate::TextureImage,
    ) -> Result<BindlessIndex, AddGlyphError> {
        let texture_key = TextureKey::Glyph(key);
        if let Some(index) = bindless.index_of(texture_key) {
            return Ok(index);
        }
        if !image.is_complete() {
            return Err(AddGlyphError::Incomplete);
        }

        let view = upload_texture(device, queue, "glyph", image);
        bindless
            .register(texture_key, view)
            .map_err(AddGlyphError::BindlessFull)
    }

    /// Replace everything this pass draws.
    ///
    /// An empty slice means "draw nothing", which is the state `--screenshot` stays in while
    /// the console is closed — and the reason the byte-identical capture keeps working (§13.6).
    pub fn set_text(&mut self, device: &Device, bindless: &BindlessTextures, instances: &[GlyphInstance]) {
        self.generation = bindless.generation();
        self.instances = (!instances.is_empty()).then(|| {
            (
                device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("text_instance_buffer"),
                    contents: bytemuck::cast_slice(instances),
                    usage: BufferUsages::VERTEX,
                }),
                instances.len() as u32,
            )
        });
    }

    /// Glyphs currently queued to draw.
    pub fn glyph_count(&self) -> u32 {
        self.instances.as_ref().map_or(0, |&(_, count)| count)
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        bindless: BindlessFrame<'_>,
        target_texture_view: &TextureView,
    ) {
        let Some((instance_buffer, instance_count)) = &self.instances else {
            return;
        };

        // The hazard §13.2a names: glyphs outlive a scene load, the registry does not. If a
        // `clear_scene` landed between `set_text` and here, these indices now point at whatever
        // took their slots — which draws as plausible garbage rather than as nothing.
        debug_assert_eq!(
            self.generation, bindless.generation,
            "text is drawing with bindless indices from a cleared generation — \
             re-register the glyphs and call set_text again after a scene load",
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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.frame_bind_group, &[]);
        rpass.set_bind_group(1, bindless.bind_group, &[]);
        rpass.set_vertex_buffer(0, instance_buffer.slice(..));
        // Every glyph on screen in **one** draw, each instance reaching a different slot.
        rpass.draw(0..4, 0..*instance_count);
    }
}
