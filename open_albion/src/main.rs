use ember::{Ember,EmberConfig};

use ember::winit::event::{Event,WindowEvent,KeyboardInput,VirtualKeyCode,ElementState};
use ember::winit::event_loop::ControlFlow;

use fable_format::Lev;

pub struct Scene {
    pub landscape: Lev,
}

fn main() {
    let config = EmberConfig {
        title: "Open Albion",
        width: 1024,
        height: 768,
    };

    let Ember {
        window,
        window_event_loop,
        ..
    } = Ember::new(config).expect("Failed to initialize app.");

    window_event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                println!("{:?}", event);
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode, state, .. }, .. } => {
                        match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit;
                            },
                            _key => {}
                        }
                    },
                    _window_event => {}
                }
            },
            Event::MainEventsCleared => {
                window.request_redraw()
            },
            Event::RedrawRequested(_window_id) => {
                // TODO: Draw
            },
            _event => {}
        }
    })
}

