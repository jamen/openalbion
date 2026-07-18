use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `NUISystem::CUIStateDef` — C++ `NUISystem::CUIStateDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct UiStateDef {
        "GraphicIndex" => pub graphic_index: u32,
        "PositionX" => pub position_x: f32,
        "PositionY" => pub position_y: f32,
        "ZoomX" => pub zoom_x: f32,
        "ZoomY" => pub zoom_y: f32,
        "ColourR" => pub colour_r: f32,
        "ColourG" => pub colour_g: f32,
        "ColourB" => pub colour_b: f32,
        "ColourA" => pub colour_a: f32,
        "UpdateTime" => pub update_time: f32,
        "StateChangeType" => pub state_change_type: i32,
        "LinearChange" => pub linear_change: bool,
        "StateChangeFlag" => pub state_change_flag: u32,
        "ChildrenNotAffected" => pub children_not_affected: Vec<i32>,
    }
}

