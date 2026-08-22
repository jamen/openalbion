//! OpenAlbion — a recreation of Fable: The Lost Chapters' engine.
//!
//! The engine binary. Drawing lives in the `renderer` crate, which knows nothing about
//! Fable's asset formats; `scene` is where this binary turns assets into the plain data
//! the renderer accepts. World space is Z-up, matching the game (AGENTS.md §3.6).

mod camera;
mod console;
mod files;
mod scene;
mod text;

use crate::camera::Camera;
use crate::console::{Console, Effect, Input, Subsystem};
use crate::files::{Files, NewFilesError};
use argh::FromArgs;
use derive_more::{Display, Error};
use renderer::{NewRendererError, Renderer};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
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

    /// fable's directory (default: the current working directory)
    #[argh(option)]
    fable_directory: Option<String>,

    /// level to load from FinalAlbion.wad (default: LookoutPoint)
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

    /// open the developer console at startup, instead of on the first backquote (AGENTS.md
    /// §13.5). Works with --screenshot, which is how a console frame gets captured.
    #[argh(switch)]
    console: bool,

    /// run a console command once the level is loaded, before the first frame. Repeatable:
    /// `--command "EnableLandscape false" --command ShowStats`. This is how a subsystem toggle
    /// reaches a `--screenshot`, which never sees a key press.
    #[argh(option)]
    command: Vec<String>,

    /// load every map in FinalAlbion.wld at once instead of --level's region, and report what
    /// the whole world costs (AGENTS.md §12.3b). Needs a release build and patience.
    #[argh(switch)]
    world: bool,
}

/// How much of a level's texture work the bindless registry saves (AGENTS.md §12.6).
///
/// Counted across both upload paths — `.tng` things and local detail's repeated meshes —
/// because they draw from one array and a mesh used by both should show up as one upload.
#[derive(Default)]
struct TextureReuse {
    /// Material texture references seen.
    refs: std::cell::Cell<usize>,
    /// Of those, the ones already resident, which skipped the read *and* the decode.
    reused: std::cell::Cell<usize>,
}

impl TextureReuse {
    /// Record one reference and pass its residency straight back, so counting cannot change
    /// what is counted.
    fn observe(&self, resident: bool) -> bool {
        self.refs.set(self.refs.get() + 1);
        if resident {
            self.reused.set(self.reused.get() + 1);
        }
        resident
    }
}

/// `x,y,z` → a point. Returns `None` for anything else, so a typo falls back to the default
/// framing rather than putting the camera somewhere silently wrong.
fn parse_point(text: &str) -> Option<glam::Vec3> {
    let parts: Vec<f32> = text
        .split(',')
        .filter_map(|p| p.trim().parse().ok())
        .collect();
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

    let mut renderer = pollster::block_on(Renderer::new_headless(SIZE)).map_err(E::NewRenderer)?;

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
    app.run_startup_commands();
    let sky_blend = app.refresh_sky();
    // Twice: the stats overlay reports its own glyph count, which only exists once it has been
    // laid out. A live window gets that for free from the previous frame; a single capture does
    // not. Both calls are no-ops with the console closed, which is the default.
    app.update_console();
    app.update_console();

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
    /// Whether we should be trying to lock the cursor — set on startup/click/Enter, cleared by
    /// Escape. `set_cursor_grab` can fail (observed under Wine: the OS clip/hide calls silently
    /// no-op while `GetActiveWindow` hasn't caught up with a just-received focus event yet), so
    /// this drives a retry on the next `WindowEvent::Focused(true)` rather than trusting the
    /// first attempt.
    cursor_lock_desired: bool,
    /// Frame counter to detect first load.
    first_frame: bool,
    /// The developer console (AGENTS.md §13.5). `None` only if the embedded font failed to
    /// parse, which leaves the engine running without one rather than refusing to start.
    /// Closed by default, so it draws nothing and `--screenshot` is unaffected.
    console: Option<Console>,
    /// `--command` lines, run once the scene is loaded and then taken.
    startup_commands: Vec<String>,
    /// Smoothed frames per second, for the stats overlay to have something that moves.
    fps: f32,
    /// `--world`: load every map at once rather than `--level`'s region.
    load_world: bool,
}

#[derive(Debug, Display)]
enum NewAppError {
    CurrentDir(std::io::Error),
    Files(NewFilesError),
}

impl App {
    fn new(cli: Cli) -> Result<Self, NewAppError> {
        use NewAppError as E;

        let fable_directory = match cli.fable_directory.as_ref() {
            Some(fable_directory) => PathBuf::from(fable_directory),
            None => std::env::current_dir().map_err(E::CurrentDir)?,
        };

        tracing::info!("{}", fable_directory.display());

        let files = Files::new(&fable_directory).map_err(E::Files)?;

        Ok(Self {
            files,
            renderer: None,
            window: None,
            camera: Camera::new(),
            last_frame_time: None,
            time_of_day: 18.0,
            level_name: cli
                .level
                .clone()
                .unwrap_or_else(|| "LookoutPoint".to_string()),
            mesh_name: cli.mesh.clone(),
            terrain_center: glam::Vec3::ZERO,
            terrain_radius: 1.0,
            sky_textures: None,
            keys: HashSet::new(),
            cursor_locked: false,
            cursor_lock_desired: true,
            first_frame: true,
            console: match Console::new() {
                Ok(mut console) => {
                    // `--console` opens it at startup, which is the only way a capture can show
                    // one: `--screenshot` renders a single frame and never sees a key press.
                    if cli.console {
                        console.handle(Input::Toggle);
                    }
                    Some(console)
                }
                Err(error) => {
                    tracing::error!("no developer console: {error}");
                    None
                }
            },
            startup_commands: cli.command.clone(),
            fps: 0.0,
            load_world: cli.world,
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

        // `--command` lines, now that there is a renderer for a toggle to reach.
        self.run_startup_commands();

        Ok(())
    }

    /// Load the level's `.wld` region: every `ContainsMap` populated from its `.tng` and local
    /// detail, every `SeesMap` filler as terrain only, each at its own `MapX`/`MapY` world
    /// position (AGENTS.md §6.12/§5.10) — then frame the camera on the requested level.
    /// Shared by the windowed and offscreen paths so a screenshot is the same scene the
    /// viewer shows.
    ///
    /// A level absent from the `.wld` — or with no `.wld` at all — loads alone, at the world
    /// origin, exactly as a single level always has.
    fn load_scene(&mut self, renderer: &mut Renderer<'_>) -> Result<(), TryResumedError> {
        use TryResumedError as E;

        let region = if self.load_world {
            self.files.world_maps(&self.level_name)
        } else {
            self.files.region_maps(&self.level_name)
        };
        let primary = region
            .iter()
            .find(|m| m.level_name.eq_ignore_ascii_case(&self.level_name))
            .cloned()
            .unwrap_or_else(|| fable_data::wld::RegionMap {
                level_name: self.level_name.clone(),
                origin: (0, 0),
                populated: true,
            });

        // The requested level is the one load failure that's fatal; every other map in its
        // region is best-effort — a missing neighbour narrows the view, it does not stop the
        // primary level from showing.
        let primary_lev = self
            .files
            .load_level(&primary.level_name)
            .map_err(E::LoadLevel)?;

        let mut maps: Vec<(fable_data::wld::RegionMap, fable_data::lev::Lev)> =
            vec![(primary.clone(), primary_lev)];
        for map in region {
            if map.level_name.eq_ignore_ascii_case(&primary.level_name) {
                continue;
            }
            match self.files.load_level(&map.level_name) {
                Ok(lev) => maps.push((map, lev)),
                Err(error) => {
                    tracing::warn!("{}: {error} — dropped from the region", map.level_name);
                }
            }
        }

        // Frame the camera on the requested level specifically, in world space.
        let scale = fable_data::landscape::HEIGHT_SCALE;
        let primary_lev = &maps[0].1;
        let span_x = primary_lev.header.width as f32 + 1.0;
        let span_z = primary_lev.header.height as f32 + 1.0;
        let raw_min = primary_lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::INFINITY, f32::min);
        let raw_max = primary_lev
            .heightmap_cells
            .iter()
            .map(|c| c.height)
            .fold(f32::NEG_INFINITY, f32::max);
        let mid_z = (raw_min + raw_max) * 0.5 * scale;
        let (origin_x, origin_y) = (primary.origin.0 as f32, primary.origin.1 as f32);

        // Z-up: the heightmap spans X/Y and height is Z. AGENTS.md §3.6.
        self.terrain_center =
            glam::Vec3::new(origin_x + span_x * 0.5, origin_y + span_z * 0.5, mid_z);
        self.terrain_radius = span_x.max(span_z) * 0.5;

        // The far plane and fly speed must cover every loaded map, not just the primary one,
        // or a region's filler hills at the edge get clipped.
        let mut world_min = glam::Vec2::new(origin_x, origin_y);
        let mut world_max = glam::Vec2::new(origin_x + span_x, origin_y + span_z);
        for (map, lev) in &maps[1..] {
            let (mx, my) = (map.origin.0 as f32, map.origin.1 as f32);
            world_min = world_min.min(glam::Vec2::new(mx, my));
            world_max = world_max.max(glam::Vec2::new(
                mx + lev.header.width as f32 + 1.0,
                my + lev.header.height as f32 + 1.0,
            ));
        }
        let world_span = (world_max - world_min)
            .max_element()
            .max((raw_max - raw_min).abs() * scale);
        self.camera.set_world_extents(world_span);
        // Position camera above and back from the terrain centre for a good initial view.
        self.camera.position = self.terrain_center
            + glam::Vec3::new(world_span * 0.3, world_span * 0.5, world_span * 0.4);
        self.camera.look_at(self.terrain_center);
        self.camera.fly_speed = world_span * 0.1;

        // Every map's terrain, built and stitched across its region in one call — see
        // `scene::build_region_terrain` for how a boundary vertex reads the neighbouring
        // map's height, normal and theme instead of clamping to its own edge (AGENTS.md
        // §3.4/§6.12).
        // Before `set_terrain`, not after: the bindless registry is scene-scoped, and
        // terrain registers its ground textures and blend tables inside `set_terrain`
        // (AGENTS.md §12.4).
        renderer.clear_scene();
        renderer.set_terrain(&scene::build_region_terrain(&mut self.files, &maps));
        tracing::info!(
            "Uploaded terrain to GPU: {} map(s) ({} populated), primary {} at {:?}, \
             center=({:.1}, {:.1}, {:.1}), radius={:.1}, world_span={world_span:.1}",
            maps.len(),
            maps.iter().filter(|(m, _)| m.populated).count(),
            primary.level_name,
            primary.origin,
            self.terrain_center.x,
            self.terrain_center.y,
            self.terrain_center.z,
            self.terrain_radius,
        );

        // Populate the region from its `.tng`s. Never fatal — unresolvable things still
        // leave the landscape showing.
        self.load_things(renderer, &maps);

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

    /// Populate the region: every `.tng` thing whose def draws a static mesh, plus the local
    /// detail its ground themes generate, at their own transforms and one upload per mesh —
    /// across every populated (`ContainsMap`) map in `maps`. A `SeesMap` filler contributes
    /// terrain only (AGENTS.md §3.4): the original never loads a `GameMap` for one either, so
    /// it stays unpopulated here too.
    ///
    /// Nothing here is fatal. A level that cannot resolve some of its things should still
    /// render the rest and say what it dropped — a missing object must be visible in the log
    /// even when it is invisible on screen.
    fn load_things(
        &mut self,
        renderer: &mut Renderer<'_>,
        maps: &[(fable_data::wld::RegionMap, fable_data::lev::Lev)],
    ) {

        if self.mesh_name.is_some() {
            self.load_single_mesh(renderer);
            return;
        }

        let mut all_things = Vec::new();
        let mut all_local_detail = Vec::new();

        for (map, lev) in maps {
            if !map.populated {
                continue;
            }

            match self.files.load_tng(&map.level_name) {
                Ok(tng) => {
                    all_things.push(scene::resolve_things(
                        &tng,
                        &self.files.thing_graphics,
                        &self.files.creature_body_parts,
                        map.origin,
                    ));
                }
                Err(error) => {
                    tracing::warn!("No .tng for {}: {error} — it will be bare", map.level_name);
                }
            }

            all_local_detail.push(scene::build_local_detail(&self.files, lev, map.origin));
        }

        let things = scene::merge_things(all_things);
        let resolved_placements = things.placement_count();

        // A thing and a local detail object are the same kind of draw — the engine puts both
        // through `AddStaticMesh` — so they share one instance buffer per mesh rather than
        // two passes over the same asset. The kind rides along per mesh so the two mesh
        // toggles can gate independently; it does not change the draw.
        let mut instances_by_mesh: HashMap<u32, (renderer::ModelKind, Vec<renderer::ModelInstance>)> =
            things
                .by_mesh
                .iter()
                .map(|(&mesh_id, group)| {
                    let instances = group
                        .placements
                        .iter()
                        .map(|p| renderer::ModelInstance {
                            transform: p.transform,
                            // The per-object colour (`c0`) is opaque white until fade distance
                            // lands; that leaves the material exactly as authored.
                            ..Default::default()
                        })
                        .collect();
                    (mesh_id, (group.kind, instances))
                })
                .collect();

        // Provenance: which defs became which mesh (AGENTS.md §6.8).
        let mut sources: HashMap<u32, BTreeSet<String>> = HashMap::new();
        for (&mesh_id, group) in &things.by_mesh {
            let entry = sources.entry(mesh_id).or_default();
            for placement in &group.placements {
                entry.insert(placement.definition_type.clone());
            }
        }

        // What the bindless registry saves, counted rather than asserted (AGENTS.md §12.6):
        // every material texture reference across *both* upload paths, and how many were
        // already resident and so skipped the archive read and the BC decode entirely.
        let textures = TextureReuse::default();

        let local_detail = scene::merge_local_detail(all_local_detail);
        self.load_repeated_meshes(renderer, &local_detail, &textures);
        for (mesh_id, mut objects) in local_detail.by_mesh {
            sources
                .entry(mesh_id)
                .or_default()
                .insert("local detail".to_string());
            // Local detail's mesh objects are literally `AddStaticMesh` calls in the engine
            // (AGENTS.md §3.13), so they answer to `EnableStaticMeshes`.
            instances_by_mesh
                .entry(mesh_id)
                .or_insert_with(|| (renderer::ModelKind::Static, Vec::new()))
                .1
                .append(&mut objects);
        }

        let mut uploaded_meshes = 0usize;
        let mut placed = 0usize;
        let mut failed_meshes = 0usize;

        // Deterministic order so two runs log the same thing.
        let mut mesh_ids: Vec<u32> = instances_by_mesh.keys().copied().collect();
        mesh_ids.sort_unstable();

        for mesh_id in mesh_ids {
            let (kind, instances) = &instances_by_mesh[&mesh_id];
            let name = self
                .files
                .mesh_name_by_id(mesh_id)
                .unwrap_or_else(|| format!("#{mesh_id}"));

            let (mesh, material_textures) = match self.files.read_mesh_by_id(mesh_id) {
                Ok(loaded) => loaded,
                Err(error) => {
                    tracing::warn!(
                        "Mesh {name} ({mesh_id}): {error} — {} placements dropped",
                        instances.len()
                    );
                    failed_meshes += 1;
                    continue;
                }
            };

            // A material with no resolvable texture draws white rather than dropping the
            // whole mesh: roughly a quarter of the materials in graphics.big have no base
            // texture at all, and a silently absent object is worse than an untextured one.
            let model = match scene::build_model(&mesh, &material_textures, |id| {
                textures.observe(renderer.has_texture(id))
            }) {
                Ok(model) => model,
                Err(error) => {
                    tracing::warn!(
                        "Mesh {name}: {error} — {} placements dropped",
                        instances.len()
                    );
                    failed_meshes += 1;
                    continue;
                }
            };

            match renderer.add_model(&model, instances, *kind) {
                Ok(()) => {
                    uploaded_meshes += 1;
                    placed += instances.len();
                    tracing::debug!(
                        "{name} (id {mesh_id}) ← {} {kind:?} placements from {:?}",
                        instances.len(),
                        sources.get(&mesh_id).map(|s| s.iter().collect::<Vec<_>>()),
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        "Mesh {name}: {error} — {} placements dropped",
                        instances.len()
                    );
                    failed_meshes += 1;
                }
            }
        }

        let (registered, capacity) = renderer.bindless_stats();
        tracing::info!(
            "Textures: {registered}/{capacity} bindless slots; {} of {} material references \
             were already resident and skipped the read and decode",
            textures.reused.get(),
            textures.refs.get(),
        );

        let skipped = &things.skipped;
        tracing::info!(
            "Things: placed {placed} of {} placements over \
             {uploaded_meshes} meshes ({failed_meshes} meshes failed); skipped {} things — \
             {} no def, {} not drawable, {} without a placement, {} kind conflicts, \
             {:?} by graphic type",
            resolved_placements + local_detail.counts.drawn_as_models(),
            skipped.total(),
            skipped.no_def,
            skipped.not_drawable,
            skipped.no_placement,
            skipped.kind_conflict,
            skipped.other_graphic_type,
        );

        // Creatures and the other skinned things, counted on their own: they are the half of
        // this that draws in bind pose, and the number is what says whether a level got any.
        // Resolved and uploaded are both reported, because a mesh that failed to decode would
        // otherwise vanish between them without saying so.
        let kind = renderer::ModelKind::Animated;
        let (uploaded_animated, drawn_animated) = renderer.model_stats_of(kind);
        tracing::info!(
            "Animating meshes: {drawn_animated} of {} placements over {uploaded_animated} of {} \
             meshes, drawn in bind pose (no bones — AGENTS.md §5 step 6.7)",
            things.placement_count_of(kind),
            things.mesh_count_of(kind),
        );

        let body_parts = things.body_parts;
        tracing::info!(
            "Body parts: {} creatures assembled from {} part meshes instead of their base body \
             (AGENTS.md §3.16)",
            body_parts.creatures,
            body_parts.placements,
        );
    }

    /// Upload local detail's repeated meshes — the grass, bracken and flowers, which outnumber
    /// everything else in a level by two orders of magnitude.
    ///
    /// Their own pass, because the original gives them their own shaders: one draw per
    /// `(mesh, AlphaRef)` batch however many thousands of instances it carries.
    fn load_repeated_meshes(
        &mut self,
        renderer: &mut Renderer<'_>,
        local_detail: &scene::LevelLocalDetail,
        textures: &TextureReuse,
    ) {
        // Deterministic order so two runs log the same thing.
        let mut keys: Vec<(u32, i32)> = local_detail.repeated.keys().copied().collect();
        keys.sort_unstable();

        let mut dropped = 0usize;
        for key in keys {
            let (mesh_id, alpha_ref) = key;
            let instances = &local_detail.repeated[&key];
            let name = self
                .files
                .mesh_name_by_id(mesh_id)
                .unwrap_or_else(|| format!("#{mesh_id}"));

            let built = self
                .files
                .read_mesh_by_id(mesh_id)
                .map_err(|e| e.to_string())
                .and_then(|(mesh, material_textures)| {
                    scene::build_model(&mesh, &material_textures, |id| {
                        textures.observe(renderer.has_texture(id))
                    })
                    .map_err(|e| e.to_string())
                });
            let model = match built {
                Ok(model) => model,
                Err(error) => {
                    tracing::warn!(
                        "Repeated mesh {name}: {error} — {} dropped",
                        instances.len()
                    );
                    dropped += instances.len();
                    continue;
                }
            };

            // `AlphaRef` is the object type's, not the material's, and it is read rather than
            // invented — unlike the static mesh pass's cutoff (AGENTS.md §9).
            let cutoff = alpha_ref as f32 / 255.0;
            match renderer.add_local_detail(&model, instances, cutoff) {
                Ok(()) => tracing::debug!(
                    "{name} (id {mesh_id}) ← {} repeated instances, alpha ref {alpha_ref}",
                    instances.len(),
                ),
                Err(error) => {
                    tracing::warn!(
                        "Repeated mesh {name}: {error} — {} dropped",
                        instances.len()
                    );
                    dropped += instances.len();
                }
            }
        }

        let (meshes, drawn) = renderer.local_detail_stats();
        tracing::info!(
            "Local detail: {drawn} repeated instances over {meshes} batches ({dropped} dropped)",
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
        let model = match scene::build_model(&mesh, &textures, |id| renderer.has_texture(id)) {
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
        // There is no def here to read a graphic type from, so the mesh's own `animated` flag
        // decides which toggle it answers to. A skinned mesh still draws in bind pose.
        let kind = if mesh.animated {
            renderer::ModelKind::Animated
        } else {
            renderer::ModelKind::Static
        };
        if let Err(error) = renderer.add_model(&model, &[instance], kind) {
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
            WindowEvent::KeyboardInput { ref event, .. }
                if event.state == ElementState::Pressed && self.console_key(event) =>
            {
                // The console took it. Nothing below runs, which is what stops a `w` typed at
                // the prompt from also flying the camera forward.
            }
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
                        if self.cursor_locked {
                            self.unlock_cursor();
                        } else {
                            self.cursor_lock_desired = true;
                            self.try_lock_cursor();
                        }
                        return Ok(());
                    }
                    if keycode == KeyCode::Enter && !self.cursor_locked {
                        self.cursor_lock_desired = true;
                        self.try_lock_cursor();
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
                    self.cursor_lock_desired = true;
                    self.try_lock_cursor();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // When cursor is unlocked, we use absolute position delta
                // (handled via raw device events for locked mode).
                let _ = position;
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // The console asks for its glyphs at `16 px × scale_factor`, and `px` is part of
                // the cache key — so a scale change re-rasterizes at the new size with no
                // invalidation of its own (AGENTS.md §13.6).
                if let Some(console) = self.console.as_mut() {
                    console.set_scale(scale_factor as f32);
                }
            }
            WindowEvent::Focused(true) => {
                // A grab attempted before the window was focused (startup, or a click that
                // hadn't yet activated the window) can fail without erroring — `set_cursor_grab`
                // silently no-ops on Windows while `GetActiveWindow` doesn't yet agree this
                // window is active. Retry now that we have a real activation transition.
                if self.cursor_lock_desired && !self.cursor_locked {
                    self.try_lock_cursor();
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Offer one key press to the console. Returns whether it was taken.
    ///
    /// **Backquote is checked before anything else**, because the platform reports the same
    /// press as both a physical key *and* the text "`" — so handling it second would open the
    /// console and immediately type into it.
    ///
    /// Printable characters come from `KeyEvent::text` rather than the physical key, which is
    /// the only way a non-US layout types what its keycaps say: the platform has already applied
    /// the layout and the modifiers. Editing keys stay physical, because their meaning does not
    /// depend on the layout.
    fn console_key(&mut self, event: &KeyEvent) -> bool {
        if self.console.is_none() {
            return false;
        }
        let was_open = self.console.as_ref().is_some_and(Console::is_open);
        let code = match event.physical_key {
            PhysicalKey::Code(code) => Some(code),
            PhysicalKey::Unidentified(_) => None,
        };

        let input = if code == Some(KeyCode::Backquote) {
            Input::Toggle
        } else if !was_open {
            return false;
        } else {
            match code {
                Some(KeyCode::Escape) => Input::Close,
                Some(KeyCode::Enter | KeyCode::NumpadEnter) => Input::Enter,
                Some(KeyCode::Backspace) => Input::Backspace,
                Some(KeyCode::Delete) => Input::Delete,
                Some(KeyCode::ArrowLeft) => Input::Left,
                Some(KeyCode::ArrowRight) => Input::Right,
                Some(KeyCode::Home) => Input::Home,
                Some(KeyCode::End) => Input::End,
                Some(KeyCode::ArrowUp) => Input::HistoryPrev,
                Some(KeyCode::ArrowDown) => Input::HistoryNext,
                Some(KeyCode::PageUp) => Input::ScrollUp,
                Some(KeyCode::PageDown) => Input::ScrollDown,
                // Everything else is either text or nothing — a modifier on its own reports no
                // text, and is swallowed rather than reaching the camera.
                _ => match event.text.as_ref() {
                    Some(text) => Input::Text(text.to_string()),
                    None => return true,
                },
            }
        };

        let effect = self
            .console
            .as_mut()
            .expect("checked above")
            .handle(input);

        // Opening the console releases the mouse; closing it takes it back. Held movement keys
        // are dropped either way, or a `w` held as the console opens would fly forever.
        let is_open = self.console.as_ref().is_some_and(Console::is_open);
        if is_open != was_open {
            self.keys.clear();
            if is_open {
                self.unlock_cursor();
            } else {
                self.cursor_lock_desired = true;
                self.try_lock_cursor();
            }
        }

        if let Some(effect) = effect {
            self.apply_effect(effect);
        }

        true
    }

    /// Run the `--command` lines, once, after the scene is loaded and the renderer exists —
    /// `EnableSky false` has nowhere to go before that.
    fn run_startup_commands(&mut self) {
        for line in std::mem::take(&mut self.startup_commands) {
            let effect = self.console.as_mut().and_then(|c| c.run_line(&line));
            if let Some(effect) = effect {
                self.apply_effect(effect);
            }
        }
    }

    /// Perform what the console handed back, and report the result into its scrollback.
    ///
    /// The console cannot reach the renderer itself, deliberately (AGENTS.md §11.1) — so this is
    /// where `EnableSky false` becomes a `RenderToggles` and where `Stats` gets its numbers.
    fn apply_effect(&mut self, effect: Effect) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        let message = match effect {
            Effect::Enable(subsystem, value) => {
                let mut toggles = renderer.toggles();
                let field = match subsystem {
                    Subsystem::Sky => &mut toggles.sky,
                    Subsystem::Landscape => &mut toggles.landscape,
                    Subsystem::StaticMeshes => &mut toggles.static_meshes,
                    Subsystem::AnimatedMeshes => &mut toggles.animated_meshes,
                    Subsystem::RepeatedMeshes => &mut toggles.repeated_meshes,
                };
                *field = value.unwrap_or(!*field);
                let now = *field;
                renderer.set_toggles(toggles);
                format!("Enable{} {now}", subsystem.name())
            }
            Effect::Stats => self.stats_lines().join("\n"),
        };

        if let Some(console) = self.console.as_mut() {
            console.println(message);
        }
    }

    /// What `Stats` prints and what `ShowStats` keeps on screen — one source, so the two cannot
    /// disagree.
    ///
    /// These are the numbers `--text-demo` used to show, and they are still worth showing for
    /// the reason it existed: the bindless slot count fills in as you fly and then stops, which
    /// is the glyph cache working and is invisible in a still image (§13.7 step 3).
    fn stats_lines(&self) -> Vec<String> {
        let (slots, capacity) = self.renderer.as_ref().map_or((0, 0), Renderer::bindless_stats);
        let (meshes, placements) = self.renderer.as_ref().map_or((0, 0), Renderer::model_stats);
        let (animated_meshes, animated_placements) = self
            .renderer
            .as_ref()
            .map_or((0, 0), |r| r.model_stats_of(renderer::ModelKind::Animated));
        let (_, foliage) = self
            .renderer
            .as_ref()
            .map_or((0, 0), Renderer::local_detail_stats);
        let toggles = self
            .renderer
            .as_ref()
            .map_or_else(Default::default, Renderer::toggles);

        let mut off: Vec<&str> = Vec::new();
        for (on, name) in [
            (toggles.sky, "Sky"),
            (toggles.landscape, "Landscape"),
            (toggles.static_meshes, "StaticMeshes"),
            (toggles.animated_meshes, "AnimatedMeshes"),
            (toggles.repeated_meshes, "RepeatedMeshes"),
        ] {
            if !on {
                off.push(name);
            }
        }

        vec![
            format!("level     {}", self.level_name),
            format!(
                "camera    {:9.1} {:9.1} {:9.1}",
                self.camera.position.x, self.camera.position.y, self.camera.position.z,
            ),
            format!("time      {:5.2}h", self.time_of_day),
            format!(
                "fps       {}",
                if self.fps > 0.0 {
                    format!("{:.0}", self.fps)
                } else {
                    "n/a".to_string()
                },
            ),
            format!("bindless  {slots}/{capacity} slots"),
            format!("meshes    {meshes} assets, {placements} placements"),
            format!("animated  {animated_meshes} assets, {animated_placements} placements (bind pose)"),
            format!("foliage   {foliage} instances"),
            // The previous frame's, necessarily: this line is part of what gets counted, so a
            // number describing the frame it appears in cannot be known before it is laid out.
            format!(
                "glyphs    {} quads, one draw",
                self.renderer.as_ref().map_or(0, Renderer::text_stats),
            ),
            format!(
                "disabled  {}",
                if off.is_empty() {
                    "none".to_string()
                } else {
                    off.join(" ")
                },
            ),
        ]
    }

    /// Attempts to hide and grab the cursor, only committing `cursor_locked` on success.
    /// `set_cursor_grab`/`set_cursor_visible` can fail silently on Windows when the window
    /// hasn't been marked active yet — see `cursor_lock_desired`, which drives a retry.
    fn try_lock_cursor(&mut self) {
        let Some(window) = &self.window else { return };
        window.set_cursor_visible(false);
        match window.set_cursor_grab(winit::window::CursorGrabMode::Locked) {
            Ok(()) => self.cursor_locked = true,
            Err(error) => {
                window.set_cursor_visible(true);
                tracing::warn!("Cursor grab failed, will retry on next focus: {error}");
            }
        }
    }

    fn unlock_cursor(&mut self) {
        self.cursor_lock_desired = false;
        let Some(window) = &self.window else { return };
        window.set_cursor_visible(true);
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
        self.cursor_locked = false;
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
            self.try_lock_cursor();
        }

        // Fly camera: process input.
        let speed_mult = if self.keys.contains(&KeyCode::ShiftLeft)
            || self.keys.contains(&KeyCode::ShiftRight)
        {
            3.0
        } else {
            1.0
        };

        // The console holds the keyboard while it is open. `console_key` already swallows the
        // presses and clears held keys, so this is belt and braces — but it is the one place
        // that says *why* the camera goes still, rather than leaving it as a consequence of an
        // empty key set.
        let flying = !self.console.as_ref().is_some_and(Console::captures_input);

        self.camera.fly(
            delta_time,
            (
                flying && self.keys.contains(&KeyCode::KeyW),
                flying && self.keys.contains(&KeyCode::KeyS),
                flying && self.keys.contains(&KeyCode::KeyA),
                flying && self.keys.contains(&KeyCode::KeyD),
                flying && self.keys.contains(&KeyCode::Space),
                flying
                    && (self.keys.contains(&KeyCode::ControlLeft)
                        || self.keys.contains(&KeyCode::ControlRight)),
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

        // Smoothed, so the number is readable rather than flickering every frame.
        if delta_time > 0.0 {
            let instant = 1.0 / delta_time;
            self.fps = if self.fps > 0.0 {
                self.fps * 0.9 + instant * 0.1
            } else {
                instant
            };
        }
        self.update_console();

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

impl App {
    /// Refresh the stats overlay and hand the console's glyphs to the renderer.
    ///
    /// Called every frame. With the console closed and `ShowStats` off — the default — the
    /// layout is empty and this hands the renderer an empty batch, which draws nothing
    /// (AGENTS.md §13.6).
    fn update_console(&mut self) {
        // Built here rather than in `layout` because the console has no camera and no renderer
        // to ask, and only when it is on screen: a live console is not a reason to format nine
        // strings a frame.
        if self.console.as_ref().is_some_and(Console::show_stats) {
            let lines = self.stats_lines();
            if let Some(console) = self.console.as_mut() {
                console.set_stats(lines);
            }
        }

        if let (Some(renderer), Some(console)) = (self.renderer.as_mut(), self.console.as_ref()) {
            console::draw(renderer, console);
        }
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
