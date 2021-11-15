use std::collections::BTreeMap;
use std::time::Instant;

use winit::dpi::PhysicalPosition;
use winit::event::{
    ElementState, Event, KeyboardInput, ModifiersState, MouseButton, ScanCode, VirtualKeyCode,
    WindowEvent,
};
use winit::event_loop::ControlFlow;

pub(crate) struct InputState {
    // pub keys: [Option<Instant>; mem::variant_count::<VirtualKeyCode>()],
    pub keys: [Option<Instant>; 163],
    pub scancodes: BTreeMap<ScanCode, Instant>,
    pub modifiers: ModifiersState,
    pub mouse_left: Option<Instant>,
    pub mouse_middle: Option<Instant>,
    pub mouse_right: Option<Instant>,
    pub mouse_other: BTreeMap<u16, Instant>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            // keys: [None; mem::variant_count::<VirtualKeyCode>()],
            keys: [None; 163],
            scancodes: BTreeMap::new(),
            modifiers: ModifiersState::empty(),
            mouse_left: None,
            mouse_right: None,
            mouse_middle: None,
            mouse_other: BTreeMap::new(),
            cursor_position: None,
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode,
                            scancode,
                            state: element_state,
                            ..
                        },
                    ..
                } => match element_state {
                    ElementState::Pressed => self.key_down(*virtual_keycode, *scancode),
                    ElementState::Released => self.key_up(*virtual_keycode, *scancode),
                },
                WindowEvent::MouseInput {
                    button,
                    state: element_state,
                    ..
                } => match element_state {
                    ElementState::Pressed => self.mouse_up(*button),
                    ElementState::Released => self.mouse_down(*button),
                },
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.modifiers = *modifiers;
                    log::debug!("modifiers changed: {:?}", modifiers);
                }
                WindowEvent::CursorLeft { .. } => {
                    self.cursor_position = None;
                    log::debug!("cursor left");
                }
                WindowEvent::CursorMoved { position, .. } => self.cursor_position = Some(*position),
                _ => {}
            },
            _ => {}
        }
    }

    fn key_down(&mut self, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {
        let now = Instant::now();

        self.scancodes.insert(scancode, now);

        if let Some(keycode) = keycode {
            if self.keys[keycode as usize].is_none() {
                self.keys[keycode as usize] = Some(now);
                log::debug!("key down: keycode={:?} scancode={:?}", keycode, scancode);
            }
        }
    }

    fn key_up(&mut self, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {
        self.scancodes.remove(&scancode);

        if let Some(keycode) = keycode {
            if let Some(_instant) = self.keys[keycode as usize] {
                self.keys[keycode as usize] = None;
                log::debug!("key up: keycode={:?} scancode={:?}", keycode, scancode);
            }
        }
    }

    fn mouse_down(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_left = Some(Instant::now()),
            MouseButton::Right => self.mouse_right = Some(Instant::now()),
            MouseButton::Middle => self.mouse_middle = Some(Instant::now()),
            MouseButton::Other(x) => drop(self.mouse_other.insert(x, Instant::now())),
        };
        log::debug!("mouse down: button={:?}", button);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_left = None,
            MouseButton::Right => self.mouse_right = None,
            MouseButton::Middle => self.mouse_middle = None,
            MouseButton::Other(x) => drop(self.mouse_other.remove(&x)),
        };
        log::debug!("mouse up: button={:?}", button);
    }
}
