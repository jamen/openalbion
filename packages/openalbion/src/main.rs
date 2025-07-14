use std::sync::Arc;

use derive_more::{Display, Error};
use tracing_subscriber::layer::SubscriberExt;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

#[derive(Error, Display, Debug)]
enum StateError {
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
}

impl State {
    async fn new(window: Arc<Window>) -> Result<State, StateError> {
        use StateError as E;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(E::RequestAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(E::RequestDevice)?;

        let surface = instance.create_surface(window.clone()).unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];

        let inner_size = window.inner_size();

        let state = Self {
            window,
            device,
            queue,
            surface,
            surface_format,
        };

        state.reconfigure_surface(inner_size);

        Ok(state)
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn reconfigure_surface(&self, inner_size: winit::dpi::PhysicalSize<u32>) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: inner_size.width,
            height: inner_size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        self.surface.configure(&self.device, &surface_config);
    }
}

#[derive(Error, Display, Debug)]
enum RenderError {
    GetCurrentSurface(wgpu::SurfaceError),
}

impl State {
    fn render(&mut self) -> Result<(), RenderError> {
        use RenderError as E;

        let surface_texture = self
            .surface
            .get_current_texture()
            .map_err(E::GetCurrentSurface)?;

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO: Something better than an unwrap
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // TODO: Something better than an unwrap
        let state = pollster::block_on(State::new(window.clone())).unwrap();

        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // TODO: Something better than an unwrap
        let state = self.state.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // TODO: Something better than an unwrap
                state.render().unwrap();
                state.window().request_redraw();
            }
            WindowEvent::Resized(inner_size) => {
                state.reconfigure_surface(inner_size);
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_log::LogTracer::init().expect("setup tracing-log");

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
    )
    .expect("setup tracing-tracy");

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}
