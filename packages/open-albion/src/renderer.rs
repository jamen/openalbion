mod depth;
mod model;
mod sky;
pub mod terrain;
mod texture;

use self::depth::DepthTexture;
use self::model::ModelPass;
use self::sky::OuterSkyPass;
use self::terrain::TerrainPass;
pub use self::model::ModelTextureError;
pub use self::sky::LightingColoursError;
pub use self::texture::TextureUploadError;
use derive_more::{Display, Error};
use fable_data::big::AssetMetadata;
use fable_data::lev::Lev;
use wgpu::{
    CommandEncoder, CompositeAlphaMode, CreateSurfaceError, Device, DeviceDescriptor, Features,
    Instance, InstanceDescriptor, PresentMode, Queue, RequestAdapterError, RequestAdapterOptions,
    RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError, SurfaceTarget, SurfaceTexture,
    TextureFormat, TextureUsages, TextureView,
};

pub struct Renderer<'target> {
    device: Device,
    queue: Queue,
    surface: Surface<'target>,
    surface_format: TextureFormat,
    depth_texture: DepthTexture,
    passes: RenderPasses,
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

        let &surface_format = surface_capabilities
            .formats
            .first()
            .unwrap_or(&TextureFormat::Rgba8UnormSrgb);

        let passes = RenderPasses::new(&device, &queue, surface_format, DepthTexture::FORMAT);
        let depth_texture = DepthTexture::new(&device, [1, 1]);

        Ok(Self {
            surface,
            surface_format,
            depth_texture,
            device,
            queue,
            passes,
        })
    }

    pub fn resize_surface(&mut self, size: [u32; 2]) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                view_formats: vec![self.surface_format.add_srgb_suffix()],
                alpha_mode: CompositeAlphaMode::Auto,
                width: size[0],
                height: size[1],
                desired_maximum_frame_latency: 2,
                present_mode: PresentMode::AutoVsync,
            },
        );

        self.depth_texture = DepthTexture::new(&self.device, size);
    }

    pub fn set_terrain(&mut self, lev: &Lev) {
        self.passes.terrain.set_terrain(&self.device, lev);
    }

    pub fn set_model(
        &mut self,
        mesh: &fable_data::mesh::Mesh,
        material_textures: &[Option<(AssetMetadata, Vec<u8>)>],
    ) -> Result<(), ModelTextureError> {
        self.passes
            .model
            .set_model(&self.device, &self.queue, mesh, material_textures)
    }

    pub fn update_terrain_uniforms(&self, view_proj: [[f32; 4]; 4]) {
        self.passes.terrain.update_uniforms(&self.queue, view_proj);
    }

    pub fn update_model_uniforms(&self, view_proj: [[f32; 4]; 4]) {
        self.passes.model.update_uniforms(&self.queue, view_proj);
    }

    pub fn set_sky_texture0(
        &mut self,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), TextureUploadError> {
        self.passes
            .sky
            .set_texture0(&self.device, &self.queue, asset_info, asset_data)
    }

    pub fn set_sky_texture1(
        &mut self,
        asset_info: &AssetMetadata,
        asset_data: &[u8],
    ) -> Result<(), TextureUploadError> {
        self.passes
            .sky
            .set_texture1(&self.device, &self.queue, asset_info, asset_data)
    }

    pub fn set_lighting_lut(&mut self, tga_bytes: &[u8]) -> Result<(), LightingColoursError> {
        self.passes
            .sky
            .set_lighting_lut(&self.device, &self.queue, tga_bytes)
    }

    pub fn update_sky_uniforms(&self, view_proj: [[f32; 4]; 4], time_of_day: f32, sky_blend: f32) {
        self.passes
            .sky
            .update_uniforms(&self.queue, view_proj, time_of_day, sky_blend);
    }

    pub fn render(&mut self) -> Result<PrePresent, SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;

        let surface_texture_view =
            surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    format: Some(self.surface_format.add_srgb_suffix()),
                    ..Default::default()
                });

        let mut cmd = self.device.create_command_encoder(&Default::default());

        self.passes.clear.pass(&mut cmd, &surface_texture_view);
        self.passes.sky.pass(&mut cmd, &surface_texture_view);
        self.passes
            .terrain
            .pass(&mut cmd, &surface_texture_view, self.depth_texture.view());
        self.passes
            .model
            .pass(&mut cmd, &surface_texture_view, self.depth_texture.view());

        self.queue.submit([cmd.finish()]);

        Ok(PrePresent(surface_texture))
    }
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
