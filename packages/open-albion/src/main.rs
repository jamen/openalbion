mod camera;
mod files;
mod renderer;

use self::{
    camera::Camera,
    files::{Files, NewFilesError},
    renderer::{LightingColoursError, ModelTextureError, NewRendererError, Renderer},
};
use argh::FromArgs;
use derive_more::{Display, Error};
use std::{borrow::Cow, collections::HashSet, path::Path, sync::Arc, time::Instant};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::{EventLoopError, OsError},
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

/// OpenAlbion renderer
#[derive(FromArgs)]
struct Cli {
    /// log filter directive
    #[argh(option)]
    log: Option<String>,

    /// fable's directory
    #[argh(option)]
    fable_directory: Option<String>,

    /// level to load from FinalAlbion.wad (default: Witchwood)
    #[argh(option)]
    level: Option<String>,

    /// mesh to load from graphics.big by symbol name (default: first renderable mesh found)
    #[argh(option)]
    mesh: Option<String>,
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
    camera: Camera,
    last_frame_time: Option<Instant>,
    time_of_day: f32,
    level_name: String,
    mesh_name: Option<String>,
    terrain_center: glam::Vec3,
    terrain_radius: f32,
    /// The sky texture name pair currently uploaded to the GPU, so we only re-upload on change.
    sky_textures: Option<(Option<String>, Option<String>)>,
    /// Currently pressed keys.
    keys: HashSet<KeyCode>,
    /// Whether the cursor is locked (mouse look active).
    cursor_locked: bool,
    /// Frame counter to detect first load.
    first_frame: bool,
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
            camera: Camera::new(),
            last_frame_time: None,
            time_of_day: 18.0,
            level_name: cli.level.clone().unwrap_or_else(|| "Witchwood".to_string()),
            mesh_name: cli.mesh.clone(),
            terrain_center: glam::Vec3::ZERO,
            terrain_radius: 1.0,
            sky_textures: None,
            keys: HashSet::new(),
            cursor_locked: false,
            first_frame: true,
        })
    }
}

#[derive(Debug, Display, Error)]
enum TryResumedError {
    CreateWindow(OsError),
    NewRenderer(NewRendererError),
    UploadLightingLut(LightingColoursError),
    LoadLevel(files::LoadLevelError),
    UploadModelTexture(ModelTextureError),
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

        // Load and upload the terrain.
        let lev = self.files.load_level(&self.level_name).map_err(E::LoadLevel)?;
        let span_x = lev.header.width as f32 + 1.0;
        let span_z = lev.header.height as f32 + 1.0;

        let raw_min = lev.heightmap_cells.iter().map(|c| c.height).fold(f32::INFINITY, f32::min);
        let raw_max = lev.heightmap_cells.iter().map(|c| c.height).fold(f32::NEG_INFINITY, f32::max);
        let scale = renderer::terrain::HEIGHT_SCALE;
        let mid_y = (raw_min + raw_max) * 0.5 * scale;

        self.terrain_center = glam::Vec3::new(span_x * 0.5, mid_y, span_z * 0.5);
        self.terrain_radius = span_x.max(span_z) * 0.5;
        let world_span = span_x.max(span_z).max((raw_max - raw_min).abs() * scale);
        self.camera.set_world_extents(world_span);
        // Position camera above and back from the terrain centre for a good initial view.
        self.camera.position =
            self.terrain_center + glam::Vec3::new(world_span * 0.3, world_span * 0.4, world_span * 0.5);
        self.camera.look_at(self.terrain_center, glam::Vec3::Y);
        self.camera.fly_speed = world_span * 0.1;
        renderer.set_terrain(&lev);
        tracing::info!(
            "Uploaded terrain to GPU (size {}x{} cells, height raw=[{:.4}, {:.4}] scaled=[{:.1}, {:.1}], center=({:.1}, {:.1}, {:.1}), radius={:.1}, world_span={world_span:.1})",
            lev.header.width, lev.header.height,
            raw_min, raw_max,
            raw_min * scale, raw_max * scale,
            self.terrain_center.x, self.terrain_center.y, self.terrain_center.z,
            self.terrain_radius,
        );

        // Load and upload a test model (optional — continues without if mesh loading fails).
        self.load_model(&mut renderer)?;

        let size = window.inner_size();

        renderer.resize_surface(size.into());

        self.camera.set_aspect(size.width, size.height);

        window.request_redraw();

        self.window = Some(window.clone());
        self.renderer = Some(renderer);

        // Upload the initial sky now that the renderer is in place (sky is optional).
        self.refresh_sky();

        Ok(())
    }

    /// Resolve the sky textures and blend factor for the current time-of-day, re-uploading the
    /// textures to the GPU only when the active pair changes. Returns the blend factor (0..1)
    /// between sky texture 0 and 1. A missing environment theme or failed read leaves the sky
    /// unchanged and returns 0.0 — the sky is optional.
    fn refresh_sky(&mut self) -> f32 {
        let Some(renderer) = self.renderer.as_mut() else {
            return 0.0;
        };

        let Some((tex0_name, tex1_name, blend)) = self
            .files
            .environment_theme("ENVIRONMENT_THEME1")
            .map(|theme| {
                let (tex0, tex1, blend) = theme.sky_textures_at_time(self.time_of_day);
                (tex0.map(String::from), tex1.map(String::from), blend)
            })
        else {
            return 0.0;
        };

        let names = (tex0_name, tex1_name);
        if self.sky_textures.as_ref() != Some(&names) {
            let (tex0_name, tex1_name) = &names;
            if let Some(name) = tex0_name {
                upload_sky_texture(&mut self.files, renderer, name, false);
            }
            // Only upload texture1 when it differs from texture0 (the pass reuses 0 otherwise).
            if let Some(name) = tex1_name.as_ref().filter(|n| Some(*n) != tex0_name.as_ref()) {
                upload_sky_texture(&mut self.files, renderer, name, true);
            }
            tracing::debug!("Sky textures at {:.1}h: {:?}", self.time_of_day, names);
            self.sky_textures = Some(names);
        }

        blend
    }

    /// Load a model and upload it to the renderer. Uses `--mesh` if given, otherwise scans
    /// graphics.big for the first renderable mesh (one with an unpacked vertex format and a
    /// resolvable texture). The model is optional — failures are logged, not fatal.
    fn load_model(&mut self, renderer: &mut Renderer<'static>) -> Result<(), TryResumedError> {
        use TryResumedError as E;

        let explicit = self.mesh_name.is_some();
        let candidates: Vec<String> = match &self.mesh_name {
            Some(name) => vec![name.clone()],
            None => {
                let mut meshes: Vec<String> = self
                    .files
                    .graphics
                    .bank_iter()
                    .flat_map(|bank| bank.asset_iter())
                    .filter(|a| matches!(&a.extras, Some(fable_data::big::ExtraMetadata::Mesh(_))))
                    .map(|a| a.symbol_name.to_string())
                    .collect();
                meshes.sort();
                meshes
            }
        };

        for name in &candidates {
            let (mesh, textures) = match self.files.read_mesh(name) {
                Ok(loaded) => loaded,
                Err(error) => {
                    tracing::warn!("Failed to load mesh {name}: {error}");
                    continue;
                }
            };
            let resolved_textures = textures.iter().filter(|t| t.is_some()).count();
            // When auto-scanning, skip meshes with no resolvable texture; an explicit request is
            // honoured regardless.
            if !explicit && resolved_textures == 0 {
                tracing::debug!("Skipping mesh {name} (no resolvable textures)");
                continue;
            }
            tracing::info!(
                "Loading mesh {name} ({} materials, {resolved_textures} textures)",
                mesh.materials.len(),
            );
            renderer
                .set_model(&mesh, &textures)
                .map_err(E::UploadModelTexture)?;
            tracing::info!("Uploaded model to GPU");
            return Ok(());
        }

        if explicit {
            tracing::warn!("Requested mesh {:?} could not be loaded", self.mesh_name);
        }
        Ok(())
    }
}

/// Read a sky texture from the textures archive and upload it to the renderer's primary
/// (`secondary == false`) or blend (`secondary == true`) slot. Failures are logged, not fatal.
fn upload_sky_texture(files: &mut Files, renderer: &mut Renderer<'static>, name: &str, secondary: bool) {
    let (metadata, bytes) = match files.read_sky_texture(name) {
        Ok(asset) => asset,
        Err(error) => {
            tracing::warn!("Failed to read sky texture {name}: {error}");
            return;
        }
    };

    let result = if secondary {
        renderer.set_sky_texture1(&metadata, &bytes)
    } else {
        renderer.set_sky_texture0(&metadata, &bytes)
    };

    if let Err(error) = result {
        tracing::warn!("Failed to upload sky texture {name}: {error}");
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                if state == ElementState::Pressed {
                    // Escape unlocks the cursor.
                    if keycode == KeyCode::Escape {
                        if let Some(window) = &self.window {
                            window.set_cursor_visible(true);
                            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                            self.cursor_locked = false;
                        }
                        return Ok(());
                    }
                    // Click locks the cursor again.
                    if keycode == KeyCode::Enter && !self.cursor_locked {
                        if let Some(window) = &self.window {
                            window.set_cursor_visible(false);
                            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                            self.cursor_locked = true;
                        }
                        return Ok(());
                    }
                    self.keys.insert(keycode);
                } else {
                    self.keys.remove(&keycode);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // When cursor is unlocked, we use absolute position delta
                // (handled via raw device events for locked mode).
                let _ = position;
            }
            _ => {}
        }

        Ok(())
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if !self.cursor_locked {
            return;
        }
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera.process_mouse(delta.0 as f32, delta.1 as f32);
        }
        if let DeviceEvent::MouseWheel { delta } = &event {
            let dy = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
            };
            self.camera.fly_speed = (self.camera.fly_speed * 1.1_f32.powf(dy))
                .clamp(0.1, self.terrain_radius * 20.0);
        }
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

        let now = Instant::now();

        let delta_time = self
            .last_frame_time
            .map(|last| now.duration_since(last).as_secs_f32())
            .unwrap_or(0.0);

        self.last_frame_time = Some(now);

        // On first frame, lock cursor for fly camera.
        if self.first_frame {
            self.first_frame = false;
            if let Some(window) = &self.window {
                window.set_cursor_visible(false);
                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                self.cursor_locked = true;
            }
        }

        // Fly camera: process input.
        let speed_mult = if self.keys.contains(&KeyCode::ShiftLeft)
            || self.keys.contains(&KeyCode::ShiftRight)
        {
            3.0
        } else {
            1.0
        };

        self.camera.fly(
            delta_time,
            (
                self.keys.contains(&KeyCode::KeyW),
                self.keys.contains(&KeyCode::KeyS),
                self.keys.contains(&KeyCode::KeyA),
                self.keys.contains(&KeyCode::KeyD),
                self.keys.contains(&KeyCode::Space),
                self.keys.contains(&KeyCode::ControlLeft)
                    || self.keys.contains(&KeyCode::ControlRight),
            ),
            speed_mult,
        );

        self.time_of_day += delta_time * 0.1; // ~4 real minutes per game hour
        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }

        // Re-select the sky textures for the new time-of-day before borrowing the renderer.
        let sky_blend = self.refresh_sky();
        let sky_view_proj = self.camera.sky_view_projection_matrix().to_cols_array_2d();
        let view_proj = self.camera.view_projection_matrix().to_cols_array_2d();

        let window = self.window.as_ref().ok_or(E::NoWindow)?;
        let renderer = self.renderer.as_mut().ok_or(E::NoRenderer)?;

        renderer.update_sky_uniforms(sky_view_proj, self.time_of_day, sky_blend);
        renderer.update_terrain_uniforms(view_proj);
        renderer.update_model_uniforms(view_proj);

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

        let renderer = self.renderer.as_mut().ok_or(E::NoRenderer)?;

        renderer.resize_surface(size.into());

        self.camera.set_aspect(size.width, size.height);

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

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.device_event(event_loop, device_id, event);
    }
}
