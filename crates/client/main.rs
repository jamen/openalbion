pub mod game;
pub mod render;
pub mod window;

use game::{GameSystem, GameSystemParams};
use render::{RenderSystem, RenderSystemParams};
use std::{env, path::PathBuf, sync::mpsc::channel};
use window::{ControlFlowRef, WindowSystem, WindowSystemParams};

fn main() {
    env_logger::init();

    let game_path = env::args()
        .skip(1)
        .next()
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let (event_loop, window_ref) = window::new().unwrap();
    let (event_sender, event_receiver) = channel();
    let control_flow = ControlFlowRef::default();

    // Start render system
    let render_handle = RenderSystem::spawn(RenderSystemParams {
        window_ref: window_ref.clone(),
    });

    // Start game system
    let game_handle = GameSystem::spawn(GameSystemParams {
        window_ref: window_ref.clone(),
        event_receiver,
        control_flow: control_flow.clone(),
        game_path,
    });

    WindowSystem::start(WindowSystemParams {
        event_loop,
        control_flow,
        event_sender,
        render_handle,
        game_handle,
    });
}
