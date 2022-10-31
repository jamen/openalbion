use futures::executor::block_on;

use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
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

    let _ = renderer.render();

    window.set_visible(true);

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                let _ = renderer.render();
            }
            _ => {}
        }
    });
}
