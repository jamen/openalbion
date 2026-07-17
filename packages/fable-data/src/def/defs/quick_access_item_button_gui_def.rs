use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CQuickAccessItemButtonGuiDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct QuickAccessItemButtonGuiDef {
        "ButtonGraphic" => pub button_graphic: i32,
        "Position" => pub position: Vector2D,
        "Offset" => pub offset: Vector2D,
    }
}
