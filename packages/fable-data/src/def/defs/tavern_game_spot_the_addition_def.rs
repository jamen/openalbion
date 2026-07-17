use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernGameSpotTheAdditionDef` — C++ `CTavernGameSpotTheAdditionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernGameSpotTheAdditionDef {
        "Pointer" => pub pointer: DefIndex,
        "Cover" => pub cover: DefIndex,
        "CoverWidth" => pub cover_width: f32,
        "CoverLength" => pub cover_length: f32,
        "Knicknacks" => pub knicknacks: Vec<i32>,
        "ItemsPerRound" => pub items_per_round: Vec<i32>,
        "JoystickScale" => pub joystick_scale: f32,
        "TimeToThink" => pub time_to_think: f32,
        "MaxCursorHeight" => pub max_cursor_height: f32,
        "KnickKnackRadius" => pub knick_knack_radius: f32,
        "ImpulseScale" => pub impulse_scale: f32,
        "PointerModel" => pub pointer_model: DefIndex,
        "PointerHeightOffset" => pub pointer_height_offset: f32,
        "TimeToMoveCover" => pub time_to_move_cover: f32,
        "TimeToPauseCover" => pub time_to_pause_cover: f32,
        "MaxCoverHeight" => pub max_cover_height: f32,
        "MinimumKnickKnackGap" => pub minimum_knick_knack_gap: f32,
        "LengthMovementScale" => pub length_movement_scale: f32,
        "HeightMovementScale" => pub height_movement_scale: f32,
        "HighlightWidth" => pub highlight_width: f32,
        "HighlightColour" => pub highlight_colour: RGBColour,
        "HighlightColourError" => pub highlight_colour_error: RGBColour,
    }
}
