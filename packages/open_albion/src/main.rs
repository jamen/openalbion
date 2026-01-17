mod files;
mod renderer;

use self::{
    files::{Files, NewFilesError},
    renderer::{NewRendererError, Renderer},
};
use argh::FromArgs;
use derive_more::{Display, Error};
use std::{borrow::Cow, sync::Arc};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::{EventLoopError, OsError},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// music-streamer web server
#[derive(FromArgs)]
struct Cli {
    /// log filter directive
    #[argh(option)]
    log: Option<String>,

    /// fable's directory
    #[argh(option)]
    fable_directory: Option<String>,
}

fn main() {
    // Parse CLI
    let cli = argh::from_env::<Cli>();

    // Start logger
    let log_directive = cli.log.clone().map(Cow::Owned).unwrap_or(Cow::Borrowed(""));

    let log_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .parse_lossy(log_directive.as_ref());

    tracing_subscriber::registry()
        .with(log_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(error) = try_main(cli) {
        tracing::error!("{}", error);
    }
}

#[derive(Debug, Display)]
enum TryMainError {
    NewApp(NewAppError),
    NewEventLoop(EventLoopError),
    RunEventLoop(EventLoopError),
}

fn try_main(cli: Cli) -> Result<(), TryMainError> {
    use TryMainError as E;

    let mut app = App::new(cli).map_err(E::NewApp)?;
    let event_loop = EventLoop::new().map_err(E::NewEventLoop)?;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).map_err(E::RunEventLoop)?;
    Ok(())
}

struct App {
    cli: Cli,
    renderer: Option<Renderer<'static>>,
    window: Option<Arc<Window>>,
}

#[derive(Debug, Display)]
enum NewAppError {
    NoFableDirectory,
    Files(NewFilesError),
}

impl App {
    fn new(cli: Cli) -> Result<Self, NewAppError> {
        use NewAppError as E;

        let fable_directory = cli.fable_directory.as_ref().ok_or(E::NoFableDirectory)?;

        tracing::info!("{}", fable_directory);

        let _files = Files::new(fable_directory).map_err(E::Files)?;

        Ok(Self {
            cli,
            renderer: None,
            window: None,
        })
    }
}

#[derive(Debug, Display, Error)]
enum TryResumedError {
    CreateWindow(OsError),
    NewRenderer(NewRendererError),
}

impl App {
    fn try_resumed(&mut self, event_loop: &ActiveEventLoop) -> Result<(), TryResumedError> {
        use TryResumedError as E;

        let window = event_loop
            .create_window(Window::default_attributes())
            .map_err(E::CreateWindow)?;
        let window = Arc::new(window);

        let renderer = Renderer::new(window.clone());
        let renderer = pollster::block_on(renderer).map_err(E::NewRenderer)?;

        renderer.resize_surface(window.inner_size().into());
        window.request_redraw();

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        Ok(())
    }
}

#[derive(Debug, Display, Error)]
enum HandleWindowEventError {
    Render(SurfaceError),
}

impl App {
    fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) -> Result<(), HandleWindowEventError> {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.handle_redraw_requested()?,
            WindowEvent::Resized(size) => self.handle_resize(size),
            _ => {}
        }
        Ok(())
    }

    fn handle_redraw_requested(&mut self) -> Result<(), HandleWindowEventError> {
        use HandleWindowEventError as E;

        if let Some(renderer) = self.renderer.as_mut() {
            let pre_present = renderer.render().map_err(E::Render)?;

            if let Some(window) = self.window.as_ref() {
                window.pre_present_notify();
            } else {
                tracing::warn!("No window to pre-present notify");
            }

            pre_present.present();
        } else {
            tracing::warn!("No renderer to render");
        }

        Ok(())
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        if let Some(renderer) = self.renderer.as_ref() {
            renderer.resize_surface(size.into());
        } else {
            tracing::warn!("No renderer to resize");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.try_resumed(event_loop) {
            tracing::error!("{}", error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Err(error) = self.handle_window_event(event_loop, id, event) {
            tracing::error!("{}", error);
        }
    }
}
