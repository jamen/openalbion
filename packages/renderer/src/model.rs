//! Renders static meshes: every primitive, every per-material draw range, with the
//! material's base texture and alpha mode (opaque / alpha-test cutout / alpha-blended).
//!
//! A [`Model`] is a mesh *asset* — geometry and materials, uploaded once — and a slice of
//! [`ModelInstance`]s places it in the world. That split is what a level needs: LookoutPoint
//! puts 192 things on the ground drawn from 44 distinct meshes, one of which
//! (`MESH_SMALL_WALL_CURVED_POST_01`) is placed 50 times.
//!
//! Backface culling is enabled per-material: `two_sided` materials use `cull_mode: None`, the rest
//! use `cull_mode: Back`, with `front_face: Cw` — Fable's meshes are clockwise-front, matching
//! D3D9's default `D3DCULL_CCW`. See the note on the pipeline.

use crate::image::TextureImage;
use crate::TargetFormats;
use crate::texture::{repeat_sampler, upload_texture};
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Extent3d, Face, FragmentState, FrontFace, IndexFormat,
    PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderStages, StencilState,
    TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Texels with alpha below this are discarded by alpha-test (cutout) materials.
// UNVERIFIED: not sourced from the game. AGENTS.md §9.
const ALPHA_CUTOFF: f32 = 0.5;

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
pub struct ModelMaterial {
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
    /// `c3`
    ambient: [f32; 4],
    /// `c19`
    light_dir: [f32; 4],
    /// `c20`
    diffuse: [f32; 4],
    /// `c35`
    backlight: [f32; 4],
}

/// The lighting the pass runs with until the environment layer lands (AGENTS.md step 2).
///
/// UNVERIFIED, and deliberately inert rather than plausible — the same stand-in
/// `TerrainPass` uses, for the same reason: an `Ambient` of 0.5 cancels the pixel shader's
/// `mul_x2` exactly, so meshes show their textures at their authored colour with no
/// directional term at all. The mechanism is fully wired; step 2 changes only these values.
impl FrameUniforms {
    fn new(view_proj: [[f32; 4]; 4]) -> FrameUniforms {
        FrameUniforms {
            view_proj,
            // UNVERIFIED: neutral stand-in — see above.
            ambient: [0.5, 0.5, 0.5, 1.0],
            light_dir: [0.0, 0.0, -1.0, 0.0],
            diffuse: [0.0, 0.0, 0.0, 0.0],
            backlight: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct MaterialUniforms {
    /// Non-zero enables alpha-test (cutout) in the shader.
    alpha_test: u32,
    alpha_cutoff: f32,
    _pad0: f32,
    _pad1: f32,
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

/// Bind group layout for one material: diffuse texture, sampler, and the material uniform.
pub struct ModelMaterialBindGroupLayout(BindGroupLayout);

impl ModelMaterialBindGroupLayout {
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
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }))
    }
}

pub struct ModelShader(ShaderModule);

impl ModelShader {
    pub fn new(device: &Device) -> Self {
        Self(device.create_shader_module(include_wgsl!("model.wgsl")))
    }
}

pub struct ModelPipelineLayout(PipelineLayout);

impl ModelPipelineLayout {
    pub fn new(
        device: &Device,
        frame_layout: &ModelFrameBindGroupLayout,
        material_layout: &ModelMaterialBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&frame_layout.0, &material_layout.0],
            immediate_size: 0,
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

/// One material's GPU resources: its bind group (texture + sampler + uniform) and alpha mode.
struct GpuMaterial {
    bind_group: BindGroup,
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
}

#[derive(Debug, Display, Error)]
pub enum AddModelError {
    #[display("model has no primitives")]
    NoPrimitives,
    #[display("model has no instances to place")]
    NoInstances,
}

pub struct ModelPass {
    material_layout: ModelMaterialBindGroupLayout,
    pipelines: ModelPipelines,
    sampler: wgpu::Sampler,
    /// 1x1 white texture used for materials that have no base map.
    white_view: TextureView,
    meshes: Vec<GpuModel>,
    /// One buffer for the whole pass — the static mesh shader has no per-object constants.
    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,
    /// Camera world-space position, used for depth-sorting transparent draws.
    camera_pos: [f32; 3],
}

impl ModelPass {
    pub fn new(
        device: &Device,
        queue: &Queue,
        targets: TargetFormats,
    ) -> Self {
        let shader = ModelShader::new(device);
        let frame_layout = ModelFrameBindGroupLayout::new(device);
        let material_layout = ModelMaterialBindGroupLayout::new(device);
        let layout = ModelPipelineLayout::new(device, &frame_layout, &material_layout);
        let pipelines = ModelPipelines::new(device, &layout, &shader, targets);
        // D3D9's default addressing is WRAP, and a third of the mesh library needs it: 501
        // of 1500 meshes sampled out of graphics.big carry UVs outside 0..1.
        let sampler = repeat_sampler(device, "model_sampler");
        let white_view = create_white_view(device, queue);

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
            material_layout,
            pipelines,
            sampler,
            white_view,
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
    pub fn add_model(
        &mut self,
        device: &Device,
        queue: &Queue,
        model: &Model,
        instances: &[ModelInstance],
    ) -> Result<(), AddModelError> {
        if model.primitives.is_empty() {
            return Err(AddModelError::NoPrimitives);
        }
        if instances.is_empty() {
            return Err(AddModelError::NoInstances);
        }

        let materials = self.build_materials(device, queue, &model.materials);
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

    /// Build a [`GpuMaterial`] per input material, uploading each diffuse texture (or
    /// falling back to the shared white texture) and baking its alpha mode into a uniform.
    fn build_materials(
        &self,
        device: &Device,
        queue: &Queue,
        materials_in: &[ModelMaterial],
    ) -> Vec<GpuMaterial> {
        let mut materials = Vec::with_capacity(materials_in.len());
        for material in materials_in {
            let uploaded = material
                .diffuse
                .as_ref()
                .map(|image| upload_texture(device, queue, "model_material_diffuse", image));
            let view = uploaded.as_ref().unwrap_or(&self.white_view);

            let material_uniforms = MaterialUniforms {
                alpha_test: (material.alpha_mode == AlphaMode::Cutout) as u32,
                alpha_cutoff: ALPHA_CUTOFF,
                _pad0: 0.0,
                _pad1: 0.0,
            };
            let material_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("model_material_uniform"),
                contents: bytemuck::cast_slice(&[material_uniforms]),
                usage: BufferUsages::UNIFORM,
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("model_material_bind_group"),
                layout: &self.material_layout.0,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: material_buffer.as_entire_binding(),
                    },
                ],
            });

            materials.push(GpuMaterial {
                bind_group,
                transparent: material.alpha_mode == AlphaMode::Blend,
            });
        }
        materials
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
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        if self.meshes.is_empty() {
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

        // Opaque and cutout first — they write depth, and every placement of a mesh draws
        // in one instanced call.
        for mesh in &self.meshes {
            mesh.draw_opaque(&mut rpass, &self.pipelines);
        }

        // Then the blended ones, back to front across the whole level rather than within
        // one model, so a distant transparent object cannot paint over a near one. Sorting
        // is per instance, so these draw one at a time.
        for (mesh, instance) in self.sorted_transparent_instances() {
            mesh.draw_transparent(&mut rpass, &self.pipelines, instance);
        }
    }

    /// Every transparent placement in the level, farthest first.
    fn sorted_transparent_instances(&self) -> Vec<(&GpuModel, u32)> {
        let mut draws: Vec<(&GpuModel, u32)> = self
            .meshes
            .iter()
            .filter(|m| m.has_transparent)
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
                rpass.set_bind_group(1, &material.bind_group, &[]);
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
                rpass.set_bind_group(1, &material.bind_group, &[]);
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

/// Create a 1x1 opaque-white texture view, used for materials without a diffuse map.
pub(crate) fn create_white_view(device: &Device, queue: &Queue) -> TextureView {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("model_white_fallback"),
        size: Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        &[255, 255, 255, 255],
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: None,
        },
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    texture.create_view(&TextureViewDescriptor::default())
}
