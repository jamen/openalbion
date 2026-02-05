mod camera;
mod files;
mod renderer;

use self::{
    camera::AnimatedCamera,
    files::{Files, NewFilesError},
    renderer::{LightingColoursError, NewRendererError, Renderer, SkyTextureError},
};
use argh::FromArgs;
use derive_more::{Display, Error};
use std::{borrow::Cow, path::Path, sync::Arc, time::Instant};
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
    let cli = argh::from_env::<Cli>();

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
    files: Files,
    renderer: Option<Renderer<'static>>,
    window: Option<Arc<Window>>,
    camera: AnimatedCamera,
    last_frame_time: Option<Instant>,
    time_of_day: f32,
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

        let files = Files::new(Path::new(fable_directory)).map_err(E::Files)?;

        for bank in files.textures.bank_iter() {
            let metadata = bank.metadata();

            tracing::debug!(
                "Bank: {} (id={}, assets={})",
                metadata.name,
                metadata.id,
                metadata.asset_count
            );

            for asset in bank.asset_iter() {
                tracing::debug!("  Asset: {} (id={})", asset.symbol_name, asset.id);
            }
        }

        Ok(Self {
            files,
            renderer: None,
            window: None,
            camera: AnimatedCamera::new(),
            last_frame_time: None,
            time_of_day: 18.0,
        })
    }
}

use fable_data::big::ReadAssetDataError;

#[derive(Debug, Display, Error)]
enum TryResumedError {
    CreateWindow(OsError),
    NewRenderer(NewRendererError),
    ReadAssetData(ReadAssetDataError),
    UploadSkyTexture(SkyTextureError),
    UploadLightingLut(LightingColoursError),
}

impl App {
    fn try_resumed(&mut self, event_loop: &ActiveEventLoop) -> Result<(), TryResumedError> {
        use TryResumedError as E;

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .map_err(E::CreateWindow)?,
        );

        let mut renderer =
            pollster::block_on(Renderer::new(window.clone())).map_err(E::NewRenderer)?;

        renderer
            .set_lighting_lut(&self.files.lighting_lut_bytes)
            .map_err(E::UploadLightingLut)?;

        tracing::info!("Uploaded lighting LUT to GPU");

        let (tex0_name, tex1_name) = {
            let theme = self
                .files
                .environment_theme("ENVIRONMENT_THEME1")
                .expect("No environment theme found");

            let (tex0, tex1, _) = theme.sky_textures_at_time(self.time_of_day);

            (tex0.map(String::from), tex1.map(String::from))
        };

        if let Some(ref name) = tex0_name {
            let (metadata, bytes) = self
                .files
                .read_sky_texture(name)
                .map_err(E::ReadAssetData)?;

            renderer
                .set_sky_texture0(&metadata, &bytes)
                .map_err(E::UploadSkyTexture)?;

            tracing::info!("Uploaded sky texture 0: {}", name);
        }

        if let Some(ref name) = tex1_name {
            if tex1_name != tex0_name {
                let (metadata, bytes) = self
                    .files
                    .read_sky_texture(name)
                    .map_err(E::ReadAssetData)?;

                renderer
                    .set_sky_texture1(&metadata, &bytes)
                    .map_err(E::UploadSkyTexture)?;

                tracing::info!("Uploaded sky texture 1: {}", name);
            }
        }

        let size = window.inner_size();

        renderer.resize_surface(size.into());

        self.camera.camera.set_aspect(size.width, size.height);

        window.request_redraw();

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        Ok(())
    }
}

#[derive(Debug, Display, Error)]
enum WindowEventError {
    Resize(ResizeError),
    RedrawRequested(RedrawRequestedError),
}

impl App {
    fn try_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) -> Result<(), WindowEventError> {
        use WindowEventError as E;

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw_requested().map_err(E::RedrawRequested)?,
            WindowEvent::Resized(size) => self.resize(size).map_err(E::Resize)?,
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Display, Error)]
enum RedrawRequestedError {
    NoWindow,
    NoRenderer,
    Render(SurfaceError),
}

impl App {
    fn redraw_requested(&mut self) -> Result<(), RedrawRequestedError> {
        use RedrawRequestedError as E;

        let window = self.window.as_ref().ok_or(E::NoWindow)?;
        let renderer = self.renderer.as_mut().ok_or(E::NoRenderer)?;

        let now = Instant::now();

        let delta_time = self
            .last_frame_time
            .map(|last| now.duration_since(last).as_secs_f32())
            .unwrap_or(0.0);

        self.last_frame_time = Some(now);

        self.camera.update(delta_time);

        self.time_of_day += delta_time;

        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }

        let sky_blend = 0.0;

        renderer.update_sky_uniforms(
            self.camera.sky_view_projection(),
            self.time_of_day,
            sky_blend,
        );

        let pre_present = renderer.render().map_err(E::Render)?;

        window.pre_present_notify();

        pre_present.present();

        window.request_redraw();

        Ok(())
    }
}

#[derive(Debug, Display, Error)]
enum ResizeError {
    NoRenderer,
}

impl App {
    fn resize(&mut self, size: PhysicalSize<u32>) -> Result<(), ResizeError> {
        use ResizeError as E;

        let renderer = self.renderer.as_ref().ok_or(E::NoRenderer)?;

        renderer.resize_surface(size.into());

        self.camera.camera.set_aspect(size.width, size.height);

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.try_resumed(event_loop) {
            tracing::error!("{}", error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Err(error) = self.try_window_event(event_loop, id, event) {
            tracing::error!("{}", error);
        }
    }
}
