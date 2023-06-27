use crate::Renderer;
use std::{any::type_name, mem::swap};

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
        let frame = self.surface.get_current_texture()?;

        self.view = frame.texture.create_view(&Default::default());
        self.update();
        self.color_pass();

        let mut main_cmds = MainCommands::new(&self.device);

        swap(&mut main_cmds, &mut self.main_cmds);

        self.queue.submit([main_cmds.into_inner().finish()]);

        frame.present();

        Ok(())
    }

    fn update(&mut self) {}

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
