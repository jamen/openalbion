use std::io::Cursor;

use winit::event::{Event,WindowEvent};
use winit::event_loop::ControlFlow;

pub struct Render {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub pipeline: wgpu::RenderPipeline,
}

pub trait Renderer: Sized + Send + 'static {
    fn draw(&mut self, render: &mut Render);

    // fn update(&mut self, event: WindowEvent);

    fn start(mut self) -> ! {
        log::info!("Creating window.");

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("Open Albion")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
            .with_resizable(false)
            .build(&event_loop)
            .expect("Failed to create window");

        log::info!("Creating wgpu.");

        // let instance = wgpu_core::instance::Instance::new("Open Albion", 0);

        log::debug!("Creating wgpu surface.");

        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        log::debug!("Creating wgpu adapater.");

        let adapter = {
            let request_adapter_options = wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            };

            let future_adapter_result = wgpu::Adapter::request(&request_adapter_options, wgpu::BackendBit::PRIMARY);

            pollster::block_on(future_adapter_result).unwrap()
        };

        log::debug!("Creating wgpu device and queue.");

        let (device, queue) = {
            let device_descriptor = wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions { anisotropic_filtering: false },
                limits: wgpu::Limits::default(),
            };

            let future_device = adapter.request_device(&device_descriptor);

            pollster::block_on(future_device)
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

        log::debug!("Create render pipieline.");

        let pipeline_layout = {
            let pipeline_layout_descriptor = wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            };

            device.create_pipeline_layout(&pipeline_layout_descriptor)
        };

        let pipeline = {
            let render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
                layout: &pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vert_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &frag_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            };

            device.create_render_pipeline(&render_pipeline_descriptor)
        };

        log::info!("Creating Open Albion context.");

        let mut render = Render {
            device,
            queue,
            swap_chain,
            pipeline,
        };

        log::info!("Starting event loop.");

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