mod action;
mod event;

use self::action::{Action, KeyState};
use crate::window::{ControlFlowRef, WindowRef};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};
use winit::event::{Event, ModifiersState};

pub struct GameSystemParams {
    pub window_ref: WindowRef,
    pub event_receiver: Receiver<Event<'static, ()>>,
    pub control_flow: ControlFlowRef,
    pub game_path: PathBuf,
}

pub struct GameHandle(JoinHandle<()>);

impl AsRef<JoinHandle<()>> for GameHandle {
    fn as_ref(&self) -> &JoinHandle<()> {
        &self.0
    }
}

pub struct GameSystem {
    event_receiver: Receiver<Event<'static, ()>>,
    control_flow: ControlFlowRef,
    modifiers: ModifiersState,
    key_bindings: HashMap<KeyState, Action>,
    // big_reader: BigReader<BufReader<File>>,
}

impl GameSystem {
    pub fn spawn(params: GameSystemParams) -> GameHandle {
        GameHandle(thread::spawn(move || GameSystem::new(params).run()))
    }

    fn new(params: GameSystemParams) -> Self {
        println!("{:?}", params.game_path);

        let graphics_path = params.game_path.join("data/graphics/graphics.big");

        println!("{:?}", graphics_path);

        let _graphics_file = BufReader::new(File::open(graphics_path).unwrap());

        Self {
            event_receiver: params.event_receiver,
            control_flow: params.control_flow,
            key_bindings: HashMap::default(),
            modifiers: ModifiersState::empty(),
            // big_reader: BigReader::new(graphics_file),
        }
    }

    fn run(mut self) -> ! {
        self.load_default_key_bindings();

        // Start update loop
        loop {
            self.update()
        }
    }

    fn update(&mut self) {
        // Process window events
        loop {
            match self.event_receiver.try_recv() {
                Err(_) => break,
                Ok(event) => self.on_event(event),
            }
        }
    }
}
