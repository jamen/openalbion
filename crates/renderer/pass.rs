mod forward;

use crate::Renderer;

use std::mem;

impl Renderer {
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut main_encoder = self.device.create_command_encoder(&Default::default());

        mem::swap(&mut main_encoder, &mut self.main_encoder);

        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let forward_pass = self.forward_pass(&view);

        self.queue.submit([main_encoder.finish(), forward_pass]);

        frame.present();

        Ok(())
    }
}
