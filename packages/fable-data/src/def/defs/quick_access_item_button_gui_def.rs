use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CQuickAccessItemButtonGuiDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct QuickAccessItemButtonGuiDef {
    #[def("ButtonGraphic")]
    pub button_graphic: i32,
    #[def("Position")]
    pub position: Vector2D,
    #[def("Offset")]
    pub offset: Vector2D,
}
