//! Renders local detail's repeated meshes: grass, bracken, brambles, flowers.
//!
//! The other half of local detail — the objects the engine builds as plain static meshes —
//! goes through [`crate::model::ModelPass`] instead, because that is literally the call the
//! engine makes for them. Only `LOCAL_DETAIL_PRIMITIVE_TYPE_REPEATED_MESH` comes here, and it
//! earns a pass of its own: `SHADERS_REPEATED_MESH` places an object with a Z rotation and a
//! Z scale rather than a matrix, and lights it with one colour per object taken from the
//! ground normal. See `local_detail.wgsl` for the transcription.
//!
//! It is also where the count is: a level grows a few hundred static-mesh objects and tens of
//! thousands of repeated ones. A batch is one mesh asset uploaded once with every instance of
//! it in a single buffer, so LookoutPoint's ~15,700 grass tufts are a handful of draws rather
//! than fifteen thousand.

use crate::TargetFormats;
use crate::image::TextureImage;
use crate::model::{Model, ModelVertex};
use crate::lighting::{LIGHTING_WGSL, LightingUniforms};
use crate::texture::{repeat_sampler, upload_texture};
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Face, FragmentState, FrontFace, IndexFormat, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModule, ShaderStages, StencilState, TextureSampleType, TextureView,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

/// One placement of a repeated mesh — `ObjectMatricies[i]`, `ObjectOffsets[i]` and
/// `LightingResults[i]`, which the original delivers as three vertex constants per instance.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LocalDetailInstance {
    /// `c[a + 19]` — `(cos(angle) * scale, sin(angle) * scale, wind, wind)`.
    ///
    /// The last two are the wind skew and stay zero until wind animation lands;
    /// `BuildFromSourceMeshes` writes them as zero too and `SetupWindAnimation` fills them in
    /// afterwards.
    pub rotation: [f32; 4],
    /// `c[a + 35]` — `(x, y, z, scale)`.
    pub offset: [f32; 4],
    /// The landscape normal under the object, which `LightingResults[i]` is computed from —
    /// see the DIVERGENCE note in `local_detail.wgsl`.
    pub ground_normal: [f32; 4],
}

impl LocalDetailInstance {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Default for LocalDetailInstance {
    fn default() -> Self {
        LocalDetailInstance {
            rotation: [1.0, 0.0, 0.0, 0.0],
            offset: [0.0, 0.0, 0.0, 1.0],
            ground_normal: [0.0, 0.0, 1.0, 0.0],
        }
    }
}

/// The per-frame shader constants, by their register names in the Lights layout (§3.8).
/// Identical to the static mesh pass's, and for the same reason: one environment supplies
/// every pass that lights (AGENTS.md step 2).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct FrameUniforms {
    /// `c5..c8`
    view_proj: [[f32; 4]; 4],
    /// `c3`/`c19`/`c20`/`c35`, shared with the landscape and the static meshes — and shared
    /// by construction here, since `CalcSWLightingNoClip` is `VSHADER_STATIC_DIRLIGHT`'s own
    /// expression over these registers (AGENTS.md §3.13).
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct MaterialUniforms {
    alpha_cutoff: f32,
    _pad: [f32; 3],
}

/// A mesh asset and every repeated-mesh placement of it.
struct GpuBatch {
    materials: Vec<BindGroup>,
    primitives: Vec<GpuPrimitive>,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

struct GpuPrimitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sub_meshes: Vec<SubMeshDraw>,
}

struct SubMeshDraw {
    material: usize,
    index_start: u32,
    index_count: u32,
    cull: bool,
}

#[derive(Debug, Display, Error)]
pub enum AddLocalDetailError {
    #[display("model has no primitives")]
    NoPrimitives,
    #[display("no instances to place")]
    NoInstances,
}

pub struct LocalDetailPass {
    material_layout: BindGroupLayout,
    culled: RenderPipeline,
    unculled: RenderPipeline,
    sampler: wgpu::Sampler,
    white_view: TextureView,
    batches: Vec<GpuBatch>,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: BindGroup,
}

impl LocalDetailPass {
    pub fn new(device: &Device, queue: &Queue, targets: TargetFormats) -> Self {
        let shader: ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("local_detail.wgsl"),
            source: wgpu::ShaderSource::Wgsl(
                format!("{LIGHTING_WGSL}\n{}", include_str!("local_detail.wgsl")).into(),
            ),
        });

        let frame_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("local_detail_frame_layout"),
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

        let material_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("local_detail_material_layout"),
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
        });

        let layout: PipelineLayout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&frame_layout, &material_layout],
            immediate_size: 0,
        });

        // One pipeline shape, in a culled and an unculled variant. There is no blended
        // variant: the pass alpha-tests at the object type's `AlphaRef` and writes depth, so
        // nothing needs sorting — which is what makes tens of thousands of instances cheap.
        let make = |cull: bool| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(type_name::<Self>()),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[ModelVertex::layout(), LocalDetailInstance::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: targets.colour,
                        blend: None,
                        write_mask: ColorWrites::COLOR,
                    })],
                }),
                primitive: PrimitiveState {
                    // Clockwise-front, like every other Fable mesh (AGENTS.md §3.11).
                    front_face: FrontFace::Cw,
                    cull_mode: cull.then_some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: targets.depth,
                    depth_write_enabled: true,
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

        let frame_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("local_detail_frame_buffer"),
            contents: bytemuck::cast_slice(&[FrameUniforms::new(
                glam::Mat4::IDENTITY.to_cols_array_2d(),
            )]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("local_detail_frame_bind_group"),
            layout: &frame_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });

        Self {
            material_layout,
            culled: make(true),
            unculled: make(false),
            // D3D9 addresses WRAP by default, and a third of the mesh library needs it.
            sampler: repeat_sampler(device, "local_detail_sampler"),
            white_view: crate::model::create_white_view(device, queue),
            batches: Vec::new(),
            frame_buffer,
            frame_bind_group,
        }
    }

    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Upload one mesh asset and every repeated-mesh placement of it.
    ///
    /// `alpha_cutoff` is the object type's `AlphaRef / 255`. It belongs to the object type
    /// rather than the material — the engine passes it per `CAddMeshDesc` — so a mesh shared
    /// by two object types with different refs is two batches, not one.
    pub fn add_batch(
        &mut self,
        device: &Device,
        queue: &Queue,
        model: &Model,
        instances: &[LocalDetailInstance],
        alpha_cutoff: f32,
    ) -> Result<(), AddLocalDetailError> {
        if model.primitives.is_empty() {
            return Err(AddLocalDetailError::NoPrimitives);
        }
        if instances.is_empty() {
            return Err(AddLocalDetailError::NoInstances);
        }

        let materials = model
            .materials
            .iter()
            .map(|material| {
                let uploaded = material.diffuse.as_ref().map(|image: &TextureImage| {
                    upload_texture(device, queue, "local_detail_diffuse", image)
                });
                let view = uploaded.as_ref().unwrap_or(&self.white_view);

                let material_buffer = device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("local_detail_material_uniform"),
                    contents: bytemuck::cast_slice(&[MaterialUniforms {
                        alpha_cutoff,
                        _pad: [0.0; 3],
                    }]),
                    usage: BufferUsages::UNIFORM,
                });

                device.create_bind_group(&BindGroupDescriptor {
                    label: Some("local_detail_material_bind_group"),
                    layout: &self.material_layout,
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
                })
            })
            .collect();

        let primitives = model
            .primitives
            .iter()
            .map(|primitive| GpuPrimitive {
                vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("local_detail_vertex_buffer"),
                    contents: bytemuck::cast_slice(&primitive.vertices),
                    usage: BufferUsages::VERTEX,
                }),
                index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("local_detail_index_buffer"),
                    contents: bytemuck::cast_slice(&primitive.indices),
                    usage: BufferUsages::INDEX,
                }),
                sub_meshes: primitive
                    .sub_meshes
                    .iter()
                    .map(|sub| SubMeshDraw {
                        material: sub.material as usize,
                        index_start: sub.index_start,
                        index_count: sub.index_count,
                        cull: !model
                            .materials
                            .get(sub.material as usize)
                            .is_some_and(|m| m.two_sided),
                    })
                    .collect(),
            })
            .collect();

        self.batches.push(GpuBatch {
            materials,
            primitives,
            instance_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("local_detail_instance_buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: BufferUsages::VERTEX,
            }),
            instance_count: instances.len() as u32,
        });
        Ok(())
    }

    /// `(mesh assets uploaded, instances drawn)`.
    pub fn stats(&self) -> (usize, usize) {
        (
            self.batches.len(),
            self.batches.iter().map(|b| b.instance_count as usize).sum(),
        )
    }

    pub fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        queue.write_buffer(
            &self.frame_buffer,
            0,
            bytemuck::cast_slice(&[FrameUniforms::new(view_proj)]),
        );
    }

    pub fn pass(
        &self,
        cmd: &mut CommandEncoder,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        if self.batches.is_empty() {
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

        for batch in &self.batches {
            rpass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            for primitive in &batch.primitives {
                rpass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
                rpass.set_index_buffer(primitive.index_buffer.slice(..), IndexFormat::Uint16);
                for sub in &primitive.sub_meshes {
                    let Some(material) = batch.materials.get(sub.material) else {
                        continue;
                    };
                    rpass.set_pipeline(if sub.cull {
                        &self.culled
                    } else {
                        &self.unculled
                    });
                    rpass.set_bind_group(1, material, &[]);
                    rpass.draw_indexed(
                        sub.index_start..sub.index_start + sub.index_count,
                        0,
                        0..batch.instance_count,
                    );
                }
            }
        }
    }
}
