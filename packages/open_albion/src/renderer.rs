mod sky;

use self::sky::SkyPass;
use derive_more::{Display, Error};
use wgpu::{
    Backends, CommandEncoder, CompositeAlphaMode, CreateSurfaceError, Device, DeviceDescriptor,
    Instance, InstanceDescriptor, InstanceFlags, PresentMode, Queue, RequestAdapterError,
    RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration, SurfaceError,
    SurfaceTarget, SurfaceTexture, TextureFormat, TextureUsages, TextureView,
};

pub struct Renderer<'target> {
    device: Device,
    queue: Queue,
    surface: Surface<'target>,
    surface_format: TextureFormat,
    passes: RenderPasses,
}

impl<'target> Renderer<'target> {
    pub async fn new(target: impl Into<SurfaceTarget<'target>>) -> Result<Self, NewRendererError> {
        use NewRendererError as E;

        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .map_err(E::RequestAdapter)?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .map_err(E::RequestDevice)?;

        let surface = instance.create_surface(target).map_err(E::CreateSurface)?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let &surface_format = surface_capabilities
            .formats
            .get(0)
            .unwrap_or(&TextureFormat::Rgba8UnormSrgb);

        let passes = RenderPasses::new(&device, surface_format);

        Ok(Self {
            surface,
            surface_format,
            device,
            queue,
            passes,
        })
    }

    pub fn resize_surface(&self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                view_formats: vec![self.surface_format.add_srgb_suffix()],
                alpha_mode: CompositeAlphaMode::Auto,
                width,
                height,
                desired_maximum_frame_latency: 2,
                present_mode: PresentMode::AutoVsync,
            },
        );
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
    sky: SkyPass,
}

impl RenderPasses {
    pub fn new(device: &Device, surface_format: TextureFormat) -> Self {
        Self {
            clear: ClearPass,
            sky: SkyPass::new(device, surface_format),
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
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
