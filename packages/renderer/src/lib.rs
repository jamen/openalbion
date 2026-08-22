//! OpenAlbion's renderer: wgpu passes and the data they draw.
//!
//! Nothing here reads a file or knows an asset format — the dependency list is wgpu,
//! glam, bytemuck and tracing, and that is the point. Every input arrives as a plain
//! struct ([`TerrainData`], [`Model`], [`TextureImage`]), so a pass *cannot* invent a
//! data lookup of its own; AGENTS.md §2 rule 3 becomes a property the compiler checks
//! rather than one a reviewer has to notice.
//!
//! Turning Fable's archives into these types is `openalbion::scene`'s job. Anything that
//! can produce them can drive this renderer, which is what lets tools other than the
//! game — the `mirror` harness, a world editor — share it.
//!
//! World space is Z-up, matching the game (AGENTS.md §3.6).

mod bindless;
mod depth;
mod image;
mod lighting;
mod local_detail;
mod model;
mod sky;
mod terrain;
mod text;
mod texture;

use self::bindless::{BindlessTextures, TextureKey};
use self::depth::DepthTexture;
use self::local_detail::LocalDetailPass;
use self::model::ModelPass;
use self::sky::OuterSkyPass;
use self::terrain::TerrainPass;
use self::text::TextPass;
use derive_more::{Display, Error};
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoder, CompositeAlphaMode, CreateSurfaceError, Device,
    DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, MapMode, PresentMode, Queue,
    RequestAdapterError, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTarget, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

pub use self::bindless::{MAX_BINDLESS_TEXTURES, MIN_BINDLESS_TEXTURES};
pub use self::image::{ImageFormat, TextureImage};
pub use self::local_detail::{AddLocalDetailError, LocalDetailInstance};
pub use self::model::{
    AddModelError, AlphaMode, Model, ModelInstance, ModelKind, ModelKinds, ModelMaterial,
    ModelPrimitive, ModelSubMesh, ModelVertex,
};
pub use self::terrain::{TerrainData, TerrainDraw, TerrainVertex};
pub use self::text::{AddGlyphError, GlyphInstance};

/// 4× MSAA, the sample count every pass is built for when the adapter supports it.
///
/// **A deliberate divergence** (AGENTS.md §3.12, §6.3 `ACCEPTED`). Multisampling is
/// device-level in the original — `CDisplayManager::SetDisplayMode` takes an
/// `ESurfaceMultisampleType` and `ConsoleSetAntialiasing` re-creates the device — and the
/// shipped configuration has it **off**: both `SetAntialiasing` lines in `~/Fable/dbugst.ini`
/// are commented out. We turn it on because it looks better, not because the game does it.
const MSAA_SAMPLES: u32 = 4;

/// The attachment formats and sample count every pipeline in a frame must agree on.
///
/// Bundled rather than passed separately because a pipeline that disagrees with the
/// attachments on *any* of the three fails at draw time, not at creation — so they travel
/// together and are set in one place.
#[derive(Copy, Clone, Debug)]
pub(crate) struct TargetFormats {
    pub colour: TextureFormat,
    pub depth: TextureFormat,
    pub sample_count: u32,
}

impl TargetFormats {
    /// The `MultisampleState` every pipeline built for these targets must use.
    pub fn multisample(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: self.sample_count,
            ..Default::default()
        }
    }

    /// The `MultisampleState` for a pipeline that draws **after** [`ResolvePass`], into the
    /// presentable texture rather than the multisampled one.
    ///
    /// Always one sample, whatever `sample_count` says — and it is a method rather than an
    /// inline `count: 1` so the exception is stated where the rule is. Only [`text::TextPass`]
    /// uses it: glyphs arrive already antialiased by coverage, so running them through MSAA
    /// would spend samples to change nothing, and the resolve has happened by then regardless
    /// (AGENTS.md §13.6).
    pub fn post_resolve_multisample(&self) -> wgpu::MultisampleState {
        wgpu::MultisampleState::default()
    }

    /// Whether drawing goes through a multisampled texture that must then be resolved.
    pub fn is_multisampled(&self) -> bool {
        self.sample_count > 1
    }
}

/// Where a `Renderer` draws. Windowed presentation and offscreen capture share every pass;
/// only the colour attachment and what happens after submit differ.
enum Target<'t> {
    Surface(Surface<'t>),
    /// Owned colour texture plus a `MAP_READ` staging buffer, for headless capture.
    Offscreen {
        texture: Texture,
        readback: wgpu::Buffer,
        size: [u32; 2],
        padded_bytes_per_row: u32,
    },
}

pub struct Renderer<'target> {
    device: Device,
    queue: Queue,
    format: TextureFormat,
    targets: TargetFormats,
    depth_texture: DepthTexture,
    /// The multisampled colour attachment every pass draws into, resolved into the
    /// presentable texture at the end of the frame. `None` when MSAA is unavailable, in which
    /// case the passes draw into the presentable texture directly.
    msaa_texture: Option<MsaaTexture>,
    passes: RenderPasses,
    /// The one texture array every pass draws through (AGENTS.md §12).
    ///
    /// **Scene-scoped**: `clear_scene` drops every registration, so a `BindlessIndex` is valid
    /// only from one clear to the next. A pass that registers outside a scene load — the sky
    /// does, on a keyframe change — carries the generation it registered under and checks it
    /// at draw time (§12.4).
    bindless: BindlessTextures,
    /// Which world passes run this frame (AGENTS.md §13.5). All on by default.
    toggles: RenderToggles,
    /// The target's size in physical pixels. Kept because screen-space passes need it and
    /// `resize_surface` is the only place it is known — it used to be consumed and dropped.
    size: [u32; 2],
    target: Target<'target>,
}

/// Which world subsystems draw — the renderer half of the console's `Enable*` commands
/// (AGENTS.md §13.5).
///
/// **Subsystem isolation is what made comparing against the original possible at all** (§3.9):
/// every `CEngineComponent` in Fable has a `GetConsoleEnableFunctionName`, and the same trick
/// answers "is that seam the terrain or the foliage?" here without a reference image. It is a
/// struct rather than four setters so a caller reads and writes the whole state — the console
/// needs to *report* a toggle as well as flip it.
///
/// A toggle skips a pass entirely; it does not draw it differently. That is only safe because
/// [`ClearPass`] owns the depth clear (§13.7 step 1) — while `TerrainPass` owned it, switching
/// the landscape off would have left the passes after it depth-testing against the previous
/// frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RenderToggles {
    /// `EnableSky` — the outer sky dome and base band.
    pub sky: bool,
    /// `EnableLandscape` — the terrain's foreground layers.
    pub landscape: bool,
    /// `EnableStaticMeshes` — the `.tng` things.
    pub static_meshes: bool,
    /// `EnableAnimatedMeshes` — the skinned things, drawn in bind pose.
    pub animated_meshes: bool,
    /// `EnableRepeatedMeshes` — local detail's foliage.
    pub repeated_meshes: bool,
}

impl Default for RenderToggles {
    fn default() -> Self {
        Self {
            sky: true,
            landscape: true,
            static_meshes: true,
            animated_meshes: true,
            repeated_meshes: true,
        }
    }
}

/// The multisampled colour attachment. Recreated on resize alongside the depth buffer, which
/// must always match it in sample count as well as size.
struct MsaaTexture {
    #[allow(dead_code)]
    texture: Texture,
    view: TextureView,
}

impl MsaaTexture {
    fn new(device: &Device, format: TextureFormat, size: [u32; 2], sample_count: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("msaa_colour"),
            size: Extent3d {
                width: size[0].max(1),
                height: size[1].max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}

/// Everything the renderer requires of a device, and it is a **hard requirement** — there is
/// no non-bindless fallback path (AGENTS.md §12.2). Keeping two texture-binding paths alive
/// would reintroduce exactly the per-pass duplication bindless exists to remove, and
/// `TEXTURE_COMPRESSION_BC` was already a hard requirement, so this is the same kind of
/// demand rather than a new kind. The cost is GL and WebGPU, neither of which this targets.
///
/// `SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING` is requested from day one
/// even though nothing needs it yet: every pass draws one material per draw, so its texture
/// index is dynamically uniform and `TEXTURE_BINDING_ARRAY` alone would do. Only §13's glyph
/// batching indexes per instance. It is supported on exactly the platforms
/// `TEXTURE_BINDING_ARRAY` is, so asking now costs nothing and saves a second migration.
const REQUIRED_FEATURES: Features = Features::TEXTURE_COMPRESSION_BC
    .union(Features::TEXTURE_BINDING_ARRAY)
    .union(Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
    .union(Features::IMMEDIATES);

/// Bytes of per-draw immediate data (wgpu's name for push constants) a pipeline may declare.
///
/// Sized by the largest pass: terrain's is 48 bytes, because a layer pass binds a *pair* of
/// textures and carries the planar projection for its mapping direction (§3.4). The model and
/// local-detail passes use 16. Vulkan guarantees 128, DX12 256, Metal 4096.
const MAX_IMMEDIATE_SIZE: u32 = 48;

/// The device request, shared by the windowed and offscreen paths so they cannot drift.
///
/// `required_limits` is the trap worth naming: it defaults to `Limits::default()`, whose
/// binding-array and immediate limits are **0**, and wgpu validates against *what was asked
/// for*, never what the adapter can do — so a granted `TEXTURE_BINDING_ARRAY` plus default
/// limits creates a device on which the array's bind group layout cannot be built
/// (AGENTS.md §12.2).
async fn request_device(adapter: &wgpu::Adapter) -> Result<(Device, Queue, u32), NewRendererError> {
    let missing = REQUIRED_FEATURES - adapter.features();
    if !missing.is_empty() {
        return Err(NewRendererError::MissingFeatures(missing));
    }

    let adapter_limits = adapter.limits();
    let capacity = bindless::BindlessTextures::capacity_for(
        adapter_limits.max_binding_array_elements_per_shader_stage,
    )
    .ok_or(NewRendererError::BindingArrayTooSmall(
        adapter_limits.max_binding_array_elements_per_shader_stage,
    ))?;
    if adapter_limits.max_immediate_size < MAX_IMMEDIATE_SIZE {
        return Err(NewRendererError::ImmediatesTooSmall(
            adapter_limits.max_immediate_size,
        ));
    }

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            required_features: REQUIRED_FEATURES,
            required_limits: wgpu::Limits {
                max_binding_array_elements_per_shader_stage: capacity,
                max_immediate_size: MAX_IMMEDIATE_SIZE,
                // Whatever the adapter allows, rather than the 256 MiB default
                // (`limits.rs:383`). One region's terrain fits in the default comfortably; the
                // *whole world* does not — all 398 maps in one scene want a 374 MB vertex
                // buffer, and asking for the default turns that into a validation error at
                // `create_buffer` rather than an out-of-memory. This machine's RADV reports
                // ~4 GB. Raising it costs nothing on a device that cannot honour it, because
                // this takes the adapter's own number.
                max_buffer_size: adapter_limits.max_buffer_size,
                ..wgpu::Limits::default()
            },
            ..Default::default()
        })
        .await
        .map_err(NewRendererError::RequestDevice)?;

    // Assert what was granted rather than assume it. Requesting a feature and getting a
    // device is not proof the device has it, and a limit silently left at its default is the
    // failure this whole function exists to prevent — it would surface much later, as a bind
    // group layout that will not validate.
    let granted = device.limits();
    assert!(
        device.features().contains(REQUIRED_FEATURES),
        "device was created without the features it was asked for: missing {:?}",
        REQUIRED_FEATURES - device.features(),
    );
    assert!(
        granted.max_binding_array_elements_per_shader_stage >= capacity
            && granted.max_immediate_size >= MAX_IMMEDIATE_SIZE,
        "device limits were not granted: binding array {} (wanted {capacity}), immediates {} \
         (wanted {MAX_IMMEDIATE_SIZE})",
        granted.max_binding_array_elements_per_shader_stage,
        granted.max_immediate_size,
    );
    tracing::info!(
        "Device: bindless {capacity} textures, {MAX_IMMEDIATE_SIZE}B immediates, \
         non-uniform indexing available",
    );

    Ok((device, queue, capacity))
}

/// The sample count to build for: [`MSAA_SAMPLES`] when the adapter supports it for *both*
/// the colour and depth formats, otherwise 1.
///
/// Checked rather than assumed. A pipeline built for a sample count the format cannot carry
/// fails at texture creation, and the fallback is a working frame without antialiasing —
/// which is the original's own configuration anyway (AGENTS.md §3.12).
fn supported_sample_count(
    adapter: &wgpu::Adapter,
    colour: TextureFormat,
    depth: TextureFormat,
) -> u32 {
    let supported = |format: TextureFormat| {
        adapter
            .get_texture_format_features(format)
            .flags
            .sample_count_supported(MSAA_SAMPLES)
    };

    if supported(colour) && supported(depth) {
        tracing::info!("MSAA: {MSAA_SAMPLES}x");
        MSAA_SAMPLES
    } else {
        tracing::warn!(
            "MSAA: disabled — the adapter does not support {MSAA_SAMPLES} samples for \
             {colour:?} and {depth:?}"
        );
        1
    }
}

impl<'target> Renderer<'target> {
    pub async fn new(target: impl Into<SurfaceTarget<'target>>) -> Result<Self, NewRendererError> {
        use NewRendererError as E;

        let instance = Instance::new(&InstanceDescriptor::default());

        let surface = instance.create_surface(target).map_err(E::CreateSurface)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(E::RequestAdapter)?;

        let (device, queue, bindless_capacity) = request_device(&adapter).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Prefer a non-sRGB surface. Textures upload as *Unorm (linear, no sRGB decode),
        // so an sRGB target would encode without a matching decode and wash out every
        // colour. The original is D3D9 with no sRGB framebuffer. AGENTS.md §3.5.
        //
        // Take the surface's *preferred* format and strip the sRGB suffix, rather than
        // scanning for any non-sRGB format — the capability list can lead with exotic
        // entries (Rgba16Unorm here) that need features we do not request.
        let preferred = surface_capabilities
            .formats
            .first()
            .copied()
            .unwrap_or(TextureFormat::Rgba8Unorm);
        let stripped = preferred.remove_srgb_suffix();
        let surface_format = if surface_capabilities.formats.contains(&stripped) {
            stripped
        } else {
            preferred
        };

        if surface_format.is_srgb() {
            tracing::warn!(
                "No non-sRGB surface format available; colours will be sRGB-encoded \
                 without a matching decode (AGENTS.md §3.5)"
            );
        }
        tracing::info!("Surface format: {surface_format:?}");

        let targets = TargetFormats {
            colour: surface_format,
            depth: DepthTexture::FORMAT,
            sample_count: supported_sample_count(&adapter, surface_format, DepthTexture::FORMAT),
        };

        let bindless = BindlessTextures::new(&device, &queue, bindless_capacity);
        let passes = RenderPasses::new(&device, targets, &bindless);
        let depth_texture = DepthTexture::new(&device, [1, 1], targets.sample_count);

        Ok(Self {
            target: Target::Surface(surface),
            format: surface_format,
            targets,
            depth_texture,
            // Sized on the first `resize_surface`, like the depth buffer.
            msaa_texture: targets
                .is_multisampled()
                .then(|| MsaaTexture::new(&device, surface_format, [1, 1], targets.sample_count)),
            bindless,
            toggles: RenderToggles::default(),
            size: [1, 1],
            device,
            queue,
            passes,
        })
    }

    /// A renderer with no window, drawing into an owned texture that can be read back as
    /// RGBA8. Used by the comparison harness; see AGENTS.md §6.
    ///
    /// The colour format is `Rgba8Unorm` — deliberately not sRGB, matching the windowed
    /// path so captures and the live view agree (AGENTS.md §3.5).
    pub async fn new_headless(size: [u32; 2]) -> Result<Renderer<'static>, NewRendererError> {
        use NewRendererError as E;

        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .map_err(E::RequestAdapter)?;
        let (device, queue, bindless_capacity) = request_device(&adapter).await?;

        let format = TextureFormat::Rgba8Unorm;
        let targets = TargetFormats {
            colour: format,
            depth: DepthTexture::FORMAT,
            sample_count: supported_sample_count(&adapter, format, DepthTexture::FORMAT),
        };
        let bindless = BindlessTextures::new(&device, &queue, bindless_capacity);
        let passes = RenderPasses::new(&device, targets, &bindless);
        let depth_texture = DepthTexture::new(&device, size, targets.sample_count);
        let target = Self::make_offscreen(&device, format, size);
        // The headless path never calls `resize_surface`, so this is its only chance to tell
        // the screen-space passes how big the target is.
        passes.text.set_viewport(&queue, size);

        Ok(Renderer {
            target,
            format,
            targets,
            depth_texture,
            msaa_texture: targets
                .is_multisampled()
                .then(|| MsaaTexture::new(&device, format, size, targets.sample_count)),
            bindless,
            toggles: RenderToggles::default(),
            size,
            device,
            queue,
            passes,
        })
    }

    fn make_offscreen(device: &Device, format: TextureFormat, size: [u32; 2]) -> Target<'static> {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("offscreen_colour"),
            size: Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Texture-to-buffer copies need rows aligned to COPY_BYTES_PER_ROW_ALIGNMENT.
        let padded_bytes_per_row = (size[0] * 4).div_ceil(256) * 256;
        let readback = device.create_buffer(&BufferDescriptor {
            label: Some("offscreen_readback"),
            size: (padded_bytes_per_row * size[1]) as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Target::Offscreen {
            texture,
            readback,
            size,
            padded_bytes_per_row,
        }
    }

    /// `(textures registered, capacity)` in the bindless array (AGENTS.md §12).
    ///
    /// The measurement §12.3 predicted and §13.2 budgets against: LookoutPoint's region — the
    /// worst case in the shipped data — registers 227 slots at scene load and 229 once the
    /// sky's keyframe pair lands, against a capacity of 4096.
    pub fn bindless_stats(&self) -> (u32, u32) {
        self.bindless.stats()
    }

    pub fn resize_surface(&mut self, size: [u32; 2]) {
        self.size = size;
        self.passes.text.set_viewport(&self.queue, size);

        // The multisampled colour attachment must track the target's size and the depth
        // buffer's sample count, so all three are rebuilt together.
        self.msaa_texture = self.targets.is_multisampled().then(|| {
            MsaaTexture::new(
                &self.device,
                self.format,
                size,
                self.targets.sample_count,
            )
        });

        let Target::Surface(surface) = &self.target else {
            self.target = Self::make_offscreen(&self.device, self.format, size);
            self.depth_texture =
                DepthTexture::new(&self.device, size, self.targets.sample_count);
            return;
        };
        surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                view_formats: vec![],
                alpha_mode: CompositeAlphaMode::Auto,
                width: size[0],
                height: size[1],
                desired_maximum_frame_latency: 2,
                present_mode: PresentMode::AutoVsync,
            },
        );

        self.depth_texture = DepthTexture::new(&self.device, size, self.targets.sample_count);
    }

    pub fn set_terrain(&mut self, terrain: &TerrainData) {
        self.passes
            .terrain
            .set_terrain(&self.device, &self.queue, &mut self.bindless, terrain);
    }

    /// Drop everything the current scene uploaded — models, local detail, and the textures
    /// all of them plus the terrain were drawing with.
    ///
    /// **Call this before `set_terrain`, not after.** The bindless registry is scene-scoped,
    /// not model-scoped: clearing it invalidates every `BindlessIndex` handed out, terrain's
    /// included, and terrain registers its ground textures and blend tables the moment
    /// `set_terrain` runs (AGENTS.md §12.4). Getting that order wrong leaves the landscape
    /// sampling whatever took its slots next — which looks almost right, so `TerrainPass`
    /// carries the registry's generation and asserts on it rather than trusting this comment.
    pub fn clear_scene(&mut self) {
        self.passes.model.clear_models();
        self.passes.local_detail.clear();
        self.passes.sky.clear();
        self.bindless.clear();
    }

    /// Whether `id` is already resident, so the caller can skip reading and decoding it.
    ///
    /// This is where the dedup pays: the registry would deduplicate the *upload* anyway, but
    /// asking first lets `scene` skip the archive read and the BC slice too, which is the
    /// larger half of the work (AGENTS.md §12.6).
    pub fn has_texture(&self, asset_id: u32) -> bool {
        self.bindless.index_of(TextureKey::Asset(asset_id)).is_some()
    }

    /// Upload one mesh asset and every repeated-mesh local detail placement of it.
    ///
    /// `alpha_cutoff` is the object type's `AlphaRef / 255` — a property of the object type
    /// rather than of the mesh's materials, so one mesh may appear in more than one batch.
    pub fn add_local_detail(
        &mut self,
        model: &Model,
        instances: &[LocalDetailInstance],
        alpha_cutoff: f32,
    ) -> Result<(), AddLocalDetailError> {
        self.passes.local_detail.add_batch(
            &self.device,
            &self.queue,
            &mut self.bindless,
            model,
            instances,
            alpha_cutoff,
        )
    }

    /// `(mesh assets uploaded, instances drawn)` for the repeated-mesh pass.
    pub fn local_detail_stats(&self) -> (usize, usize) {
        self.passes.local_detail.stats()
    }

    /// Upload one mesh asset and every placement of it. Geometry, materials and textures
    /// are uploaded once regardless of how many instances there are.
    pub fn add_model(
        &mut self,
        model: &Model,
        instances: &[ModelInstance],
        kind: ModelKind,
    ) -> Result<(), AddModelError> {
        self.passes.model.add_model(
            &self.device,
            &self.queue,
            &mut self.bindless,
            model,
            instances,
            kind,
        )
    }

    /// `(mesh assets uploaded, placements drawn)`.
    pub fn model_stats(&self) -> (usize, usize) {
        (
            self.passes.model.model_count(),
            self.passes.model.instance_count(),
        )
    }

    /// `(mesh assets uploaded, placements drawn)` for one kind.
    pub fn model_stats_of(&self, kind: ModelKind) -> (usize, usize) {
        (
            self.passes.model.model_count_of(kind),
            self.passes.model.instance_count_of(kind),
        )
    }

    /// The target's size in physical pixels — the space [`GlyphInstance::rect`] is measured in.
    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    /// The slot holding a 1×1 opaque white texture, so a [`GlyphInstance`] can be a **solid
    /// rectangle** — the console's panel, and any other screen-space fill.
    ///
    /// It is the bindless array's fallback (`bindless.rs`), which is the whole reason this
    /// needs no second pipeline, no second shader and no upload: the slot is created with the
    /// device and `clear_scene` never touches it, so unlike a glyph it is valid in every
    /// generation. Sampling white gives coverage 1 everywhere, and `text.wgsl` multiplies the
    /// instance colour's alpha by coverage — so the instance's colour *is* the fill.
    pub fn solid_index(&self) -> u32 {
        self.bindless.fallback_index()
    }

    /// Which world subsystems currently draw (AGENTS.md §13.5).
    pub fn toggles(&self) -> RenderToggles {
        self.toggles
    }

    /// Choose which world subsystems draw. Text is not among them: the console must be able to
    /// turn the world off and still say so.
    pub fn set_toggles(&mut self, toggles: RenderToggles) {
        self.toggles = toggles;
    }

    /// The bindless slot holding the glyph registered under `key`, if it is still resident.
    ///
    /// **Ask every frame rather than caching the answer.** The registry is scene-scoped and
    /// `clear_scene` takes glyph slots with everything else, so an index kept across a level
    /// load is stale (AGENTS.md §13.2a). A `None` here means "rasterize and re-add", which is
    /// cheap because the caller still holds the bitmap.
    pub fn glyph_index(&self, key: u64) -> Option<u32> {
        self.bindless.index_of(TextureKey::Glyph(key))
    }

    /// Upload one rasterized glyph and give it a slot, or return the slot it already has.
    ///
    /// `key` is opaque — the renderer compares it and nothing else. Batch every glyph a frame
    /// newly needs before drawing rather than adding them one at a time: each registration
    /// dirties the whole array, and `encode` rebuilds it once (§13.3).
    pub fn add_glyph(
        &mut self,
        key: u64,
        image: &TextureImage,
    ) -> Result<u32, AddGlyphError> {
        self.passes.text.add_glyph(
            &self.device,
            &self.queue,
            &mut self.bindless,
            key,
            image,
        )
    }

    /// Replace the text drawn over the frame. An empty slice draws nothing.
    ///
    /// Indices in `instances` must come from [`Self::add_glyph`] or [`Self::glyph_index`] in
    /// the *current* generation; the pass records which one and asserts on it at draw time.
    pub fn set_text(&mut self, instances: &[GlyphInstance]) {
        self.passes
            .text
            .set_text(&self.device, &self.bindless, instances);
    }

    /// Glyphs queued to draw this frame.
    pub fn text_stats(&self) -> u32 {
        self.passes.text.glyph_count()
    }

    pub fn update_terrain_uniforms(&self, view_proj: [[f32; 4]; 4], camera_pos: glam::Vec3) {
        self.passes
            .terrain
            .update_uniforms(&self.queue, view_proj, camera_pos.to_array());
    }

    pub fn update_model_uniforms(&self, view_proj: [[f32; 4]; 4]) {
        self.passes.model.update_uniforms(&self.queue, view_proj);
        self.passes
            .local_detail
            .update_uniforms(&self.queue, view_proj);
    }

    pub fn set_model_camera_pos(&mut self, pos: glam::Vec3) {
        self.passes.model.set_camera_pos(pos);
    }

    /// Set one of the sky's two texture stages. `secondary` picks `t1`, the one the pixel
    /// shader blends toward; `asset_id` is the registry key (AGENTS.md §12.6).
    pub fn set_sky_texture(
        &mut self,
        secondary: bool,
        asset_id: u32,
        image: &TextureImage,
    ) {
        self.passes.sky.set_texture(
            &self.device,
            &self.queue,
            &mut self.bindless,
            secondary,
            asset_id,
            image,
        );
    }

    /// `gradient_top`/`gradient_bottom` are the sky shader's `c92`/`c93`, `texture_blend`
    /// its `c0.w`. The environment layer (AGENTS.md step 2) will supply them; until then
    /// the caller passes zeros and the sky shows its raw texture.
    pub fn update_sky_uniforms(
        &self,
        view_proj: [[f32; 4]; 4],
        gradient_top: [f32; 4],
        gradient_bottom: [f32; 4],
        texture_blend: f32,
    ) {
        self.passes.sky.update_uniforms(
            &self.queue,
            view_proj,
            gradient_top,
            gradient_bottom,
            texture_blend,
        );
    }

    /// Record every pass, ending with `view` holding the finished frame. Shared by the
    /// windowed and offscreen paths so a capture is the same frame the viewer would show.
    ///
    /// Under MSAA the passes draw into the multisampled texture and [`ResolvePass`] resolves
    /// it into `view`; without it they draw into `view` directly.
    fn encode(&mut self, view: &TextureView) -> CommandEncoder {
        // One rebuild per frame, at a controlled point, rather than one per registered
        // texture: a `BindGroup` is immutable, so every new texture means rebuilding the
        // whole array (AGENTS.md §12.2).
        self.bindless.rebuild_if_dirty(&self.device);
        let bindless = self.bindless.frame();

        let mut cmd = self.device.create_command_encoder(&Default::default());

        let colour = match &self.msaa_texture {
            Some(msaa) => &msaa.view,
            None => view,
        };

        // `clear` and `resolve` are not toggleable, and that is deliberate: a pass that draws
        // nothing cannot be switched off by accident, so the depth buffer is always cleared
        // however many subsystems the console has turned off (§13.5).
        self.passes
            .clear
            .pass(&mut cmd, colour, self.depth_texture.view());
        if self.toggles.sky {
            self.passes.sky.pass(&mut cmd, bindless, colour);
        }
        if self.toggles.landscape {
            self.passes
                .terrain
                .pass(&mut cmd, bindless, colour, self.depth_texture.view());
        }
        // Local detail before the model pass: both write depth for their opaque draws, and
        // the model pass ends with its depth-sorted blended ones, which must come last.
        if self.toggles.repeated_meshes {
            self.passes.local_detail.pass(
                &mut cmd,
                bindless.bind_group,
                colour,
                self.depth_texture.view(),
            );
        }
        // Both mesh kinds go through one pass: they share the pipeline, and their blended
        // instances share one depth sort. The toggles are therefore a filter inside the pass,
        // not a condition around it.
        self.passes.model.pass(
            &mut cmd,
            bindless.bind_group,
            colour,
            self.depth_texture.view(),
            ModelKinds {
                static_meshes: self.toggles.static_meshes,
                animated_meshes: self.toggles.animated_meshes,
            },
        );

        if let Some(msaa) = &self.msaa_texture {
            self.passes.resolve.pass(&mut cmd, &msaa.view, view);
        }

        // Text last, into `view` rather than `colour`: it draws over the *resolved* frame, so
        // it is the one pass that is not multisampled (AGENTS.md §13.6). With MSAA off the two
        // are the same texture and this is simply the last pass.
        self.passes.text.pass(&mut cmd, bindless, view);

        cmd
    }

    /// Draw and present. Windowed targets only.
    pub fn render(&mut self) -> Result<PrePresent, SurfaceError> {
        let Target::Surface(surface) = &self.target else {
            return Err(SurfaceError::Lost);
        };
        let surface_texture = surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let cmd = self.encode(&view);
        self.queue.submit([cmd.finish()]);

        Ok(PrePresent(surface_texture))
    }

    /// Draw offscreen and read the result back as tightly packed RGBA8, row-major from the
    /// top left. Headless targets only.
    pub fn render_to_image(&mut self) -> Result<Image, CaptureError> {
        // Handles are Arc-based, so cloning releases the borrow on `self.target` and lets
        // `encode` take `&mut self`.
        let Target::Offscreen {
            texture,
            readback,
            size,
            padded_bytes_per_row,
        } = &self.target
        else {
            return Err(CaptureError::NotHeadless);
        };
        let (texture, readback, size, padded_bytes_per_row) = (
            texture.clone(),
            readback.clone(),
            *size,
            *padded_bytes_per_row,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let copy_src = texture.as_image_copy();
        let copy_dst = TexelCopyBufferInfo {
            buffer: &readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size[1]),
            },
        };

        let mut cmd = self.encode(&view);
        cmd.copy_texture_to_buffer(
            copy_src,
            copy_dst,
            Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([cmd.finish()]);

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|_| CaptureError::Poll)?;
        rx.recv()
            .map_err(|_| CaptureError::Poll)?
            .map_err(|_| CaptureError::Map)?;

        // Drop the padding wgpu required on each row.
        let mapped = slice.get_mapped_range();
        let row_bytes = (size[0] * 4) as usize;
        let mut pixels = Vec::with_capacity(row_bytes * size[1] as usize);
        for row in 0..size[1] as usize {
            let start = row * padded_bytes_per_row as usize;
            pixels.extend_from_slice(&mapped[start..start + row_bytes]);
        }
        drop(mapped);
        readback.unmap();

        Ok(Image {
            width: size[0],
            height: size[1],
            rgba: pixels,
        })
    }
}

/// A captured frame: tightly packed RGBA8, row-major from the top left.
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Error, Display, Debug)]
pub enum CaptureError {
    #[display("renderer was not created headless")]
    NotHeadless,
    #[display("device poll failed")]
    Poll,
    #[display("buffer map failed")]
    Map,
}

pub struct PrePresent(SurfaceTexture);

impl PrePresent {
    pub fn present(self) {
        self.0.present();
    }
}

#[derive(Error, Display, Debug)]
pub enum NewRendererError {
    RequestAdapter(RequestAdapterError),
    RequestDevice(RequestDeviceError),
    CreateSurface(CreateSurfaceError),
    #[display(
        "this GPU is missing {_0:?}. OpenAlbion's renderer is bindless and has no fallback \
         path; it needs a DX12, Vulkan or Metal 2.0+ adapter (AGENTS.md §12.2)"
    )]
    MissingFeatures(#[error(not(source))] Features),
    #[display(
        "this GPU allows only {_0} textures in a binding array, and the renderer needs at \
         least {}. That is Metal argument-buffers Tier 1 territory (AGENTS.md §12.3)",
        bindless::MIN_BINDLESS_TEXTURES
    )]
    BindingArrayTooSmall(#[error(not(source))] u32),
    #[display("this GPU allows only {_0} bytes of immediate data; the renderer needs {MAX_IMMEDIATE_SIZE}")]
    ImmediatesTooSmall(#[error(not(source))] u32),
}

/// In frame order. `clear` and `resolve` draw nothing; the four in between draw the world into
/// the multisampled texture, and `text` draws over the resolved result.
struct RenderPasses {
    clear: ClearPass,
    sky: OuterSkyPass,
    terrain: TerrainPass,
    model: ModelPass,
    local_detail: LocalDetailPass,
    resolve: ResolvePass,
    text: TextPass,
}

impl RenderPasses {
    fn new(device: &Device, targets: TargetFormats, bindless: &BindlessTextures) -> Self {
        Self {
            clear: ClearPass,
            sky: OuterSkyPass::new(device, targets, bindless),
            terrain: TerrainPass::new(device, targets, bindless),
            model: ModelPass::new(device, targets, bindless),
            local_detail: LocalDetailPass::new(device, targets, bindless),
            resolve: ResolvePass,
            text: TextPass::new(device, targets, bindless),
        }
    }
}

/// Clears the frame: colour to black, depth to the reverse-Z "nothing here yet" value.
///
/// **The depth clear belongs here and not in a drawing pass.** It used to be `TerrainPass`'s,
/// which made that pass silently load-bearing in two ways: it returns early when it has no
/// buffers, and it is the first pass a `EnableLandscape`-style toggle (AGENTS.md §13.5) would
/// switch off — either way the passes after it would depth-test against the *previous* frame.
/// A pass that draws nothing cannot be turned off by accident.
struct ClearPass;

impl ClearPass {
    fn pass(
        &mut self,
        cmd: &mut CommandEncoder,
        target_texture_view: &TextureView,
        depth_texture_view: &TextureView,
    ) {
        cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture_view,
                // Reverse-Z: `0.0` is "nothing here yet", not `1.0`
                // (`Camera::projection_matrix`).
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}

/// Resolves the multisampled colour texture down into the presentable one.
///
/// A pass of its own rather than a `resolve_target` on the last drawing pass: resolving in
/// every pass would resolve three times for nothing, and resolving in *one* of them would
/// make that pass silently load-bearing — reorder the passes and the frame goes blank. This
/// draws nothing; the resolve happens because the attachment has a `resolve_target`, and
/// `StoreOp::Discard` then throws the multisampled contents away.
struct ResolvePass;

impl ResolvePass {
    fn pass(&mut self, cmd: &mut CommandEncoder, multisampled: &TextureView, target: &TextureView) {
        cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: multisampled,
                depth_slice: None,
                resolve_target: Some(target),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}
