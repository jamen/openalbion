//! Renders a Fable mesh: every primitive, every per-material draw range, with the material's
//! diffuse texture and alpha mode (opaque / alpha-test cutout / alpha-blended).
//!
//! Backface culling is intentionally left off: the historical decoder doesn't flip triangle-strip
//! winding, so `two_sided` can't be honoured reliably until that's fixed (see `mesh::expand_block`).

use super::texture::{TextureUploadError, linear_clamp_sampler, upload_texture};
use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use fable_data::{big::AssetMetadata, mesh::Mesh};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoder, CompareFunction, DepthBiasState,
    DepthStencilState, Device, Extent3d, FragmentState, IndexFormat, MultisampleState,
    PipelineLayout, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModule, ShaderStages, StencilState,
    TexelCopyBufferLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Texels with alpha below this are discarded by alpha-test (cutout) materials.
const ALPHA_CUTOFF: f32 = 0.5;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct MeshVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl MeshVertex {
    const ATTRIBS: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

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
struct ModelUniforms {
    view_proj: [[f32; 4]; 4],
    model_scale: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    model_pos: [f32; 3],
    _pad3: f32,
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

pub struct ModelUniformBindGroupLayout(BindGroupLayout);

impl ModelUniformBindGroupLayout {
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
        uniform_layout: &ModelUniformBindGroupLayout,
        material_layout: &ModelMaterialBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, &material_layout.0],
            immediate_size: 0,
        }))
    }
}

/// The opaque and alpha-blended pipeline variants. Cutout materials use the opaque pipeline and
/// discard in the shader; blended materials use the blend pipeline (depth test on, depth write off).
struct ModelPipelines {
    opaque: RenderPipeline,
    blend: RenderPipeline,
}

impl ModelPipelines {
    fn new(
        device: &Device,
        layout: &ModelPipelineLayout,
        shader: &ModelShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        let make = |blend: bool| {
            let color_target = ColorTargetState {
                format: target_format,
                blend: blend.then_some(BlendState::ALPHA_BLENDING),
                // Opaque draws leave the surface alpha alone (so an opaque material with an alpha
                // channel can't make the window see-through); blended draws need it.
                write_mask: if blend { ColorWrites::ALL } else { ColorWrites::COLOR },
            };
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(type_name::<Self>()),
                layout: Some(&layout.0),
                vertex: VertexState {
                    module: &shader.0,
                    entry_point: Some("vs_main"),
                    buffers: &[MeshVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &shader.0,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(color_target)],
                }),
                primitive: PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: depth_format,
                    // Transparent draws test against opaque depth but don't write, so they don't
                    // occlude each other order-dependently.
                    depth_write_enabled: !blend,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };

        Self {
            opaque: make(false),
            blend: make(true),
        }
    }
}

/// One material's GPU resources: its bind group (texture + sampler + uniform) and alpha mode.
struct ModelMaterial {
    bind_group: BindGroup,
    transparent: bool,
}

/// One draw range within a primitive's index buffer, resolved to a material slot.
struct SubMeshDraw {
    material: usize,
    index_start: u32,
    index_count: u32,
}

/// One primitive's uploaded geometry plus its per-material sub-draws.
struct ModelPrimitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    sub_meshes: Vec<SubMeshDraw>,
}

/// A fully uploaded model: shared transform uniform, its materials, and its primitives.
struct ModelMesh {
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
    materials: Vec<ModelMaterial>,
    primitives: Vec<ModelPrimitive>,
    model_scale: f32,
    model_pos: [f32; 3],
}

impl ModelMesh {
    fn update_uniforms(&self, queue: &Queue, view_proj: [[f32; 4]; 4]) {
        let uniforms = ModelUniforms {
            view_proj,
            model_scale: self.model_scale,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            model_pos: self.model_pos,
            _pad3: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

#[derive(Debug, Display, Error)]
pub enum ModelTextureError {
    #[display("mesh has no primitives")]
    NoPrimitives,
    #[display("{_0}")]
    Texture(TextureUploadError),
}

pub struct ModelPass {
    uniform_layout: ModelUniformBindGroupLayout,
    material_layout: ModelMaterialBindGroupLayout,
    pipelines: ModelPipelines,
    sampler: wgpu::Sampler,
    /// 1x1 white texture used for materials that have no diffuse map.
    white_view: TextureView,
    mesh: Option<ModelMesh>,
}

impl ModelPass {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        let shader = ModelShader::new(device);
        let uniform_layout = ModelUniformBindGroupLayout::new(device);
        let material_layout = ModelMaterialBindGroupLayout::new(device);
        let layout = ModelPipelineLayout::new(device, &uniform_layout, &material_layout);
        let pipelines = ModelPipelines::new(device, &layout, &shader, surface_format, depth_format);
        let sampler = linear_clamp_sampler(device, "model_sampler");
        let white_view = create_white_view(device, queue);

        Self {
            uniform_layout,
            material_layout,
            pipelines,
            sampler,
            white_view,
            mesh: None,
        }
    }

    pub fn set_model(
        &mut self,
        device: &Device,
        queue: &Queue,
        mesh: &Mesh,
        material_textures: &[Option<(AssetMetadata, Vec<u8>)>],
    ) -> Result<(), ModelTextureError> {
        use ModelTextureError as E;

        if mesh.primitives.is_empty() {
            return Err(E::NoPrimitives);
        }

        // Place the model at the terrain centre with a small scale (provisional framing).
        let model_scale = 0.05;
        let model_pos = [32.0, 16.0, 32.0];

        let uniforms = ModelUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            model_scale,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            model_pos,
            _pad3: 0.0,
        };
        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model_uniform_bind_group"),
            layout: &self.uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let materials = self.build_materials(device, queue, mesh, material_textures)?;
        let primitives = build_primitives(device, mesh);

        let opaque = primitives
            .iter()
            .flat_map(|p| &p.sub_meshes)
            .filter(|s| matches!(materials.get(s.material), Some(m) if !m.transparent))
            .count();
        tracing::info!(
            "Model: {} primitives, {} materials ({} opaque/cutout + {} transparent sub-draws)",
            primitives.len(),
            materials.len(),
            opaque,
            primitives.iter().map(|p| p.sub_meshes.len()).sum::<usize>() - opaque,
        );

        self.mesh = Some(ModelMesh {
            uniform_buffer,
            uniform_bind_group,
            materials,
            primitives,
            model_scale,
            model_pos,
        });
        Ok(())
    }

    /// Build a [`ModelMaterial`] per `mesh.materials` entry, uploading each diffuse texture (or
    /// falling back to the shared white texture) and baking its alpha mode into a uniform.
    fn build_materials(
        &self,
        device: &Device,
        queue: &Queue,
        mesh: &Mesh,
        material_textures: &[Option<(AssetMetadata, Vec<u8>)>],
    ) -> Result<Vec<ModelMaterial>, ModelTextureError> {
        use ModelTextureError as E;

        let mut materials = Vec::with_capacity(mesh.materials.len());
        for (i, material) in mesh.materials.iter().enumerate() {
            let uploaded = match material_textures.get(i).and_then(|t| t.as_ref()) {
                Some((meta, data)) => Some(upload_texture(device, queue, meta, data).map_err(E::Texture)?),
                None => None,
            };
            let view = uploaded.as_ref().unwrap_or(&self.white_view);

            let material_uniforms = MaterialUniforms {
                alpha_test: material.boolean_alpha as u32,
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

            materials.push(ModelMaterial {
                bind_group,
                // A cutout (boolean-alpha) material draws in the opaque pass with shader discard;
                // only fully translucent materials go through the blend pass.
                transparent: material.transparent && !material.boolean_alpha,
            });
        }
        Ok(materials)
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
            tracing::debug!("ModelPass: no mesh set — model skipped");
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
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        rpass.set_bind_group(0, &mesh.uniform_bind_group, &[]);

        // Opaque + cutout first (they write depth), then transparent over the top.
        rpass.set_pipeline(&self.pipelines.opaque);
        mesh.draw(&mut rpass, false);
        rpass.set_pipeline(&self.pipelines.blend);
        mesh.draw(&mut rpass, true);
    }
}

impl ModelMesh {
    /// Draw every sub-mesh whose material's `transparent` flag matches `transparent`.
    fn draw(&self, rpass: &mut wgpu::RenderPass<'_>, transparent: bool) {
        for primitive in &self.primitives {
            let mut bound = false;
            for sub in &primitive.sub_meshes {
                let Some(material) = self.materials.get(sub.material) else {
                    continue;
                };
                if material.transparent != transparent {
                    continue;
                }
                if !bound {
                    rpass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
                    rpass.set_index_buffer(primitive.index_buffer.slice(..), IndexFormat::Uint16);
                    bound = true;
                }
                rpass.set_bind_group(1, &material.bind_group, &[]);
                let end = sub.index_start + sub.index_count;
                rpass.draw_indexed(sub.index_start..end, 0, 0..1);
            }
        }
    }
}

/// Upload every primitive's geometry and resolve its sub-mesh draw ranges. Empty primitives
/// (no vertices or no indices) are skipped so we never create a zero-sized GPU buffer.
fn build_primitives(device: &Device, mesh: &Mesh) -> Vec<ModelPrimitive> {
    mesh.primitives
        .iter()
        .filter(|p| !p.vertices.is_empty() && !p.indices.is_empty())
        .map(|primitive| {
            let vertices: Vec<MeshVertex> = primitive
                .vertices
                .iter()
                .map(|v| MeshVertex {
                    position: v.pos,
                    normal: v.normal,
                    uv: v.uv,
                })
                .collect();

            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("model_vertex_buffer"),
                contents: bytemuck::cast_slice(&vertices),
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
                    material: s.material_index as usize,
                    index_start: s.index_start,
                    index_count: s.index_count,
                })
                .collect();

            ModelPrimitive {
                vertex_buffer,
                index_buffer,
                sub_meshes,
            }
        })
        .collect()
}

/// Create a 1x1 opaque-white texture view, used for materials without a diffuse map.
fn create_white_view(device: &Device, queue: &Queue) -> TextureView {
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
