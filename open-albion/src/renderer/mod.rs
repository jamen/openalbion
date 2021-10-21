use std::sync::Arc;

use rend3::util::output::OutputFrame;
use rend3::{RenderRoutine, Renderer};

use crate::state::State;

pub struct MainRenderRoutine {
    state: Arc<State>,
}

impl MainRenderRoutine {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

impl RenderRoutine for MainRenderRoutine {
    fn render(
        &mut self,
        renderer: Arc<Renderer>,
        cmd_bufs: &mut Vec<wgpu::CommandBuffer>,
        output: &OutputFrame,
    ) {
    }
}
