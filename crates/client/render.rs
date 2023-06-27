use crate::window::WindowRef;
use futures::executor::block_on;
use renderer::Renderer;
use std::thread::{self, JoinHandle};

pub struct RenderSystemParams {
    pub window_ref: WindowRef,
}

pub struct RenderHandle(JoinHandle<()>);

impl AsRef<JoinHandle<()>> for RenderHandle {
    fn as_ref(&self) -> &JoinHandle<()> {
        &self.0
    }
}

pub struct RenderSystem {
    window_ref: WindowRef,
    renderer: Renderer,
}

impl RenderSystem {
    pub fn spawn(params: RenderSystemParams) -> RenderHandle {
        RenderHandle(thread::spawn(move || RenderSystem::new(params).run()))
    }

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

    fn render(&mut self) {
        if let Err(error) = self.renderer.render() {
            log::error!("{error:?}");
        }
    }
}
