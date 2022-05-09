use raw_window_handle::HasRawWindowHandle;

#[derive(Debug)]
pub enum BaseError {
    NoAdapter,
    NoDevice(wgpu::RequestDeviceError),
    NoPreferredFormat,
}

/// Holds the device, queue, and surface used to initialize the rest of the renderer.
pub(crate) struct Base {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
}

impl Base {
    pub(crate) async fn new<W: HasRawWindowHandle>(
        window: &W,
        size: [u32; 2]
    ) -> Result<Self, BaseError> {
        let backend = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            backend,
            Some(&surface)
        )
            .await
            .ok_or(BaseError::NoAdapter)?;

        let format = surface.get_preferred_format(&adapter).ok_or(BaseError::NoPreferredFormat)?;

        let features = wgpu::Features::SPIRV_SHADER_PASSTHROUGH;

        let limits = wgpu::Limits::downlevel_webgl2_defaults();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor { label: None, features, limits },
            None,
        )
            .await
            .map_err(BaseError::NoDevice)?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0],
            height: size[1],
            present_mode: wgpu::PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        Ok(Base {
            device,
            queue,
            surface,
            surface_config
        })
    }
}

pub(crate) trait RenderPass {
    fn render_pass(
        &mut self,
        base: &Base,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        frame: &wgpu::SurfaceTexture,
    );
}