//! Renders static meshes: every primitive, every per-material draw range, with the
//! material's base texture and alpha mode (opaque / alpha-test cutout / alpha-blended).
//!
//! A [`Model`] is a mesh *asset* — geometry and materials, uploaded once — and a slice of
//! [`ModelInstance`]s places it in the world. That split is what a level needs: LookoutPoint
//! puts 192 things on the ground drawn from 44 distinct meshes, one of which
//! (`MESH_SMALL_WALL_CURVED_POST_01`) is placed 50 times.
//!
//! **Skinned meshes draw through here too, in bind pose** ([`ModelKind::Animated`]).
//! `VSHADER_PALSKIN_DIRLIGHT_FOG` is `VSHADER_STATIC_DIRLIGHT` with a three-bone blend on the
//! front: it builds `Σ weightᵢ · BoneMatrix[indexᵢ]`, transforms position and normal by it, and
//! from `dp4 r2.x, r0, c5` onward is character-for-character the static shader over the same
//! `c19`/`c20`/`c35`/`c3`. With an identity palette that blend is a no-op, so this is the
//! *identity case* of the real shader rather than an approximation of it — and there is no
//! `PSHADER_PALSKIN` at all, so both kinds already share `PSHADER_TEXTURE_DIFFUSE`.
//! The kind exists only so `EnableStaticMeshes` and `EnableAnimatedMeshes` can gate
//! independently, as they do in `ego_r.exe` (AGENTS.md §3.9).
//!
//! Backface culling is enabled per-material: `two_sided` materials use `cull_mode: None`, the rest
//! use `cull_mode: Back`, with `front_face: Cw` — Fable's meshes are clockwise-front, matching
//! D3D9's default `D3DCULL_CCW`. See the note on the pipeline.

use crate::bindless::{BindlessIndex, BindlessTextures, TextureKey};
use crate::image::TextureImage;
use crate::lighting::{LIGHTING_WGSL, LightingUniforms};
use crate::TargetFormats;
use crate::texture::upload_texture;
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Face, FragmentState, FrontFace, IndexFormat,
    PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilState, TextureView, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Texels with alpha below this are discarded by alpha-test (cutout) materials.
// UNVERIFIED: not sourced from the game. AGENTS.md §9.
const ALPHA_CUTOFF: f32 = 0.5;

/// Which console toggle a model answers to.
///
/// Both kinds draw through the same pipeline with the same shader — see the module note. This
/// distinguishes them *only* for `EnableStaticMeshes` / `EnableAnimatedMeshes`, which the
/// original engine has as separate commands (AGENTS.md §3.9).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum ModelKind {
    /// `ENGINE_GRAPHIC_STATIC_MESH` things, and local detail's mesh objects.
    #[default]
    Static,
    /// `ENGINE_GRAPHIC_ANIMATING_MESH` things, drawn in the bind pose the asset ships in.
    Animated,
}

/// Which model kinds a frame draws — `RenderToggles`' two mesh switches, as the pass sees them.
///
/// The filter has to be applied *inside* [`ModelPass::pass`] rather than around it, because
/// transparent instances are depth-sorted across the whole level and both kinds share that
/// ordering.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModelKinds {
    pub static_meshes: bool,
    pub animated_meshes: bool,
}

impl ModelKinds {
    pub const ALL: ModelKinds = ModelKinds {
        static_meshes: true,
        animated_meshes: true,
    };

    fn allows(&self, kind: ModelKind) -> bool {
        match kind {
            ModelKind::Static => self.static_meshes,
            ModelKind::Animated => self.animated_meshes,
        }
    }

    fn any(&self) -> bool {
        self.static_meshes || self.animated_meshes
    }
}

/// How a material's alpha channel is treated.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum AlphaMode {
    /// Alpha ignored.
    #[default]
    Opaque,
    /// Alpha-tested: texels below [`ALPHA_CUTOFF`] are discarded, depth still written.
    Cutout,
    /// Alpha-blended, depth-tested but not depth-written, drawn back-to-front.
    Blend,
}

/// One material: its diffuse map (a 1×1 white texture stands in when absent) and how it
/// blends.
///
/// The map arrives as a *key plus optionally an image*, not just an image, because the
/// renderer dedups by key (AGENTS.md §12.6). A caller that knows the renderer already holds
/// `diffuse_id` may leave `diffuse` `None` and skip the archive read and the BC slice
/// entirely — which is where most of the win is: across a level, between a half and three
/// quarters of material texture references are repeats (§12.1).
pub struct ModelMaterial {
    /// The texture's global asset id (AGENTS.md §3.11), or `None` for a material with no
    /// diffuse map — roughly a quarter of `graphics.big`'s materials, so this is the normal
    /// case rather than an error, and it draws white.
    pub diffuse_id: Option<u32>,
    /// The decoded image. `None` with a `Some` `diffuse_id` means "you already have this
    /// one"; `None` with a `None` id means the material has no map at all.
    pub diffuse: Option<TextureImage>,
    pub alpha_mode: AlphaMode,
    pub two_sided: bool,
}

/// A contiguous run of a primitive's indices drawn with one material.
pub struct ModelSubMesh {
    /// Index into [`Model::materials`].
    pub material: u32,
    pub index_start: u32,
    pub index_count: u32,
}

/// One primitive's geometry and its per-material draw ranges.
pub struct ModelPrimitive {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u16>,
    pub sub_meshes: Vec<ModelSubMesh>,
}

/// A mesh asset ready to upload: geometry and materials, with no position of its own.
/// Where it stands is [`ModelInstance`]'s job.
pub struct Model {
    pub primitives: Vec<ModelPrimitive>,
    pub materials: Vec<ModelMaterial>,
}

/// One placement of a [`Model`].
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelInstance {
    /// Object → world, column-major, Z-up (AGENTS.md §3.6). The `World` half of the
    /// original's `CombinedProjectionMatrix`; see `CalcObjectMatrix`
    /// (`engine_primitive_manager_mesh_base.cpp:557`).
    pub transform: [[f32; 4]; 4],
    /// The pixel shader's `c0` — the per-object colour, tint times fade alpha, which
    /// `AddStaticMesh` takes as a `CRGBColour`. Opaque white leaves the material as authored.
    pub colour: [f32; 4],
}

impl ModelInstance {
    /// Four `Float32x4` columns of the object matrix, then the colour.
    const ATTRIBS: [VertexAttribute; 5] =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    /// World position — the transform's translation column. Used to depth-sort blended draws.
    fn position(&self) -> [f32; 3] {
        let t = self.transform[3];
        [t[0], t[1], t[2]]
    }
}

impl Default for ModelInstance {
    fn default() -> Self {
        ModelInstance {
            transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
            colour: [1.0; 4],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl ModelVertex {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

    pub(crate) fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// The per-frame shader constants, by their register names in the Lights layout (§3.8).
/// The same four lighting registers the landscape pass reads, so one environment supplies
/// both (AGENTS.md step 2).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct FrameUniforms {
    /// `c5..c8`
    view_proj: [[f32; 4]; 4],
    /// `c3`/`c19`/`c20`/`c35`, shared with the landscape and the repeated meshes.
    lighting: LightingUniforms,
}

impl FrameUniforms {
    fn new(view_proj: [[f32; 4]; 4]) -> FrameUniforms {
        FrameUniforms {
            view_proj,
            lighting: LightingUniforms::NEUTRAL,
        }
    }
}

/// Everything that used to be a per-material bind group, as 16 bytes of per-draw immediate
/// data (AGENTS.md §12.5). Must match `struct DrawConstants` in `model.wgsl`.
///
/// A bind group per material — texture, sampler and a two-field uniform buffer — is what this
/// replaces, and the buffer allocation with it: LookoutPoint's region allocated 189 of each.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct DrawConstants {
    /// Slot in the shared bindless array.
    texture_index: BindlessIndex,
    /// Non-zero enables alpha-test (cutout) in the shader.
    alpha_test: u32,
    alpha_cutoff: f32,
    _pad: u32,
}

pub struct ModelFrameBindGroupLayout(BindGroupLayout);

impl ModelFrameBindGroupLayout {
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

pub struct ModelShader(ShaderModule);

impl ModelShader {
    pub fn new(device: &Device, bindless: &BindlessTextures) -> Self {
        Self(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!(
                    "{LIGHTING_WGSL}\n{}\n{}",
                    bindless.wgsl_prelude(),
                    include_str!("model.wgsl"),
                )
                .into(),
            ),
        }))
    }
}

pub struct ModelPipelineLayout(PipelineLayout);

impl ModelPipelineLayout {
    pub fn new(
        device: &Device,
        frame_layout: &ModelFrameBindGroupLayout,
        bindless: &BindlessTextures,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            // Group 0 is the pass's own frame uniforms, group 1 the shared bindless group —
            // the same index the per-material group used, so this pass's migration touches no
            // other pass (AGENTS.md §12.4).
            bind_group_layouts: &[&frame_layout.0, bindless.layout()],
            immediate_size: size_of::<DrawConstants>() as u32,
        }))
    }
}

/// The opaque and alpha-blended pipeline variants, each with a culled and unculled version.
/// Cutout materials use the opaque pipeline and discard in the shader; blended materials use the
/// blend pipeline (depth test on, depth write off). `two_sided` materials use the unculled variant.
struct ModelPipelines {
    opaque_culled: RenderPipeline,
    opaque_unculled: RenderPipeline,
    blend_culled: RenderPipeline,
    blend_unculled: RenderPipeline,
}

impl ModelPipelines {
    fn new(
        device: &Device,
        layout: &ModelPipelineLayout,
        shader: &ModelShader,
        targets: TargetFormats,
    ) -> Self {
        let make = |blend: bool, cull: bool| {
            let color_target = ColorTargetState {
                format: targets.colour,
                blend: blend.then_some(BlendState::ALPHA_BLENDING),
                write_mask: if blend {
                    ColorWrites::ALL
                } else {
                    ColorWrites::COLOR
                },
            };
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(type_name::<Self>()),
                layout: Some(&layout.0),
                vertex: VertexState {
                    module: &shader.0,
                    entry_point: Some("vs_main"),
                    buffers: &[ModelVertex::layout(), ModelInstance::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader.0,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(color_target)],
                }),
                primitive: PrimitiveState {
                    // Fable's meshes are wound clockwise-front, matching D3D9's default
                    // `D3DCULL_CCW` (cull the counter-clockwise side). Measured, not assumed:
                    // over 400 meshes from graphics.big, the right-hand-rule normal of a
                    // triangle disagrees with its own vertex normals on 385,787 triangles and
                    // agrees on 1,082 — every one of the 400 is CW-front. wgpu defaults to
                    // `Ccw`, which culled the front faces and drew the interior.
                    front_face: FrontFace::Cw,
                    cull_mode: cull.then_some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: targets.depth,
                    depth_write_enabled: !blend,
                    // Reverse-Z (`Camera::projection_matrix`): closer is a *greater* depth.
                    depth_compare: CompareFunction::Greater,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: targets.multisample(),
                multiview_mask: None,
                cache: None,
            })
        };

        Self {
            opaque_culled: make(false, true),
            opaque_unculled: make(false, false),
            blend_culled: make(true, true),
            blend_unculled: make(true, false),
        }
    }
}

/// One material, resolved to the 16 bytes a draw needs plus how it blends.
struct GpuMaterial {
    draw: DrawConstants,
    transparent: bool,
}

/// One draw range within a primitive's index buffer, resolved to a material slot.
struct SubMeshDraw {
    material: usize,
    index_start: u32,
    index_count: u32,
    /// Whether this sub-mesh should be backface-culled (false for `two_sided` materials).
    cull: bool,
}

/// One primitive's uploaded geometry plus its per-material sub-draws.
struct GpuPrimitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sub_meshes: Vec<SubMeshDraw>,
}

/// A mesh asset uploaded once, with every placement of it in one instance buffer.
struct GpuModel {
    materials: Vec<GpuMaterial>,
    primitives: Vec<GpuPrimitive>,
    instance_buffer: wgpu::Buffer,
    instances: Vec<ModelInstance>,
    /// Whether any material on this model blends. Only these models contribute to the
    /// depth-sorted pass, so a level of fully opaque meshes sorts nothing.
    has_transparent: bool,
    kind: ModelKind,
}

#[derive(Debug, Display, Error)]
pub enum AddModelError {
    #[display("model has no primitives")]
    NoPrimitives,
    #[display("model has no instances to place")]
    NoInstances,
    #[display("no room in the bindless texture array: {_0}")]
    BindlessFull(crate::bindless::BindlessFull),
}

/// Resolve each material to the 16 bytes a draw needs, registering its diffuse map in the
/// shared array on the way (AGENTS.md §12.6).
///
/// A repeat asset id is a cache hit: `register` returns the slot it already gave out, and the
/// image — which the caller may not even have decoded — is dropped. A material with no map at
/// all points at slot 0's... no: it points at the *fallback*, via
/// [`BindlessTextures::fallback_index`], which is white, so it draws the material's own colour
/// rather than a hole.
fn build_materials(
    device: &Device,
    queue: &Queue,
    bindless: &mut BindlessTextures,
    materials_in: &[ModelMaterial],
) -> Result<Vec<GpuMaterial>, AddModelError> {
    let mut materials = Vec::with_capacity(materials_in.len());
    for material in materials_in {
        let texture_index = match material.diffuse_id {
            Some(id) => {
                let key = TextureKey::Asset(id);
                match (bindless.index_of(key), &material.diffuse) {
                    // Already resident — the dedup, and the caller skipped the decode too.
                    (Some(index), _) => index,
                    (None, Some(image)) => {
                        let view = upload_texture(device, queue, "model_material_diffuse", image);
                        bindless
                            .register(key, view)
                            .map_err(AddModelError::BindlessFull)?
                    }
                    // An id with no image and no registration: the caller believed we had it
                    // and we do not. Draw white rather than the wrong texture, and say so.
                    (None, None) => {
                        tracing::warn!("Material texture {id} was not registered — drawing white");
                        bindless.fallback_index()
                    }
                }
            }
            None => bindless.fallback_index(),
        };

        materials.push(GpuMaterial {
            draw: DrawConstants {
                texture_index,
                alpha_test: (material.alpha_mode == AlphaMode::Cutout) as u32,
                alpha_cutoff: ALPHA_CUTOFF,
                _pad: 0,
            },
            transparent: material.alpha_mode == AlphaMode::Blend,
        });
    }
    Ok(materials)
}

pub struct ModelPass {
    pipelines: ModelPipelines,
    meshes: Vec<GpuModel>,
    /// One buffer for the whole pass — the static mesh shader has no per-object constants.
    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,
    /// Camera world-space position, used for depth-sorting transparent draws.
    camera_pos: [f32; 3],
}

impl ModelPass {
    pub fn new(device: &Device, targets: TargetFormats, bindless: &BindlessTextures) -> Self {
        let shader = ModelShader::new(device, bindless);
        let frame_layout = ModelFrameBindGroupLayout::new(device);
        let layout = ModelPipelineLayout::new(device, &frame_layout, bindless);
        let pipelines = ModelPipelines::new(device, &layout, &shader, targets);

        let frame_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model_frame_buffer"),
            contents: bytemuck::cast_slice(&[FrameUniforms::new(
                glam::Mat4::IDENTITY.to_cols_array_2d(),
            )]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model_frame_bind_group"),
            layout: &frame_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipelines,
            meshes: Vec::new(),
            frame_buffer,
            frame_bind_group,
            camera_pos: [0.0; 3],
        }
    }

    pub fn clear_models(&mut self) {
        self.meshes.clear();
    }

    /// Upload one mesh asset and every placement of it. Geometry, materials and textures
    /// are uploaded once no matter how many instances there are.
    ///
    /// `kind` decides which console toggle the placements answer to, nothing else — it does
    /// not change the pipeline, the shader or the draw.
    pub fn add_model(
        &mut self,
        device: &Device,
        queue: &Queue,
        bindless: &mut BindlessTextures,
        model: &Model,
        instances: &[ModelInstance],
        kind: ModelKind,
    ) -> Result<(), AddModelError> {
        if model.primitives.is_empty() {
            return Err(AddModelError::NoPrimitives);
        }
        if instances.is_empty() {
            return Err(AddModelError::NoInstances);
        }

        let materials = build_materials(device, queue, bindless, &model.materials)?;
        let primitives = build_primitives(device, model);

        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model_instance_buffer"),
            contents: bytemuck::cast_slice(instances),
            usage: BufferUsages::VERTEX,
        });

        let has_transparent = materials.iter().any(|m| m.transparent);

        self.meshes.push(GpuModel {
            materials,
            primitives,
            instance_buffer,
            instances: instances.to_vec(),
            has_transparent,
            kind,
        });
        Ok(())
    }

    /// Total placements across every uploaded mesh — what the level actually draws.
    pub fn instance_count(&self) -> usize {
        self.meshes.iter().map(|m| m.instances.len()).sum()
    }

    pub fn model_count(&self) -> usize {
        self.meshes.len()
    }

    /// Placements of one kind — what `ShowStats` reports per toggle.
    pub fn instance_count_of(&self, kind: ModelKind) -> usize {
        self.meshes
            .iter()
            .filter(|m| m.kind == kind)
            .map(|m| m.instances.len())
            .sum()
    }

    /// Uploaded mesh assets of one kind.
    pub fn model_count_of(&self, kind: ModelKind) -> usize {
        self.meshes.iter().filter(|m| m.kind == kind).count()
    }

    pub fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        queue.write_buffer(
            &self.frame_buffer,
            0,
            bytemuck::cast_slice(&[FrameUniforms::new(view_proj)]),
        );
    }

    pub fn set_camera_pos(&mut self, pos: glam::Vec3) {
        self.camera_pos = pos.to_array();
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        bindless: &BindGroup,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
        kinds: ModelKinds,
    ) {
        if self.meshes.is_empty() || !kinds.any() {
            return;
        }

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
        // Once for the whole pass, not once per material — the array holds every texture the
        // level uses, and a draw picks its slot with an immediate instead (AGENTS.md §12.4).
        rpass.set_bind_group(1, bindless, &[]);

        // Opaque and cutout first — they write depth, and every placement of a mesh draws
        // in one instanced call.
        for mesh in self.meshes.iter().filter(|m| kinds.allows(m.kind)) {
            mesh.draw_opaque(&mut rpass, &self.pipelines);
        }

        // Then the blended ones, back to front across the whole level rather than within
        // one model, so a distant transparent object cannot paint over a near one. Sorting
        // is per instance, so these draw one at a time.
        for (mesh, instance) in self.sorted_transparent_instances(kinds) {
            mesh.draw_transparent(&mut rpass, &self.pipelines, instance);
        }
    }

    /// Every transparent placement in the level, farthest first.
    fn sorted_transparent_instances(&self, kinds: ModelKinds) -> Vec<(&GpuModel, u32)> {
        let mut draws: Vec<(&GpuModel, u32)> = self
            .meshes
            .iter()
            .filter(|m| m.has_transparent && kinds.allows(m.kind))
            .flat_map(|m| (0..m.instances.len() as u32).map(move |i| (m, i)))
            .collect();
        draws.sort_by(|(a, i), (b, j)| {
            let da = dist_sq(&a.instances[*i as usize].position(), &self.camera_pos);
            let db = dist_sq(&b.instances[*j as usize].position(), &self.camera_pos);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
        draws
    }
}

impl GpuModel {
    /// One instanced draw per (primitive, material) — the whole placement set at once.
    fn draw_opaque(&self, rpass: &mut wgpu::RenderPass<'_>, pipelines: &ModelPipelines) {
        let instances = 0..self.instances.len() as u32;
        for primitive in &self.primitives {
            let mut bound = false;
            for sub in &primitive.sub_meshes {
                let Some(material) = self.materials.get(sub.material) else {
                    continue;
                };
                if material.transparent {
                    continue;
                }
                if !bound {
                    self.bind(rpass, primitive);
                    bound = true;
                }
                let pipeline = if sub.cull {
                    &pipelines.opaque_culled
                } else {
                    &pipelines.opaque_unculled
                };
                rpass.set_pipeline(pipeline);
                rpass.set_immediates(0, bytemuck::bytes_of(&material.draw));
                let end = sub.index_start + sub.index_count;
                rpass.draw_indexed(sub.index_start..end, 0, instances.clone());
            }
        }
    }

    /// Draw only this model's blended sub-meshes, and only for placement `instance`.
    fn draw_transparent(
        &self,
        rpass: &mut wgpu::RenderPass<'_>,
        pipelines: &ModelPipelines,
        instance: u32,
    ) {
        for primitive in &self.primitives {
            let mut bound = false;
            for sub in &primitive.sub_meshes {
                let Some(material) = self.materials.get(sub.material) else {
                    continue;
                };
                if !material.transparent {
                    continue;
                }
                if !bound {
                    self.bind(rpass, primitive);
                    bound = true;
                }
                let pipeline = if sub.cull {
                    &pipelines.blend_culled
                } else {
                    &pipelines.blend_unculled
                };
                rpass.set_pipeline(pipeline);
                rpass.set_immediates(0, bytemuck::bytes_of(&material.draw));
                let end = sub.index_start + sub.index_count;
                rpass.draw_indexed(sub.index_start..end, 0, instance..instance + 1);
            }
        }
    }

    fn bind(&self, rpass: &mut wgpu::RenderPass<'_>, primitive: &GpuPrimitive) {
        rpass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rpass.set_index_buffer(primitive.index_buffer.slice(..), IndexFormat::Uint16);
    }
}

fn dist_sq(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Upload every primitive's geometry and resolve its sub-mesh draw ranges. Empty primitives
/// (no vertices or no indices) are skipped so we never create a zero-sized GPU buffer.
fn build_primitives(device: &Device, model: &Model) -> Vec<GpuPrimitive> {
    model
        .primitives
        .iter()
        .filter(|p| !p.vertices.is_empty() && !p.indices.is_empty())
        .map(|primitive| {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("model_vertex_buffer"),
                contents: bytemuck::cast_slice(&primitive.vertices),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("model_index_buffer"),
                contents: bytemuck::cast_slice(&primitive.indices),
                usage: BufferUsages::INDEX,
            });

            let sub_meshes = primitive
                .sub_meshes
                .iter()
                .map(|s| SubMeshDraw {
                    material: s.material as usize,
                    index_start: s.index_start,
                    index_count: s.index_count,
                    cull: model
                        .materials
                        .get(s.material as usize)
                        .map(|m| !m.two_sided)
                        .unwrap_or(true),
                })
                .collect();

            GpuPrimitive {
                vertex_buffer,
                index_buffer,
                sub_meshes,
            }
        })
        .collect()
}
