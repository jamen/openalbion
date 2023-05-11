mod event;

use crate::window::SharedControlFlow;
use std::{
    sync::{mpsc::Receiver, Arc},
    thread::{self, JoinHandle},
};
use winit::{event::Event, window::Window};

pub struct GameSystemParams {
    pub window: Arc<Window>,
    pub event_receiver: Receiver<Event<'static, ()>>,
    pub control_flow: SharedControlFlow,
}

pub fn spawn(params: GameSystemParams) -> JoinHandle<()> {
    thread::spawn(move || GameSystem::new(params).run())
}

struct GameSystem {
    event_receiver: Receiver<Event<'static, ()>>,
}

impl GameSystem {
    fn new(params: GameSystemParams) -> Self {
        Self {
            event_receiver: params.event_receiver,
        }
    }

    fn run(mut self) -> ! {
        self.load_default_key_bindings();

        // Start update loop
        loop {
            // Process window events
            loop {
                match self.event_receiver.try_recv() {
                    Err(_) => break,
                    Ok(event) => self.on_event(event),
                }
            }
        }
    }
}

impl GameSystem {
    pub fn load_default_key_bindings(&mut self) {}
}
