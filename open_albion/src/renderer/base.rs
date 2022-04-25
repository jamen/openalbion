use raw_window_handle::HasRawWindowHandle;

use thiserror::Error;
use glam::UVec2;

pub struct Base {
    pub size: UVec2,
    pub surface: wgpu::Surface,
    pub preferred_format: wgpu::TextureFormat,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Base {
    pub async fn new(
        window: impl HasRawWindowHandle,
        size: UVec2,
    ) -> Result<Self, BaseError> {
        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

        let instance = wgpu::Instance::new(backend);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            backend,
            Some(&surface)
        )
        .await
        .ok_or(BaseError::NoAdapter)?;

        {
            let adapter_info = adapter.get_info();

            log::debug!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        }

        let preferred_format = surface.get_preferred_format(&adapter)
            .ok_or(BaseError::NoPreferredFormat)?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
            limits: wgpu::Limits::default(),
        }, None)
        .await?;

        log::debug!("Limits {:?}", device.limits());

        surface.configure(&device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: preferred_format,
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Mailbox,
        });

        Ok(Self {
            size,
            surface,
            preferred_format,
            device,
            queue,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BaseError {
    #[error("No adapter")]
    NoAdapter,

    #[error("No preferred format")]
    NoPreferredFormat,

    #[error("Request device error")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
}