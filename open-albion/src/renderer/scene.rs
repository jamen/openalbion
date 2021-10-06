use std::borrow::Cow;
use std::convert::TryInto;
use std::fs::File;
use std::mem;
use std::num::NonZeroU64;

use crate::state::ArcballCamera;
use crate::{RendererBase, State};

use crevice::std430::{AsStd430, Std430};
use fable_data::Big;
use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{vertex_attr_array, VertexState};

pub struct SceneRenderer {
    model_pipeline: wgpu::RenderPipeline,
    wire_pipeline: wgpu::RenderPipeline,
    transform_buffer: wgpu::Buffer,
    depth_texture: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    material_bind_group_layout: wgpu::BindGroupLayout,
    aspect:
    model: Model,
}

pub struct Model {
    vector_clock: usize,
    materials: Vec<Material>,
    primitives: Vec<Primitive>,
}

/// Maybe this can be refactored so the meshes are stored in a single wgpu::Buffer and accessed with
/// wgpu::BufferSlice's.
pub struct Primitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
    count: u32,
    wire_index_buffer: wgpu::Buffer,
    material_id: usize,
}

pub struct Material {
    base_color: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

impl SceneRenderer {
    pub const VERTEX_BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        step_mode: wgpu::VertexStepMode::Vertex,
        array_stride: mem::size_of::<fable_data::Vertex>() as u64,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x3,
            2 => Float32x2
        ],
    };

    pub fn create(base: &RendererBase, state: &State) -> Self {
        let transform_buffer = base.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: mem::size_of::<glam::Mat4>().try_into().unwrap(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        Self::write_transform_buffer(&base, &transform_buffer, &state.camera);

        let depth_texture = Self::create_depth_texture(&base);

        let bind_group_layout =
            base.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(64.try_into().unwrap()),
                        },
                        count: None,
                    }],
                });

        let material_bind_group_layout =
            base.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group = base.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = base
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scene_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout, &material_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Load this renderer's shader(s)
        let shader = base
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("./scene.wgsl"))),
            });

        // Create pipeline
        let model_pipeline = base
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scene_model_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[base.surface_config.format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
            });

        // Create pipeline
        let wire_pipeline = base
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("scene_wire_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_wire",
                    buffers: &[Self::VERTEX_BUFFER_LAYOUT],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    targets: &[base.surface_config.format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
            });

        let model = Self::create_model(&base, &state, &material_bind_group_layout);

        Self {
            model_pipeline,
            wire_pipeline,
            transform_buffer,
            bind_group,
            material_bind_group_layout,
            depth_texture,
            model,
        }
    }

    pub fn resize(&mut self, base: &RendererBase) {
        self.depth_texture = Self::create_depth_texture(base);
    }

    // This is function should be replaced by UI.
    fn create_model(
        base: &RendererBase,
        state: &State,
        material_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Model {
        let entry_id = *state
            .graphics
            .entries_by_name
            .get(&state.selected_model_name)
            .unwrap();

        let entry = &state.graphics.entries[entry_id];

        let mut model_data = vec![0; entry.data_size as usize];

        fable_data::Big::read_entry(&state.graphics_file, &entry, &mut model_data)
            .expect("read model entry");

        let mut primitives = Vec::new();
        let mut materials = Vec::new();

        if let fable_data::BigInfo::Mesh(model_info) = &entry.info {
            if let Some(model) = fable_data::Model::decode(&model_data, &model_info) {
                primitives.reserve(model.primitives.len());

                log::debug!("model.name {:?}", model.name);
                log::debug!("model.animated {:?}", model.animated);
                log::debug!("model.bounding_sphere {:?}", model.bounding_sphere);
                log::debug!("model.bounding_box {:?}", model.bounding_box);
                // log::debug!("model.helper_points {:?}", model.helper_points);
                // log::debug!("model.helper_dummies {:?}", model.helper_dummies);
                // log::debug!("model.helper_point_names {:?}", model.helper_point_names);
                // log::debug!("model.helper_dummy_names {:?}", model.helper_dummy_names);
                log::debug!("model.material_count {:?}", model.material_count);
                log::debug!("model.primitive_count {:?}", model.primitive_count);
                // log::debug!("model.bone_names {:?}", model.bone_names);
                // log::debug!("model.bones {:?}", model.bones);
                // log::debug!("model.bone_keyframes {:?}", model.bone_keyframes);
                // log::debug!("model.bone_transforms {:?}", model.bone_transforms);
                log::debug!("model.cloth {:?}", model.cloth);
                log::debug!("model.unknown4 {:?}", model.unknown4);
                log::debug!("model.unknown5 {:?}", model.unknown5);
                log::debug!("model.transform_matrix {:?}", model.transform_matrix);
                log::debug!("model.materials {:#?}", model.materials);

                for (i, material) in model.materials.iter().enumerate() {
                    if material.degenerate_triangles > 0 {
                        continue;
                    }

                    let texture_entry = state
                        .textures
                        .entries
                        .iter()
                        .find(|entry| entry.id == material.base_texture_id)
                        .expect("texture not found");

                    let mut texture_data = vec![0; texture_entry.data_size as usize];

                    fable_data::Big::read_entry(
                        &state.textures_file,
                        &texture_entry,
                        &mut texture_data,
                    )
                    .expect("read texture entry");

                    log::debug!("texture_entry.info {:#?}", texture_entry.info);

                    if let fable_data::BigInfo::Texture(texture_info) = &texture_entry.info {
                        if let Some(texture) =
                            fable_data::Texture::decode(&texture_data, &texture_info)
                        {
                            let base_color = base.device.create_texture_with_data(
                                &base.queue,
                                &wgpu::TextureDescriptor {
                                    label: None,
                                    size: wgpu::Extent3d {
                                        width: texture_info.frame_width as u32,
                                        height: texture_info.frame_height as u32,
                                        depth_or_array_layers: 1,
                                    },
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    // TODO: This is probably broken
                                    format: match texture.dxt_compression {
                                        31 => wgpu::TextureFormat::Bc1RgbaUnorm,
                                        32 => wgpu::TextureFormat::Bc2RgbaUnorm,
                                        _ => wgpu::TextureFormat::Bc2RgbaUnorm,
                                    },
                                    usage: wgpu::TextureUsages::COPY_DST
                                        | wgpu::TextureUsages::TEXTURE_BINDING,
                                },
                                &texture.frames[0][..],
                            );

                            let base_color_view = base_color.create_view(&Default::default());

                            let base_color_sampler =
                                base.device.create_sampler(&wgpu::SamplerDescriptor {
                                    address_mode_u: wgpu::AddressMode::Repeat,
                                    address_mode_v: wgpu::AddressMode::Repeat,
                                    address_mode_w: wgpu::AddressMode::Repeat,
                                    mag_filter: wgpu::FilterMode::Nearest,
                                    min_filter: wgpu::FilterMode::Linear,
                                    mipmap_filter: wgpu::FilterMode::Nearest,
                                    ..Default::default()
                                });

                            let bind_group =
                                base.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                    label: None,
                                    layout: &material_bind_group_layout,
                                    entries: &[
                                        wgpu::BindGroupEntry {
                                            binding: 0,
                                            resource: wgpu::BindingResource::TextureView(
                                                &base_color_view,
                                            ),
                                        },
                                        wgpu::BindGroupEntry {
                                            binding: 1,
                                            resource: wgpu::BindingResource::Sampler(
                                                &base_color_sampler,
                                            ),
                                        },
                                    ],
                                });

                            materials.push(Material {
                                base_color,
                                bind_group,
                            });
                        } else {
                            panic!("texture parsing failed");
                        }
                    } else {
                        panic!("texture info missing");
                    }
                }

                for (i, primitive) in model.primitives.iter().enumerate() {
                    log::debug!(
                        "primitive[{:?}].material_index {:?}",
                        i,
                        primitive.material_index
                    );
                    log::debug!(
                        "primitive[{:?}].repeating_mesh_reps {:?}",
                        i,
                        primitive.repeating_mesh_reps
                    );
                    log::debug!(
                        "primitive[{:?}].bounding_sphere {:?}",
                        i,
                        primitive.bounding_sphere
                    );
                    log::debug!(
                        "primitive[{:?}].average_texture_stretch {:?}",
                        i,
                        primitive.average_texture_stretch
                    );
                    log::debug!(
                        "primitive[{:?}].vertex_count {:?}",
                        i,
                        primitive.vertex_count
                    );
                    log::debug!(
                        "primitive[{:?}].triangle_count {:?}",
                        i,
                        primitive.triangle_count
                    );
                    log::debug!("primitive[{:?}].index_count {:?}", i, primitive.index_count);
                    log::debug!("primitive[{:?}].init_flags {:?}", i, primitive.init_flags);
                    log::debug!(
                        "primitive[{:?}].static_block_count {:?}",
                        i,
                        primitive.static_block_count
                    );
                    log::debug!(
                        "primitive[{:?}].animated_block_count {:?}",
                        i,
                        primitive.animated_block_count
                    );
                    log::debug!(
                        "primitive[{:?}].static_blocks {:#?}",
                        i,
                        primitive.static_blocks
                    );
                    log::debug!(
                        "primitive[{:?}].animated_blocks {:#?}",
                        i,
                        primitive.animated_blocks
                    );
                    log::debug!("primitive[{:?}].pos_bias {:?}", i, primitive.pos_bias);
                    log::debug!("primitive[{:?}].pos_scale {:?}", i, primitive.pos_scale);
                    log::debug!("primitive[{:?}].vertex_size {:?}", i, primitive.vertex_size);
                    log::debug!("primitive[{:?}].padding {:?}", i, primitive.padding);
                    log::debug!(
                        "primitive[{:?}].vertices.len() = {:?}",
                        i,
                        primitive.vertices.len()
                    );
                    log::debug!(
                        "primitive[{:?}].indices.len() = {:?}",
                        i,
                        primitive.indices.len()
                    );

                    // log::debug!("primitive[{:?}].indices = {:?}", i, primitive.indices);

                    let vertex_buffer = base.device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&primitive.vertices),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    });

                    let index_buffer = if primitive.indices.len() > 0 {
                        Some(base.device.create_buffer_init(&BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&primitive.indices),
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                        }))
                    } else {
                        None
                    };

                    let mut wire_indices: Vec<u16> =
                        Vec::with_capacity(primitive.indices.len() * 2);

                    for (i, chunk) in primitive.indices.chunks(3).enumerate() {
                        if let &[i1, i2, i3] = chunk {
                            wire_indices.extend_from_slice(&[i1, i2, i1, i2, i2, i3]);
                        }
                    }

                    let wire_index_buffer = base.device.create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&wire_indices),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                    });

                    primitives.push(Primitive {
                        vertex_buffer,
                        index_buffer,
                        count: primitive.index_count,
                        material_id: primitive.material_index as usize,
                        wire_index_buffer,
                    });
                }
            }
        }

        Model {
            vector_clock: state.model_vector_clock,
            materials,
            primitives,
        }
    }

    pub fn create_depth_texture(base: &RendererBase) -> wgpu::TextureView {
        let depth_texture = base.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: base.surface_config.width,
                height: base.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn write_transform_buffer(
        base: &RendererBase,
        buffer: &wgpu::Buffer,
        camera: &ArcballCamera,
    ) {
        let camera_pos = camera.rotation * Vec3::Z * camera.distance + camera.focus;

        let width = base.surface_config.width as f32;
        let height = base.surface_config.height as f32;

        let aspect = width / height;
        let fov = 90.0f32;
        let z_near = 0.05f32;

        let proj = Mat4::perspective_infinite_lh(fov.to_radians(), aspect, z_near);
        let look_at = Mat4::look_at_lh(camera_pos, camera.focus, Vec3::Y);

        let transform = proj * look_at;
        let transform: mint::ColumnMatrix4<f32> = transform.into();
        let transform = transform.as_std430();

        base.queue.write_buffer(buffer, 0, transform.as_bytes());
    }

    pub fn render(
        &mut self,
        base: &RendererBase,
        view: &wgpu::TextureView,
        state: &State,
    ) -> wgpu::CommandBuffer {
        if self.model.vector_clock != state.model_vector_clock {
            self.model = Self::create_model(&base, &state, &self.material_bind_group_layout);
        }

        let mut encoder = base
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Self::write_transform_buffer(&base, &self.transform_buffer, &state.camera);

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            rpass.set_pipeline(&self.model_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            for primitive in &self.model.primitives {
                if let Some(material) = &self.model.materials.get(primitive.material_id) {
                    rpass.set_bind_group(1, &material.bind_group, &[]);
                    rpass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
                    if let Some(index_buffer) = &primitive.index_buffer {
                        rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        rpass.draw_indexed(0..primitive.count, 0, 0..1);
                    } else {
                        rpass.draw(0..primitive.count, 0..1);
                    }
                }
            }

            if state.wireframe {
                rpass.set_pipeline(&self.wire_pipeline);
                rpass.set_bind_group(0, &self.bind_group, &[]);

                for primitive in &self.model.primitives {
                    rpass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
                    rpass.set_index_buffer(
                        primitive.wire_index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    rpass.draw_indexed(0..primitive.count * 2, 0, 0..1);
                }
            }
        }

        encoder.finish()
    }
}
