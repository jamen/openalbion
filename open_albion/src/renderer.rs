use crate::{State, state::Gui};

use glam::UVec2;

use winit::window::Window;

use egui_wgpu_backend::ScreenDescriptor;

pub struct RendererCore {
    pub size: glam::UVec2,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub preferred_format: wgpu::TextureFormat,
}

impl RendererCore {
    pub async fn new(window: &Window, backends: wgpu::Backends) -> Self {
        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height).max(UVec2::new(1, 1));

        let instance = wgpu::Instance::new(backends);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
            .await
            .expect("Failed to get wgpu's adapter");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        }, None)
            .await
            .expect("Failed to get wgpu's device and queue");

        let preferred_format = surface.get_preferred_format(&adapter)
            .expect("Failed to get wgpu's preferred format");

        surface.configure(&device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: preferred_format,
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Mailbox,
        });

        let preferred_format = surface.get_preferred_format(&adapter)
            .expect("Failed to get wgpu's preferred format");

        Self {
            size,
            surface,
            adapter,
            device,
            queue,
            preferred_format,
        }
    }

    fn reconfigure_surface(&mut self, size: glam::UVec2) {
        self.size = size.max(UVec2::new(1, 1));

        self.surface.configure(&self.device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.preferred_format,
            width: self.size.x,
            height: self.size.y,
            present_mode: wgpu::PresentMode::Mailbox,
        });
    }
}

pub struct Renderer {
    core: RendererCore,
    scale_factor: f32,
    depth_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: &Window, backends: wgpu::Backends) -> Self {
        let core = RendererCore::new(window, backends).await;

        let scale_factor = window.scale_factor() as f32;

        let depth_view = Self::create_depth_view(&core);

        Self {
            core,
            scale_factor,
            depth_view,
        }
    }
    pub fn create_depth_view(core: &RendererCore) -> wgpu::TextureView {
        let depth_texture = core.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: core.size.x,
                height: core.size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
    #[inline]
    pub fn size(&self) -> &UVec2 {
        &self.core.size
    }
    #[inline]
    pub fn device(&self) -> &wgpu::Device {
        &self.core.device
    }
    #[inline]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.core.queue
    }
    #[inline]
    pub fn surface(&self) -> &wgpu::Surface {
        &self.core.surface
    }
    // #[inline]
    // pub fn adapter(&self) -> &wgpu::Adapter {
    //     &self.core.adapter
    // }
    pub fn scale_factor_mut(&mut self) -> &mut f32 {
        &mut self.scale_factor
    }
    pub fn reconfigure_surface(&mut self, size: glam::UVec2) {
        self.core.reconfigure_surface(size);
        self.depth_view = Self::create_depth_view(&self.core)
    }
}

impl Renderer {
    pub fn render(&mut self, state: &mut State) {
        let frame = match self.surface().get_current_texture() {
            Ok(x) => x,
            Err(e) => {
                log::error!("Dropped frame: {:?}", e);
                self.reconfigure_surface(self.core.size);
                return
            }
        };

        let surface_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cmd_bufs = [
            self.render_pass_scene(&surface_view),
            self.render_pass_gui(&surface_view, &mut state.gui),
        ];

        self.queue().submit(cmd_bufs.into_iter());

        frame.present();
    }

    // TODO: Figure out a better structure for render passes?

    fn render_pass_gui(&mut self, surface_view: &wgpu::TextureView, gui: &mut Gui) -> wgpu::CommandBuffer {
        let mut egui_rpass = egui_wgpu_backend::RenderPass::new(self.device(), self.core.preferred_format, 1);

        let mut encoder = self.device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        gui.platform.begin_frame();
        gui.update();
        let (_output, paint_cmds) = gui.platform.end_frame(None); // TODO: egui_winit_platform's docs say "If the optional window is set, it will set the cursor key based on egui’s instructions."
        let paint_jobs = gui.platform.context().tessellate(paint_cmds);
        let screen_desc = ScreenDescriptor {
            physical_width: self.size().x,
            physical_height: self.size().y,
            scale_factor: self.scale_factor,
        };

        {
            egui_rpass.update_texture(self.device(), self.queue(), &*gui.platform.context().texture());
            egui_rpass.update_user_textures(self.device(), self.queue());
            egui_rpass.update_buffers(self.device(), self.queue(), &paint_jobs, &screen_desc);
            egui_rpass.execute(&mut encoder, &surface_view, &paint_jobs, &screen_desc, None);
        }

        encoder.finish()
    }

    fn render_pass_scene(&mut self, surface_view: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder = self.device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        encoder.finish()
    }
}