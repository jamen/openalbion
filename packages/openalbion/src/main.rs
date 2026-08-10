//! OpenAlbion — a recreation of Fable: The Lost Chapters' engine.
//!
//! The engine binary. Drawing lives in the `renderer` crate, which knows nothing about
//! Fable's asset formats; `scene` is where this binary turns assets into the plain data
//! the renderer accepts. World space is Z-up, matching the game (AGENTS.md §3.6).

mod camera;
mod files;
mod scene;

use crate::camera::Camera;
use crate::files::{Files, NewFilesError};
use renderer::{NewRendererError, Renderer};
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
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent},
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

    /// show a single mesh from graphics.big by symbol name instead of the level's things
    #[argh(option)]
    mesh: Option<String>,

    /// render one frame offscreen to this PPM path and exit, instead of opening a window
    #[argh(option)]
    screenshot: Option<String>,

    /// camera world position for --screenshot, as `x,y,z` (default: framed on the level)
    #[argh(option)]
    camera: Option<String>,

    /// camera target for --screenshot, as `x,y,z` (default: the terrain centre)
    #[argh(option)]
    look_at: Option<String>,
}

/// `x,y,z` → a point. Returns `None` for anything else, so a typo falls back to the default
/// framing rather than putting the camera somewhere silently wrong.
fn parse_point(text: &str) -> Option<glam::Vec3> {
    let parts: Vec<f32> = text.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    match parts[..] {
        [x, y, z] => Some(glam::Vec3::new(x, y, z)),
        _ => None,
    }
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
    NewRenderer(NewRendererError),
    LoadScene(TryResumedError),
    Capture(renderer::CaptureError),
    WriteScreenshot(std::io::Error),
}

fn try_main(cli: Cli) -> Result<(), TryMainError> {
    use TryMainError as E;

    let screenshot = cli.screenshot.clone();
    let camera = cli.camera.as_deref().and_then(parse_point);
    let look_at = cli.look_at.as_deref().and_then(parse_point);
    let mut app = App::new(cli).map_err(E::NewApp)?;

    if let Some(path) = screenshot {
        return capture(&mut app, &path, camera, look_at);
    }

    let event_loop = EventLoop::new().map_err(E::NewEventLoop)?;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).map_err(E::RunEventLoop)?;

    Ok(())
}

/// Render one frame with no window and write it out as a binary PPM.
///
/// PPM because it needs no encoder — the point is a frame to look at without a compositor,
/// not an asset. `magick`, `feh` and every image viewer read it.
fn capture(
    app: &mut App,
    path: &str,
    camera: Option<glam::Vec3>,
    look_at: Option<glam::Vec3>,
) -> Result<(), TryMainError> {
    use TryMainError as E;

    const SIZE: [u32; 2] = [1280, 720];

    let mut renderer =
        pollster::block_on(Renderer::new_headless(SIZE)).map_err(E::NewRenderer)?;

    app.load_scene(&mut renderer).map_err(E::LoadScene)?;
    app.camera.set_aspect(SIZE[0], SIZE[1]);

    // `load_scene` framed the whole level; an explicit camera overrides it.
    if let Some(position) = camera {
        app.camera.position = position;
        app.camera.look_at(look_at.unwrap_or(app.terrain_center));
    } else if let Some(target) = look_at {
        app.camera.look_at(target);
    }
    tracing::info!(
        "Camera at ({:.1}, {:.1}, {:.1})",
        app.camera.position.x,
        app.camera.position.y,
        app.camera.position.z,
    );

    app.renderer = Some(renderer);
    let sky_blend = app.refresh_sky();

    let camera_relative_view_proj = app
        .camera
        .camera_relative_view_projection_matrix()
        .to_cols_array_2d();
    let view_proj = app.camera.view_projection_matrix().to_cols_array_2d();

    let renderer = app.renderer.as_mut().expect("just set");
    renderer.update_sky_uniforms(camera_relative_view_proj, [0.0; 4], [0.0; 4], sky_blend);
    renderer.update_terrain_uniforms(camera_relative_view_proj, app.camera.position);
    renderer.update_model_uniforms(view_proj);
    renderer.set_model_camera_pos(app.camera.position);

    let image = renderer.render_to_image().map_err(E::Capture)?;

    let mut ppm = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
    ppm.extend(image.rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]));
    std::fs::write(path, ppm).map_err(E::WriteScreenshot)?;

    let (meshes, instances) = renderer.model_stats();
    tracing::info!(
        "Wrote {path} ({}x{}), {instances} placements over {meshes} meshes",
        image.width,
        image.height,
    );

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
    LoadLevel(crate::files::LoadLevelError),
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

        self.load_scene(&mut renderer)?;

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

    /// Load the level: landscape, then the things standing on it, then frame the camera on
    /// what was loaded. Shared by the windowed and offscreen paths so a screenshot is the
    /// same scene the viewer shows.
    fn load_scene(&mut self, renderer: &mut Renderer<'_>) -> Result<(), TryResumedError> {
        use TryResumedError as E;

        let lev = self
            .files
            .load_level(&self.level_name)
            .map_err(E::LoadLevel)?;
        let span_x = lev.header.width as f32 + 1.0;
        let span_z = lev.header.height as f32 + 1.0;

        let raw_min = lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::INFINITY, f32::min);
        let raw_max = lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::NEG_INFINITY, f32::max);
        let scale = fable_data::landscape::HEIGHT_SCALE;
        let mid_z = (raw_min + raw_max) * 0.5 * scale;

        // Z-up: the heightmap spans X/Y and height is Z. AGENTS.md §3.6.
        self.terrain_center = glam::Vec3::new(span_x * 0.5, span_z * 0.5, mid_z);
        self.terrain_radius = span_x.max(span_z) * 0.5;
        let world_span = span_x.max(span_z).max((raw_max - raw_min).abs() * scale);
        self.camera.set_world_extents(world_span);
        // Position camera above and back from the terrain centre for a good initial view.
        self.camera.position = self.terrain_center
            + glam::Vec3::new(world_span * 0.3, world_span * 0.5, world_span * 0.4);
        self.camera.look_at(self.terrain_center);
        self.camera.fly_speed = world_span * 0.1;
        renderer.set_terrain(&scene::build_terrain(&mut self.files, &lev));
        tracing::info!(
            "Uploaded terrain to GPU (size {}x{} cells, height raw=[{:.4}, {:.4}] scaled=[{:.1}, {:.1}], center=({:.1}, {:.1}, {:.1}), radius={:.1}, world_span={world_span:.1})",
            lev.header.width,
            lev.header.height,
            raw_min,
            raw_max,
            raw_min * scale,
            raw_max * scale,
            self.terrain_center.x,
            self.terrain_center.y,
            self.terrain_center.z,
            self.terrain_radius,
        );

        // Populate the level from its .tng. Never fatal — a level with unresolvable things
        // still shows its landscape.
        self.load_things(renderer);

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

        let Some((tex0_name, tex1_name, blend)) =
            scene::sky_textures_at_time(&self.files, "ENVIRONMENT_THEME1", self.time_of_day)
        else {
            return 0.0;
        };

        // Re-upload only when the active pair changes.
        let names = (tex0_name, tex1_name);
        if self.sky_textures.as_ref() != Some(&names) {
            scene::upload_sky_textures(
                &mut self.files,
                renderer,
                names.0.as_deref(),
                names.1.as_deref(),
            );
            tracing::debug!("Sky textures at {:.1}h: {:?}", self.time_of_day, names);
            self.sky_textures = Some(names);
        }

        blend
    }

    /// Populate the level from its `.tng`: every thing whose def draws a static mesh, at its
    /// own transform, one upload per distinct mesh.
    ///
    /// Nothing here is fatal. A level that cannot resolve some of its things should still
    /// render the rest and say what it dropped — a missing object must be visible in the log
    /// even when it is invisible on screen.
    fn load_things(&mut self, renderer: &mut Renderer<'_>) {
        renderer.clear_models();

        if self.mesh_name.is_some() {
            self.load_single_mesh(renderer);
            return;
        }

        let tng = match self.files.load_tng(&self.level_name) {
            Ok(tng) => tng,
            Err(error) => {
                tracing::warn!("No .tng for {}: {error} — level will be bare", self.level_name);
                return;
            }
        };

        let things = scene::resolve_things(&tng, &self.files.thing_graphics);
        let resolved_placements = things.placement_count();

        let mut uploaded_meshes = 0usize;
        let mut placed = 0usize;
        let mut failed_meshes = 0usize;

        // Deterministic order so two runs log the same thing.
        let mut mesh_ids: Vec<u32> = things.by_mesh.keys().copied().collect();
        mesh_ids.sort_unstable();

        for mesh_id in mesh_ids {
            let placements = &things.by_mesh[&mesh_id];
            let name = self
                .files
                .mesh_name_by_id(mesh_id)
                .unwrap_or_else(|| format!("#{mesh_id}"));

            let (mesh, textures) = match self.files.read_mesh_by_id(mesh_id) {
                Ok(loaded) => loaded,
                Err(error) => {
                    tracing::warn!("Mesh {name} ({mesh_id}): {error} — {} placements dropped", placements.len());
                    failed_meshes += 1;
                    continue;
                }
            };

            // A material with no resolvable texture draws white rather than dropping the
            // whole mesh: roughly a quarter of the materials in graphics.big have no base
            // texture at all, and a silently absent object is worse than an untextured one.
            let model = match scene::build_model(&mesh, &textures) {
                Ok(model) => model,
                Err(error) => {
                    tracing::warn!("Mesh {name}: {error} — {} placements dropped", placements.len());
                    failed_meshes += 1;
                    continue;
                }
            };

            let instances: Vec<renderer::ModelInstance> = placements
                .iter()
                .map(|p| renderer::ModelInstance {
                    transform: p.transform,
                    // The per-object colour (`c0`) is opaque white until fade distance
                    // lands; that leaves the material exactly as authored.
                    ..Default::default()
                })
                .collect();

            match renderer.add_model(&model, &instances) {
                Ok(()) => {
                    uploaded_meshes += 1;
                    placed += instances.len();
                    // Provenance: which defs became which mesh (AGENTS.md §6.8).
                    let mut defs: Vec<&str> =
                        placements.iter().map(|p| p.definition_type.as_str()).collect();
                    defs.sort_unstable();
                    defs.dedup();
                    tracing::debug!(
                        "{name} (id {mesh_id}) ← {} placements from {defs:?}",
                        instances.len(),
                    );
                }
                Err(error) => {
                    tracing::warn!("Mesh {name}: {error} — {} placements dropped", placements.len());
                    failed_meshes += 1;
                }
            }
        }

        let skipped = &things.skipped;
        tracing::info!(
            "Things: placed {placed} of {resolved_placements} static-mesh placements over \
             {uploaded_meshes} meshes ({failed_meshes} meshes failed); skipped {} things — \
             {} no def, {} not drawable, {} without a placement, {:?} by graphic type",
            skipped.total(),
            skipped.no_def,
            skipped.not_drawable,
            skipped.no_placement,
            skipped.other_graphic_type,
        );
    }

    /// `--mesh NAME`: show one mesh at the terrain centre, for looking at an asset.
    fn load_single_mesh(&mut self, renderer: &mut Renderer<'_>) {
        let Some(name) = self.mesh_name.clone() else {
            return;
        };

        let (mesh, textures) = match self.files.read_mesh(&name) {
            Ok(loaded) => loaded,
            Err(error) => {
                tracing::warn!("Requested mesh {name}: {error}");
                return;
            }
        };
        let model = match scene::build_model(&mesh, &textures) {
            Ok(model) => model,
            Err(error) => {
                tracing::warn!("Requested mesh {name}: {error}");
                return;
            }
        };

        let instance = renderer::ModelInstance {
            transform: glam::Mat4::from_translation(self.terrain_center).to_cols_array_2d(),
            ..Default::default()
        };
        if let Err(error) = renderer.add_model(&model, &[instance]) {
            tracing::warn!("Requested mesh {name}: {error}");
            return;
        }
        tracing::info!(
            "Showing mesh {name} ({} materials, {} with a texture) at the terrain centre",
            mesh.materials.len(),
            textures.iter().filter(|t| t.is_some()).count(),
        );
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
                    if keycode == KeyCode::Escape {
                        if let Some(window) = &self.window {
                            if self.cursor_locked {
                                window.set_cursor_visible(true);
                                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                                self.cursor_locked = false;
                            } else {
                                window.set_cursor_visible(false);
                                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                                self.cursor_locked = true;
                            }
                        }
                        return Ok(());
                    }
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
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed
                    && button == MouseButton::Left
                    && !self.cursor_locked
                {
                    if let Some(window) = &self.window {
                        window.set_cursor_visible(false);
                        let _ =
                            window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                        self.cursor_locked = true;
                    }
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
            self.camera.fly_speed =
                (self.camera.fly_speed * 1.1_f32.powf(dy)).clamp(0.1, self.terrain_radius * 20.0);
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

        tracing::trace!(
            "Sky state: time={:.2}h, blend={:.2}",
            self.time_of_day,
            sky_blend,
        );

        // Two `c5..c8` matrices, one per convention: the sky and the landscape subtract the
        // camera position from their geometry, so they need a rotation-only view; the static
        // mesh pass gets absolute world positions from the object matrix and needs the full
        // one. See `Camera::camera_relative_view_projection_matrix`.
        let camera_relative_view_proj = self
            .camera
            .camera_relative_view_projection_matrix()
            .to_cols_array_2d();
        let view_proj = self.camera.view_projection_matrix().to_cols_array_2d();

        let window = self.window.as_ref().ok_or(E::NoWindow)?;
        let renderer = self.renderer.as_mut().ok_or(E::NoRenderer)?;

        // Gradient colours are zero until the environment layer lands (AGENTS.md step 2):
        // with alpha 0 the shader's final lrp keeps the raw sky texture, so the
        // unimplemented half is visible rather than faked.
        renderer.update_sky_uniforms(camera_relative_view_proj, [0.0; 4], [0.0; 4], sky_blend);
        renderer.update_terrain_uniforms(camera_relative_view_proj, self.camera.position);
        renderer.update_model_uniforms(view_proj);
        renderer.set_model_camera_pos(self.camera.position);

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
