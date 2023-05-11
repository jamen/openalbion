mod action;
mod event;

use crate::{reader::big::BigReader, window::SharedControlFlow};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{mpsc::Receiver, Arc},
    thread::{self, JoinHandle},
};
use winit::{
    event::{Event, ModifiersState},
    window::Window,
};

use self::action::{Action, KeyState};

pub struct GameSystemParams {
    pub window_ref: Arc<Window>,
    pub event_receiver: Receiver<Event<'static, ()>>,
    pub control_flow: SharedControlFlow,
    pub game_path: PathBuf,
}

pub fn spawn(params: GameSystemParams) -> JoinHandle<()> {
    thread::spawn(move || GameSystem::new(params).run())
}

struct GameSystem {
    event_receiver: Receiver<Event<'static, ()>>,
    control_flow: SharedControlFlow,
    modifiers: ModifiersState,
    key_bindings: HashMap<KeyState, Action>,
    big_reader: BigReader<BufReader<File>>,
}

impl GameSystem {
    fn new(params: GameSystemParams) -> Self {
        println!("{:?}", params.game_path);

        let graphics_path = params.game_path.join("data/graphics/graphics.big");

        println!("{:?}", graphics_path);

        let graphics_file = BufReader::new(File::open(graphics_path).unwrap());

        Self {
            event_receiver: params.event_receiver,
            control_flow: params.control_flow,
            key_bindings: HashMap::default(),
            modifiers: ModifiersState::empty(),
            big_reader: BigReader::new(graphics_file),
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
