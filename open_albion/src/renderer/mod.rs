
use std::mem;
use std::borrow::Cow;

use winit::window::Window;

use glam::{Vec4,Mat4};

use fable_data::Lev;

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
    pub context: Context,
    // pub console: Console,
    pub landscape: Landscape,
}

pub struct Context {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub format: wgpu::TextureFormat,
    pub swap_chain: wgpu::SwapChain,
    pub camera: wgpu::Buffer,
}

// pub struct Console (Pass);

pub struct Landscape (Pass);

pub struct Pass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub mesh: Option<Mesh>,
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub draw_count: usize,
}

impl Context {
    fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter_desc = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        };

        // TODO: Make this an async function and not blocK?

        let adapter = smol::block_on(instance.request_adapter(&adapter_desc)).unwrap();

        let device_desc = wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            shader_validation: false,
        };

        let (device, queue) = smol::block_on(adapter.request_device(&device_desc, None)).unwrap();

        let size = window.inner_size();

        let format = wgpu::TextureFormat::Bgra8Unorm;

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            size: mem::size_of::<Mat4>() as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            size,
            format,
            swap_chain,
            camera,
        }
    }

    pub fn update_swap_chain(&mut self, window: &Window) {
        let size = window.inner_size();

        self.size = size;

        self.swap_chain = self.device.create_swap_chain(&self.surface, &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        });
    }

    pub fn update_camera(&mut self, camera: Mat4) {
        self.queue.write_buffer(&self.camera, 0, bytemuck::cast_slice(camera.as_ref()))
    }
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        let context = Context::new(&window);

        let landscape = Landscape::new(&context);

        // let console = Console::new(&context);

        Self {
            context,
            landscape,
            // console,
        }
    }

    pub fn render(&mut self) {

    }

    // pub fn render(&mut self, state: &App) {
    //     let frame = match self.swap_chain.get_current_frame() {
    //         Ok(x) => x,
    //         Err(e) => {
    //             eprintln!("Dropped frame. {}", e);
    //             return
    //         }
    //     };

    //     let landscape = Landscape::new();

    //     let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    //     {
    //         let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             color_attachments: &[
    //                 wgpu::RenderPassColorAttachmentDescriptor {
    //                     attachment: &frame.output.view,
    //                     resolve_target: None,
    //                     ops: wgpu::Operations {
    //                         load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.85, g: 0.85, b: 0.9, a: 1.0 }),
    //                         store: true,
    //                     },
    //                 }
    //             ],
    //             depth_stencil_attachment: None,
    //         });

    //         passes.landscape.draw(&mut rpass);
    //     }

    //     self.queue.submit(std::iter::once(encoder.finish()));
    // }
}

impl Landscape {
    pub fn new(context: &Context) -> Self {
        let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(context.camera.slice(..))
                }
            ]
        });

        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader_module!(context.device, "shaders/shader.vert", kind: vert),
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shader_module!(context.device, "shaders/shader.frag", kind: frag),
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: context.format,
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
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Vec4>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float4]
                    }
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self (
            Pass {
                bind_groups: vec![bind_group],
                pipeline,
                mesh: None,
            }
        )
    }

    pub fn update(&mut self, lev: &Lev) {

    }
}

// impl Pass {
//     fn landscape(renderer: &mut Renderer) {

//     }
// }

// impl Camera {
//     pub fn new(device: &wgpu::Device, size: &PhysicalSize<u32>) -> Camera {
//         let sensitivity = 0.2f32;
//         let field_of_view = 90.0f32;
//         let position = Vec3::default();
//         let rotation = Quat::from_axis_angle(Vec3::unit_y(), 0.0);
//         let focal_length = 1.0;
//         let focus = Vec3::unit_x();

//         let matrix =
//             Mat4::perspective_infinite_lh(field_of_view.to_radians(), size.width as f32 / size.height as f32, 0.01) *
//             Mat4::look_at_lh(position, focus, Vec3::unit_y());

//         let buffer = device.create_buffer_init(&BufferInitDescriptor {
//             label: Some("camera"),
//             usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
//             contents: bytemuck::cast_slice(matrix.as_ref()),
//         });

//         Camera {
//             sensitivity,
//             field_of_view,
//             position,
//             rotation,
//             focal_length,
//             buffer,
//         }
//     }
// }


