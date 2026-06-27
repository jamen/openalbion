use bytemuck::{Pod, Zeroable};
use derive_more::{Display, Error};
use fable_data::{
    big::{AssetMetadata, ExtraMetadata},
    mesh::Mesh,
    texture::{BcnEncoding, Texture, TextureError},
};
use std::any::type_name;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device, Extent3d,
    FragmentState, IndexFormat, MultisampleState, PipelineLayout, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderModule, ShaderStages, StencilState, TexelCopyBufferLayout,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

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

pub struct ModelTextureBindGroupLayout(BindGroupLayout);

impl ModelTextureBindGroupLayout {
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
        texture_layout: &ModelTextureBindGroupLayout,
    ) -> Self {
        Self(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(type_name::<Self>()),
            bind_group_layouts: &[&uniform_layout.0, &texture_layout.0],
            immediate_size: 0,
        }))
    }
}

pub struct ModelPipeline(RenderPipeline);

impl ModelPipeline {
    pub fn new(
        device: &Device,
        layout: &ModelPipelineLayout,
        shader: &ModelShader,
        target_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        Self(device.create_render_pipeline(&RenderPipelineDescriptor {
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
                targets: &[Some(target_format.into())],
            }),
            primitive: PrimitiveState {
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

/// The uploaded mesh and its uniform buffer/bind group.
struct ModelMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
    texture: wgpu::Texture,
    texture_view: TextureView,
    sampler: wgpu::Sampler,
    texture_bind_group: BindGroup,
    model_scale: f32,
    model_pos: [f32; 3],
}

fn dxt_to_bcn_encoding(dxt: u16) -> BcnEncoding {
    match dxt {
        1 | 31 | 33 => BcnEncoding::Bc1,
        3 | 32 | 34 => BcnEncoding::Bc2,
        5 | 35 => BcnEncoding::Bc3,
        _ => BcnEncoding::Bc1,
    }
}

fn dxt_to_texture_format(dxt: u16) -> Option<TextureFormat> {
    match dxt {
        1 | 31 | 33 => Some(TextureFormat::Bc1RgbaUnorm),
        3 | 32 | 34 => Some(TextureFormat::Bc2RgbaUnorm),
        5 | 35 => Some(TextureFormat::Bc3RgbaUnorm),
        _ => None,
    }
}

impl ModelMesh {
    fn new(
        device: &Device,
        queue: &Queue,
        uniform_layout: &ModelUniformBindGroupLayout,
        texture_layout: &ModelTextureBindGroupLayout,
        mesh: &Mesh,
        texture_data: &[u8],
        texture_asset: &AssetMetadata,
    ) -> Result<Self, ModelTextureError> {
        use ModelTextureError as E;

        let primitive = mesh.primitives.first().ok_or(E::NoPrimitives)?;

        let vertices: Vec<MeshVertex> = primitive
            .vertices
            .iter()
            .map(|v| MeshVertex {
                position: v.pos,
                normal: v.normal,
                uv: v.uv,
            })
            .collect();

        // Validate indices: warn if any index >= vertex count (out of bounds)
        let max_idx = primitive.indices.iter().max().copied().unwrap_or(0);
        if max_idx as usize >= vertices.len() {
            tracing::warn!(
                "ModelMesh: index {} >= vertex count {} — out of bounds! Mesh will render wrong.",
                max_idx, vertices.len()
            );
        }
        // Check for NaN/infinity in first few vertices
        for (i, v) in vertices.iter().take(10).enumerate() {
            let pos = v.position;
            if pos.iter().any(|f| f.is_nan() || f.is_infinite()) {
                tracing::warn!("ModelMesh: vertex {} has NaN/infinity position: {:?}", i, pos);
            }
        }

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

        // Place model at terrain center with a small scale
        let scale = 0.05;
        let pos = [32.0, 16.0, 32.0];

        let uniforms = ModelUniforms {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
            model_scale: scale,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
            model_pos: pos,
            _pad3: 0.0,
        };

        tracing::info!(
            "ModelMesh: {} verts, {} indices, scale={:.3}, world_pos=({:.1},{:.1},{:.1})",
            vertices.len(),
            primitive.indices.len(),
            scale,
            pos[0], pos[1], pos[2],
        );

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model_uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model_uniform_bind_group"),
            layout: &uniform_layout.0,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let texture_extras = match &texture_asset.extras {
            Some(ExtraMetadata::Texture(extras)) => extras,
            _ => return Err(E::NotATexture),
        };

        let width = texture_extras.width as u32;
        let height = texture_extras.height as u32;
        let dxt = texture_extras.dxt_compression;

        let mut input = texture_data;
        let parsed = Texture::parse(
            &mut input,
            width as usize,
            height as usize,
            texture_extras.depth as usize,
            texture_extras.top_mip_map_size as usize,
            dxt_to_bcn_encoding(dxt),
        )
        .map_err(E::Parse)?;

        let Some(format) = dxt_to_texture_format(dxt) else {
            return Err(E::UnsupportedDxtFormat(dxt));
        };
        let bcn_data = parsed.get_top_mip_bcn_image().map_err(E::Parse)?.to_vec();
        let block_size = match format {
            TextureFormat::Bc1RgbaUnorm => 8,
            TextureFormat::Bc2RgbaUnorm | TextureFormat::Bc3RgbaUnorm => 16,
            _ => unreachable!(),
        };
        let bytes_per_row = width.div_ceil(4) * block_size;

        let texture = device.create_texture(&TextureDescriptor {
            label: Some(&texture_asset.symbol_name),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &bcn_data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("model_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model_texture_bind_group"),
            layout: &texture_layout.0,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: primitive.indices.len() as u32,
            uniform_buffer,
            uniform_bind_group,
            texture,
            texture_view,
            sampler,
            texture_bind_group,
            model_scale: scale,
            model_pos: pos,
        })
    }

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
    #[display("No primitives in mesh")]
    NoPrimitives,
    #[display("Asset is not a texture")]
    NotATexture,
    #[display("Unsupported DXT format {_0}")]
    UnsupportedDxtFormat(#[error(not(source))] u16),
    #[display("Texture parse error: {_0}")]
    Parse(TextureError),
}

pub struct ModelPass {
    uniform_layout: ModelUniformBindGroupLayout,
    texture_layout: ModelTextureBindGroupLayout,
    pipeline: ModelPipeline,
    mesh: Option<ModelMesh>,
}

impl ModelPass {
    pub fn new(device: &Device, surface_format: TextureFormat, depth_format: TextureFormat) -> Self {
        let shader = ModelShader::new(device);
        let uniform_layout = ModelUniformBindGroupLayout::new(device);
        let texture_layout = ModelTextureBindGroupLayout::new(device);
        let layout = ModelPipelineLayout::new(device, &uniform_layout, &texture_layout);
        let pipeline = ModelPipeline::new(device, &layout, &shader, surface_format, depth_format);

        Self {
            uniform_layout,
            texture_layout,
            pipeline,
            mesh: None,
        }
    }

    pub fn set_model(
        &mut self,
        device: &Device,
        queue: &Queue,
        mesh: &Mesh,
        texture_data: &[u8],
        texture_asset: &AssetMetadata,
    ) -> Result<(), ModelTextureError> {
        self.mesh = Some(ModelMesh::new(
            device,
            queue,
            &self.uniform_layout,
            &self.texture_layout,
            mesh,
            texture_data,
            texture_asset,
        )?);
        Ok(())
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

        rpass.set_pipeline(&self.pipeline.0);
        rpass.set_bind_group(0, &mesh.uniform_bind_group, &[]);
        rpass.set_bind_group(1, &mesh.texture_bind_group, &[]);
        rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        rpass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);
        rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }
}
