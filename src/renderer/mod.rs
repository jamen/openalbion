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
        let (swap_chain_descriptor, swap_chain) =
            RendererBase::create_swap_chain(&self.base.surface, &self.base.device, width, height);
        self.base.swap_chain_descriptor = swap_chain_descriptor;
        self.base.swap_chain = swap_chain;
    }

    pub fn render(&mut self, state: &State) {
        let frame = match self.base.swap_chain.get_current_frame() {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Dropped frame. {}", e);
                return
            }
        };

        let command_bufs = [
            self.scene_renderer.render(&self.base, &frame, &state),
        ];

        self.base.queue.submit(IntoIter::new(command_bufs));
    }
}

