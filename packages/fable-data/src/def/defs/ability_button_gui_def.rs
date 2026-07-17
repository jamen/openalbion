use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CAbilityButtonGuiDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct AbilityButtonGuiDef {
        "ControllerType" => pub controller_type: ControllerType,
        "ControllerButton" => pub controller_button: XboxControllerButton,
        "MouseButton" => pub mouse_button: MouseButtonControl,
        "ButtonGraphic" => pub button_graphic: i32,
        "Position" => pub position: Vector2D,
        "Offset" => pub offset: Vector2D,
    }
}
