use winit::event::Event;

use super::GameSystem;

impl GameSystem {
    pub fn on_event(&mut self, event: Event<'static, ()>) {}
}
