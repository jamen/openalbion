use futures::executor::block_on;

use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use renderer::Renderer;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();

    let size = [1024, 768u32];

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::<u32>::from(size))
        .with_visible(false)
        .with_title("Rust Renderer Example")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut renderer = block_on(Renderer::new(&window, size)).unwrap();

    //  TODO: Populate the renderer with graphics at some point

    renderer.render();

    window.set_visible(true);

    event_loop.run(|_event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;
    });
}
