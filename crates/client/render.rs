use futures::executor::block_on;
use renderer::Renderer;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use winit::window::Window;

pub struct RenderSystemParams {
    pub window_ref: Arc<Window>,
}

pub fn spawn(params: RenderSystemParams) -> JoinHandle<()> {
    thread::spawn(move || RenderSystem::new(params).run())
}

struct RenderSystem {
    window_ref: Arc<Window>,
    renderer: Renderer,
}

impl RenderSystem {
    fn new(params: RenderSystemParams) -> Self {
        let window_ref = params.window_ref;

        let size: [u32; 2] = window_ref.inner_size().into();

        let renderer = block_on(Renderer::new(&*window_ref, size)).unwrap();

        Self {
            window_ref,
            renderer,
        }
    }
    fn run(mut self) -> ! {
        self.render();

        self.window_ref.set_visible(true);

        loop {
            self.render();
        }
    }
}

impl RenderSystem {
    fn render(&mut self) {
        if let Err(error) = self.renderer.render() {
            log::error!("{error:?}");
        }
    }
}
