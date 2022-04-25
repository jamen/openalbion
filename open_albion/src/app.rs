use winit::event_loop::{EventLoop,ControlFlow};
use winit::event::{Event,WindowEvent};
use winit::window::WindowBuilder;
use winit::dpi::LogicalSize;

use glam::UVec2;

use crate::renderer::Renderer;

pub async fn run() -> ! {
    let event_loop = EventLoop::new();

    let size = UVec2::new(1024, 786);

    let window = WindowBuilder::new()
        .with_title("Open Albion")
        .with_inner_size(LogicalSize::new(size.x, size.y))
        .with_resizable(false)
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to make window");

    let mut renderer = Renderer::new(&window, size).await
        .expect("Failed to make renderer");

    renderer.render();

    window.set_visible(true);

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                if window_id == window.id() {
                    match window_event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(_size) => {
                            // let size: [u32; 2] = size.into();
                            // renderer.reconfigure_surface(size.into())
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                // state.update(); // Seperate tick rate from frame rate eventually (update loop and
                // render loop)
                renderer.render();
            }
            _ => {}
        }
    })
}