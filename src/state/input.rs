use std::collections::BTreeMap;
use std::time::Instant;

use winit::dpi::PhysicalPosition;
use winit::event::{ModifiersState, MouseButton, ScanCode, VirtualKeyCode};

pub struct InputState {
    pub keys: Vec<Option<Instant>>,
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
            // keys: vec![None; mem::variant_count::<VirtualKeyCode>()],
            keys: vec![None; 163],
            scancodes: BTreeMap::new(),
            modifiers: ModifiersState::empty(),
            mouse_left: None,
            mouse_right: None,
            mouse_middle: None,
            mouse_other: BTreeMap::new(),
            cursor_position: None,
        }
    }

    pub fn key_down(&mut self, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {
        let now = Instant::now();

        self.scancodes.insert(scancode, now);

        if let Some(keycode) = keycode {
            if self.keys[keycode as usize].is_none() {
                self.keys[keycode as usize] = Some(now);
            }
        }
    }

    pub fn key_up(&mut self, keycode: Option<VirtualKeyCode>, scancode: ScanCode) {
        self.scancodes.remove(&scancode);

        if let Some(keycode) = keycode {
            if let Some(instant) = self.keys[keycode as usize] {
                self.keys[keycode as usize] = None;
            }
        }
    }

    pub fn mouse_down(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_left = Some(Instant::now()),
            MouseButton::Right => self.mouse_right = Some(Instant::now()),
            MouseButton::Middle => self.mouse_middle = Some(Instant::now()),
            MouseButton::Other(x) => drop(self.mouse_other.insert(x, Instant::now())),
        };
    }

    pub fn mouse_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_left = None,
            MouseButton::Right => self.mouse_right = None,
            MouseButton::Middle => self.mouse_middle = None,
            MouseButton::Other(x) => drop(self.mouse_other.remove(&x)),
        };
    }
}
