use super::GameSystem;
use winit::event::{ModifiersState, VirtualKeyCode};

#[derive(Copy, Clone, Debug)]
pub enum Action {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Quit,
}

pub type KeyState = (Option<VirtualKeyCode>, ModifiersState);

impl GameSystem {
    pub fn load_default_key_bindings(&mut self) {
        use winit::event::{ModifiersState as M, VirtualKeyCode::*};

        self.key_bindings.extend([
            ((Some(W), M::empty()), Action::MoveForward),
            ((Some(A), M::empty()), Action::MoveLeft),
            ((Some(S), M::empty()), Action::MoveBackward),
            ((Some(D), M::empty()), Action::MoveRight),
            ((Some(Q), M::CTRL), Action::Quit),
            ((Some(Escape), M::empty()), Action::Quit),
        ]);
    }

    pub fn on_action_begin(&mut self, action: Action) {
        log::debug!("{:?}", action);
    }

    pub fn on_action_finish(&mut self, action: Action) {
        log::debug!("{:?}", action);
    }
}
