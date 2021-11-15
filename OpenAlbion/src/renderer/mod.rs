mod assets;

use std::sync::Arc;

use rend3::{ManagerReadyOutput, RenderRoutine, Renderer};
use wgpu::TextureView;

use crate::state::State;

pub struct MainRenderRoutine {}

impl MainRenderRoutine {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderRoutine<(), &'_ TextureView> for MainRenderRoutine {
    fn render(
        &mut self,
        renderer: Arc<Renderer>,
        cmd_bufs: flume::Sender<wgpu::CommandBuffer>,
        ready: ManagerReadyOutput,
        input: (),
        output: &'_ TextureView,
    ) {
    }
}
