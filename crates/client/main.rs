pub mod game;
pub mod render;
pub mod window;

use game::GameSystemParams;
use render::RenderSystemParams;
use std::sync::mpsc::channel;
use window::{SharedControlFlow, WindowSystemParams};

fn main() {
    env_logger::init();

    let (event_loop, window) = window::new().unwrap();

    // Shared state
    let control_flow = SharedControlFlow::default();
    let (event_sender, event_receiver) = channel();

    // Start render system
    render::spawn(RenderSystemParams {
        window: window.clone(),
    });

    // Start game system
    game::spawn(GameSystemParams {
        window: window.clone(),
        event_receiver,
        control_flow: control_flow.clone(),
    });

    window::spawn(WindowSystemParams {
        event_loop,
        control_flow,
        event_sender,
    });
}
