use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
};

use super::GameSystem;

impl GameSystem {
    pub fn on_event(&mut self, event: Event<'static, ()>) {
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    // TODO: Other shutdown stuff

                    // Send exit signal to window loop

                    let mut control_flow = self.control_flow.lock().unwrap();

                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.modifiers = modifiers;
                }
                _window_event => {
                    // log::debug!("{:?}", _window_event);
                }
            },
            Event::DeviceEvent {
                event: device_event,
                ..
            } => match device_event {
                DeviceEvent::Key(KeyboardInput {
                    virtual_keycode,
                    state,
                    ..
                }) => {
                    let action = match self.key_bindings.get(&(virtual_keycode, self.modifiers)) {
                        Some(action) => *action,
                        None => {
                            log::debug!("{:?} {:?}", virtual_keycode, self.modifiers);
                            return;
                        }
                    };

                    match state {
                        ElementState::Pressed => self.on_action_begin(action),
                        ElementState::Released => self.on_action_finish(action),
                    }
                }
                _ => {}
            },
            // Other events
            _event => {
                // log::debug!("{:?}", _event);
            }
        }
    }
}
