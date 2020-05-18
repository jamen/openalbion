use std::env;
use std::path::{PathBuf,Path};
use std::io::{Read,Cursor};
use std::fs::File;

use winit::event::{Event,WindowEvent};
use winit::event_loop::ControlFlow;

// use cgmath::prelude::*;
// use cgmath::Vector3;

use fable_data::{Decode,Entry,Wad,Lev,LevHeader};

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

struct OpenAlbion {
    pub wad_file: File,
    pub wad: Wad,
    pub lev_selected: Option<Lev>,
}

pub struct Render {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub bind_group: wgpu::BindGroup,
    pub swap_chain: wgpu::SwapChain,
    pub pipeline: wgpu::RenderPipeline,
}

impl OpenAlbion {
    fn new<T: AsRef<Path>>(fable_path: T) -> Self {
        let fable_path = fable_path.as_ref();
        let wad_path = fable_path.join("data/levels/FinalAlbion.wad");
        let mut wad_file = File::open(&wad_path).unwrap();
        let wad = Wad::decode(&mut wad_file).unwrap();

        OpenAlbion {
            wad_file,
            wad,
            lev_selected: None,
        }
    }

    // TODO: Make this less ugly with its unwraps
    pub fn select_lev(&mut self, lev_name: &str) -> bool {
        let lev_entry = match self.wad.entries.iter_mut().find(|x| {
            Path::new(&x.path).file_name().unwrap().to_str().unwrap() == lev_name
        }) {
            Some(x) => x,
            None => return false,
        };

        let mut lev_source = lev_entry.reader(&mut self.wad_file).unwrap();
        let mut lev_buf = Vec::with_capacity(lev_entry.length as usize);

        lev_source.read_to_end(&mut lev_buf).unwrap();

        let mut lev_buf_reader = Cursor::new(lev_buf);

        let lev = Lev::decode(&mut lev_buf_reader).unwrap();

        self.lev_selected = Some(lev);

        true
    }

    fn create_indexed_verts_from_lev(lev: &Lev) -> (Vec<cgmath::Point3<f32>>, Vec<f32>) {
        let mut vert_buf = Vec::with_capacity(lev.heightmap_cells.len() * 3);
        let mut index_buf = Vec::with_capacity(lev.heightmap_cells.len() * 2);

        for (i, cell) in lev.heightmap_cells.iter().enumerate() {
            // let LevHeader { width, height, .. } = lev.header;
            // let x = i % width as usize;
            // let z = (i - x) / height as usize;

            // vert_buf.push(
            //     cgmath::Point3::new(
            //         x as f32 / width as f32,
            //         cell.height,
            //         z as f32 / height as f32,
            //     )
            // );

            // if z == 0 {
            //     index_buf.push();
            // } else {
            //     index_buf.push();
            // }
        }

        (vert_buf, index_buf)
    }

    fn draw(&mut self, render: &mut Render) {
        let frame = render.swap_chain.get_next_texture().expect("Timeout when acquiring next swap chain texture");

        let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&render.pipeline);
            rpass.draw(0..3, 0..1);
        }

        render.queue.submit(&[ encoder.finish() ]);
    }

    fn start(mut self) -> ! {
        log::info!("Starting.");

        log::info!("Creating window.");

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("Open Albion")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
            .with_resizable(false)
            .with_visible(false)
            .build(&event_loop)
            .expect("Failed to create window");

        log::debug!("Creating wgpu surface.");

        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        log::debug!("Creating wgpu adapater.");

        let adapter = pollster::block_on(
            wgpu::Adapter::request(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::Default,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY
            )
        ).unwrap();

        log::debug!("Creating wgpu device and queue.");

        let (device, queue) = {
            let device_descriptor = wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions { anisotropic_filtering: false },
                limits: wgpu::Limits::default(),
            };

            pollster::block_on(adapter.request_device(&device_descriptor))
        };

        log::debug!("Creating wgpu swap chain.");

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        log::debug!("Load shaders.");

        let frag = include_bytes!("../out/resources/shaders/fragment.spv");
        let frag_source = wgpu::read_spirv(Cursor::new(&frag[..])).unwrap();
        let frag_module = device.create_shader_module(&frag_source);

        let vert = include_bytes!("../out/resources/shaders/vertex.spv");
        let vert_source = wgpu::read_spirv(Cursor::new(&vert[..])).unwrap();
        let vert_module = device.create_shader_module(&vert_source);

        log::debug!("Creating bind group.");

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false }
                    }
                ]
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[ &bind_group_layout ],
            }
        );

        log::debug!("Create bind groun resources");

        let mx_projection = cgmath::perspective(
            cgmath::Deg(45.0f32),
            swap_chain_descriptor.width as f32 / swap_chain_descriptor.height as f32,
            1.0,
            10.0,
        );
        let mx_view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let mx_perspective = OPENGL_TO_WGPU_MATRIX * mx_projection * mx_view;
        let mx_perspective_ref: &[f32; 16] = mx_perspective.as_ref();

        let mx_perspective_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(mx_perspective_ref),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST
        );

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &mx_perspective_buf,
                            range: 0..std::mem::size_of_val(&mx_perspective_buf) as u64
                        }
                    }
                ]
            }
        );

        log::debug!("Create render pipeline.");

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                layout: &pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vert_module,
                    entry_point: "main",
                },
                fragment_stage: Some(
                    wgpu::ProgrammableStageDescriptor {
                        module: &frag_module,
                        entry_point: "main",
                    }
                ),
                rasterization_state: Some(
                    wgpu::RasterizationStateDescriptor {
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: wgpu::CullMode::None,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                    }
                ),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[
                    wgpu::ColorStateDescriptor {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        color_blend: wgpu::BlendDescriptor::REPLACE,
                        alpha_blend: wgpu::BlendDescriptor::REPLACE,
                        write_mask: wgpu::ColorWrite::ALL,
                    }
                ],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[
                        wgpu::VertexBufferDescriptor {
                            stride: 4 * 4,
                            step_mode: wgpu::InputStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttributeDescriptor {
                                    format: wgpu::VertexFormat::Float4,
                                    offset: 0,
                                    shader_location: 0,
                                }
                            ]
                        }
                    ],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            }
        );

        log::info!("Creating renderer context.");

        let mut render = Render {
            device,
            queue,
            bind_group,
            swap_chain,
            pipeline,
        };

        log::info!("Starting event loop.");

        // Do the first render before the window is visible, so it doesn't start blank.
        self.draw(&mut render);

        window.set_visible(true);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        },
                        _ => {
                            // open_albion.update(event);
                        }
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw()
                },
                Event::RedrawRequested(_window_id) => {
                    self.draw(&mut render);
                },
                _event => {}
            }
        })
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().skip(1).collect();

    let fable_path = match args.get(0) {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().expect("No Fable directory found.")
    };

    log::debug!("Fable path: {:?}", fable_path);

    let mut open_albion = OpenAlbion::new(fable_path);

    open_albion.select_lev("LookoutPoint.lev");

    open_albion.start()
}