mod renderer;

use self::renderer::{NewRendererError, Renderer};
use derive_more::{Display, Error};
use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct State {
    window: Arc<Window>,
    renderer: Renderer<'static>,
}

#[derive(Error, Display, Debug)]
enum StateError {
    NewRenderer(NewRendererError),
}

impl State {
    async fn new(window: Arc<Window>) -> Result<State, StateError> {
        use StateError as E;

        let renderer = Renderer::new(window.clone())
            .await
            .map_err(E::NewRenderer)?;

        let inner_size = window.inner_size();

        renderer.resize_surface(inner_size.width, inner_size.height);

        Ok(Self { window, renderer })
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let pre_present = self.renderer.render()?;
        self.window.pre_present_notify();
        pre_present.present();
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
                state.window.request_redraw();
            }
            WindowEvent::Resized(inner_size) => {
                state
                    .renderer
                    .resize_surface(inner_size.width, inner_size.height);
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
