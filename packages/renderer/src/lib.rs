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

mod depth;
mod image;
mod model;
mod sky;
mod terrain;
mod texture;

use self::depth::DepthTexture;
use self::model::ModelPass;
use self::sky::OuterSkyPass;
use self::terrain::TerrainPass;
use derive_more::{Display, Error};
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoder, CompositeAlphaMode, CreateSurfaceError, Device,
    DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, MapMode, PresentMode, Queue,
    RequestAdapterError, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTarget, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

pub use self::image::{ImageFormat, TextureImage};
pub use self::model::{
    AddModelError, AlphaMode, Model, ModelMaterial, ModelPrimitive, ModelSubMesh, ModelVertex,
};
pub use self::terrain::{TerrainData, TerrainDraw, TerrainVertex};

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
    depth_texture: DepthTexture,
    passes: RenderPasses,
    target: Target<'target>,
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

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: Features::TEXTURE_COMPRESSION_BC,
                ..Default::default()
            })
            .await
            .map_err(E::RequestDevice)?;

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

        let passes = RenderPasses::new(&device, &queue, surface_format, DepthTexture::FORMAT);
        let depth_texture = DepthTexture::new(&device, [1, 1]);

        Ok(Self {
            target: Target::Surface(surface),
            format: surface_format,
            depth_texture,
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
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: Features::TEXTURE_COMPRESSION_BC,
                ..Default::default()
            })
            .await
            .map_err(E::RequestDevice)?;

        let format = TextureFormat::Rgba8Unorm;
        let passes = RenderPasses::new(&device, &queue, format, DepthTexture::FORMAT);
        let depth_texture = DepthTexture::new(&device, size);
        let target = Self::make_offscreen(&device, format, size);

        Ok(Renderer {
            target,
            format,
            depth_texture,
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

    pub fn resize_surface(&mut self, size: [u32; 2]) {
        let Target::Surface(surface) = &self.target else {
            self.target = Self::make_offscreen(&self.device, self.format, size);
            self.depth_texture = DepthTexture::new(&self.device, size);
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

        self.depth_texture = DepthTexture::new(&self.device, size);
    }

    pub fn set_terrain(&mut self, terrain: &TerrainData) {
        self.passes
            .terrain
            .set_terrain(&self.device, &self.queue, terrain);
    }

    pub fn clear_models(&mut self) {
        self.passes.model.clear_models();
    }

    pub fn add_model(&mut self, model: &Model) -> Result<(), AddModelError> {
        self.passes
            .model
            .add_model(&self.device, &self.queue, model)
    }

    pub fn update_terrain_uniforms(&self, view_proj: [[f32; 4]; 4], camera_pos: glam::Vec3) {
        self.passes
            .terrain
            .update_uniforms(&self.queue, view_proj, camera_pos.to_array());
    }

    pub fn update_model_uniforms(&self, view_proj: [[f32; 4]; 4]) {
        self.passes.model.update_uniforms(&self.queue, view_proj);
    }

    pub fn set_model_camera_pos(&mut self, pos: glam::Vec3) {
        self.passes.model.set_camera_pos(pos);
    }

    pub fn set_sky_texture0(&mut self, image: &TextureImage) {
        self.passes
            .sky
            .set_texture0(&self.device, &self.queue, image);
    }

    pub fn set_sky_texture1(&mut self, image: &TextureImage) {
        self.passes
            .sky
            .set_texture1(&self.device, &self.queue, image);
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

    /// Record every pass into `view`. Shared by the windowed and offscreen paths so a
    /// capture is the same frame the viewer would show.
    fn encode(&mut self, view: &TextureView) -> CommandEncoder {
        let mut cmd = self.device.create_command_encoder(&Default::default());
        self.passes.clear.pass(&mut cmd, view);
        self.passes.sky.pass(&mut cmd, view);
        self.passes
            .terrain
            .pass(&mut cmd, view, self.depth_texture.view());
        self.passes
            .model
            .pass(&mut cmd, view, self.depth_texture.view());
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
}

pub struct RenderPasses {
    clear: ClearPass,
    sky: OuterSkyPass,
    terrain: TerrainPass,
    model: ModelPass,
}

impl RenderPasses {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: TextureFormat,
        depth_format: TextureFormat,
    ) -> Self {
        Self {
            clear: ClearPass,
            sky: OuterSkyPass::new(device, surface_format),
            terrain: TerrainPass::new(device, surface_format, depth_format),
            model: ModelPass::new(device, queue, surface_format, depth_format),
        }
    }
}

struct ClearPass;

impl ClearPass {
    fn pass(&mut self, cmd: &mut CommandEncoder, target_texture_view: &TextureView) {
        cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target_texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
}
