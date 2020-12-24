
use std::mem;
use std::borrow::Cow;

use winit::window::Window;
use winit::dpi::PhysicalSize;

use wgpu::util::DeviceExt;

use glam::{Vec3,Mat4};

use super::App;

macro_rules! shader_module {
    ($d:expr, $( $in:tt )*) => {
        $d.create_shader_module(
            wgpu::ShaderModuleSource::SpirV(
                Cow::from(
                    &vk_shader_macros::include_glsl!($( $in )*)[..]
                )
            )
        )
    }
}

pub struct Renderer {
    pub window: Window,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub camera_buffer: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub size: PhysicalSize<u32>,
}

#[derive(Clone,Copy)]
#[repr(C)]
struct Vertex (f32, f32, f32, f32);

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        // TODO: Should I start an async executor here instead of block?

        let adapter_desc = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        };

        let adapter = smol::block_on(instance.request_adapter(&adapter_desc)).unwrap();

        let device_desc = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: false,
        };

        let (device, queue) = smol::block_on(adapter.request_device(&device_desc, None)).unwrap();

        let swap_chain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

        // TODO: Create imgui here?

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<Mat4>() as wgpu::BufferAddress),
                    },
                    count: None,
                }
            ]
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let camera =
            Mat4::perspective_infinite_lh(90f32.to_radians(), size.width as f32 / size.height as f32, 0.0) *
            Mat4::look_at_lh(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), Vec3::unit_z());

        let camera_buffer_desc = wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(camera.as_ref()),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST
        };

        let camera_buffer = device.create_buffer_init(&camera_buffer_desc);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(camera_buffer.slice(..)),
                },
            ],
            label: None,
        });

        // TODO: Improve how this all works.

        let vertex_shader_module = shader_module!(device, "shaders/shader.vert", kind: vert);
        let fragment_shader_module = shader_module!(device, "shaders/shader.frag", kind: frag);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_shader_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fragment_shader_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: swap_chain_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Subtract,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float4]
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            window,
            device,
            queue,
            swap_chain,
            camera_buffer,
            pipeline,
            bind_group,
            size,
        }
    }

    pub fn render(&mut self, _state: &App) {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Dropped frame. {}", e);
                return
            }
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let eye = Vec3::new(40.0, 40.0, 40.0);
        let focus = Vec3::new(40.0, 40.0, 40.0);

        let next_camera =
            Mat4::perspective_infinite_lh(90f32.to_radians(), self.size.width as f32 / self.size.height as f32, 0.0) *
            Mat4::look_at_lh(eye, focus, Vec3::unit_z());

        let next_camera_buffer_desc = wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(next_camera.as_ref()),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_SRC
        };

        let next_camera_buffer = self.device.create_buffer_init(&next_camera_buffer_desc);

        encoder.copy_buffer_to_buffer(&next_camera_buffer, 0, &self.camera_buffer, 0, mem::size_of::<glam::Mat4>() as wgpu::BufferAddress);

        {
            let rpass_desc = wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.03, g: 0.02, b: 0.03, a: 1.0 }),
                            store: true,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            };

            let mut _rpass = encoder.begin_render_pass(&rpass_desc);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}