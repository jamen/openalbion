mod files;
mod renderer;

use crate::files::NewFilesError;

use self::{
    files::Files,
    renderer::{NewRendererError, Renderer},
};
use derive_more::Display;
use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing_subscriber::layer::SubscriberExt;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct State {
    files: Files,
    window: Arc<Window>,
    renderer: Renderer<'static>,
}

#[derive(Display, Debug)]
enum StateError {
    NewRenderer(NewRendererError),
    NewFiles(NewFilesError),
}

impl State {
    async fn new(window: Arc<Window>, fable_directory: &Path) -> Result<State, StateError> {
        use StateError as E;

        let files = Files::new(fable_directory).map_err(E::NewFiles)?;

        let renderer = Renderer::new(window.clone())
            .await
            .map_err(E::NewRenderer)?;

        let PhysicalSize { width, height } = window.inner_size();

        renderer.resize_surface(width, height);

        Ok(Self {
            files,
            window,
            renderer,
        })
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
        let mut args = env::args().skip(1);

        let fable_directory = args
            .next()
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .expect("No fable directory");

        // TODO: Something better than an unwrap
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        // TODO: Something better than an unwrap
        let state = State::new(window.clone(), &fable_directory);
        let state = pollster::block_on(state).unwrap();

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
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                state.renderer.resize_surface(width, height);
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
