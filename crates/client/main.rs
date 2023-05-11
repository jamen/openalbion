pub mod game;
pub mod reader;
pub mod render;
pub mod window;

use game::GameSystemParams;
use render::RenderSystemParams;
use std::{env, path::PathBuf, sync::mpsc::channel};
use window::{SharedControlFlow, WindowSystemParams};

fn main() {
    env_logger::init();

    let game_path = env::args()
        .skip(1)
        .next()
        .map(PathBuf::from)
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let (event_loop, window_ref) = window::new().unwrap();

    // Shared state
    let control_flow = SharedControlFlow::default();
    let (event_sender, event_receiver) = channel();

    // Start render system
    render::spawn(RenderSystemParams {
        window_ref: window_ref.clone(),
    });

    // Start game system
    game::spawn(GameSystemParams {
        window_ref: window_ref.clone(),
        event_receiver,
        control_flow: control_flow.clone(),
        game_path,
    });

    window::spawn(WindowSystemParams {
        event_loop,
        control_flow,
        event_sender,
    });
}
