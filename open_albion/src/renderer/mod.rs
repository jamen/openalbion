mod base;
mod scene;

pub use base::*;
pub use scene::*;

use crate::State;

use std::array::IntoIter;

use winit::window::Window;

pub struct Renderer {
    pub base: RendererBase,
    pub scene_renderer: SceneRenderer,
}

impl Renderer {
    pub async fn create(window: &Window, state: &State) -> Self {
        let base = RendererBase::create(window).await;
        let scene_renderer = SceneRenderer::create(&base, &state);

        Self {
            base,
            scene_renderer,
        }
    }

    // TODO: Handle other events like scale factor change too.
    /// Resizes the swap chain.  This doesn't resize the render systems, which handle it on render instead. Maybe add to RenderSystem to handle these events
    pub fn resize(&mut self, width: u32, height: u32) {
        self.base.resize(width, height);
        self.scene_renderer.resize(&self.base);
    }

    pub fn render(&mut self, state: &State) {
        let frame = match self.base.surface.get_current_frame() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Dropped frame. {}", e);
                return
            }
        };

        let view = frame.output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let command_bufs = [
            self.scene_renderer.render(&self.base, &view, &state),
        ];

        self.base.queue.submit(IntoIter::new(command_bufs));
    }
}

