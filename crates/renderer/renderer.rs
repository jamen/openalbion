use std::{mem::swap, slice, any::type_name};

use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use derive_more::{From, Display};

use crate::texture::Textures;

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

#[derive(Display, From, Debug)]
pub enum RendererError {
    #[display(fmt = "no adapter")]
    NoAdapter,

    #[display(fmt = "no device. {}", _0)]
    NoDevice(wgpu::RequestDeviceError),

    #[display(fmt = "no preferred format")]
    NoPreferredFormat,

    #[display(fmt = "create surface error. {}", _0)]
    NoSurface(wgpu::CreateSurfaceError),

    #[display(fmt = "no surface view. {}", _0)]
    NoSurfaceView(wgpu::SurfaceError),
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
        // let buffers = Buffers::new(&device, 2048, 2048, 2048);

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

pub struct MainCommands(wgpu::CommandEncoder);

impl MainCommands {
    pub fn new(device: &wgpu::Device) -> Self {
        Self(
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(type_name::<MainCommands>()),
            }),
        )
    }

    fn into_inner(self) -> wgpu::CommandEncoder {
        self.0
    }
}

// impl AsRef<wgpu::CommandEncoder> for MainEncoder {
//     fn as_ref(&self) -> &wgpu::CommandEncoder {
//         &self.0
//     }
// }

impl AsMut<wgpu::CommandEncoder> for MainCommands {
    fn as_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.0
    }
}

impl Renderer {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // TODO: Write this on resize, instead of every frame
        self.set_surface_size(&[
            self.surface_config.width as f32,
            self.surface_config.height as f32,
        ]);

        // Next frame
        let frame = self.surface.get_current_texture()?;

        self.view = frame.texture.create_view(&Default::default());

        // Passes
        self.color_pass();

        // Next command buffer
        let mut main_cmds = MainCommands::new(&self.device);

        swap(&mut main_cmds, &mut self.main_cmds);

        // Present frame
        self.queue.submit([main_cmds.into_inner().finish()]);

        frame.present();

        Ok(())
    }

    fn set_surface_size(&mut self, size: &[f32; 2]) {
        self.buffers.param.surface_size.
            inner()
            .write_buffer(&self.queue, 0, slice::from_ref(size));
    }

    fn color_pass(&mut self) {
        let cmds = self.main_cmds.as_mut();

        let mut rpass = cmds.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: false,
                },
            })],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(self.pipelines.color.as_ref());
        rpass.set_bind_group(0, self.bind_groups.color.as_ref(), &[]);

        // ...
    }
}
