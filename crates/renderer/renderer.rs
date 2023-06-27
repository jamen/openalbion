mod pass;

use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use thiserror::Error;
use crate::{texture::Textures, buffer::Buffers, bind_group::BindGroups, pipeline::Pipelines, bind_group_layout::BindGroupLayouts, pipeline_layout::PipelineLayouts};

use self::pass::MainCommands;

pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface,
    pub(crate) surface_config: wgpu::SurfaceConfiguration,
    pub(crate) view: wgpu::TextureView,
    pub(crate) main_cmds: MainCommands,
    pub(crate) textures: Textures,
    pub(crate) buffers: Buffers,
    pub(crate) bind_groups: BindGroups,
    pub(crate) pipelines: Pipelines,
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("no adapter")]
    NoAdapter,

    #[error("no device. {0}")]
    NoDevice(#[from] wgpu::RequestDeviceError),

    #[error("no preferred format")]
    NoPreferredFormat,

    #[error("create surface error. {0}")]
    NoSurface(#[from] wgpu::CreateSurfaceError),

    #[error("no surface view. {0}")]
    NoSurfaceView(#[from] wgpu::SurfaceError),
}

impl Renderer {
    pub async fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(window: W, size: [u32; 2]) -> Result<Self, RendererError> {
                let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::all());
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }?;
        let adapter =
            wgpu::util::initialize_adapter_from_env_or_default(&instance, backends, Some(&surface))
                .await
                .ok_or(RendererError::NoAdapter)?;

        let capabilities = surface.get_capabilities(&adapter);
        let &format = capabilities.formats
            .first()
            .ok_or(RendererError::NoPreferredFormat)?;
        let features = wgpu::Features::empty();
        let limits = wgpu::Limits::downlevel_defaults();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features,
                    limits,
                },
                None,
            )
            .await?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0],
            height: size[1],
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let view = surface.get_current_texture()?.texture.create_view(&Default::default());
        let main_cmds = MainCommands::new(&device);
        let textures = Textures::new(&device, &surface_config);
        let buffers = Buffers::new(&device, 2048, 2048, 2048);

        let bg_layouts = BindGroupLayouts::new(&device);
        let bind_groups = BindGroups::new(&device, &bg_layouts);
        let pipeline_layouts = PipelineLayouts::new(&device, &bg_layouts);
        let pipelines = Pipelines::new(&device, &surface_config, &pipeline_layouts);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            view,
            main_cmds,
            textures,
            buffers,
            bind_groups,
            pipelines,
        })
    }
}