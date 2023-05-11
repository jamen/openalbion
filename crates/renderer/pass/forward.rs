use crate::Renderer;

impl Renderer {
    pub(crate) fn forward_pass(&mut self, view: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: false,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // rpass.draw_indexed()
        }

        encoder.finish()
    }
}
