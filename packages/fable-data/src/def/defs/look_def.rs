use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CLookDef` — C++ `CLookDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct LookDef {
        "EyesTurnRangeXY" => pub eyes_turn_range_xy: f32 = 40.0,
        "EyesTurnRangeYZ" => pub eyes_turn_range_yz: f32 = 20.0,
        "HeadTurnRangeXY" => pub head_turn_range_xy: f32 = 180.0,
        "HeadTurnRangeYZUp" => pub head_turn_range_yz_up: f32 = 90.0,
        "HeadTurnRangeYZDown" => pub head_turn_range_yz_down: f32 = 90.0,
        "MaxTurnSpeed" => pub max_turn_speed: f32 = 360.0,
        "AllowTurningViaAnimation" => pub allow_turning_via_animation: bool = true,
        "AnimationTurnSpeedMultiplier" => pub animation_turn_speed_multiplier: f32 = 1.0,
        "CombatMaxTurnSpeed" => pub combat_max_turn_speed: f32 = 360.0,
        "CombatAllowTurningViaAnimation" => pub combat_allow_turning_via_animation: bool = true,
        "AllowOversteer" => pub allow_oversteer: bool = true,
        "AllowUndersteer" => pub allow_understeer: bool = true,
        "DrawEyePrimitives" => pub draw_eye_primitives: bool = true,
        "CutOffNoLooking" => pub cut_off_no_looking: f32,
        "AlwaysUpdateHeadPos" => pub always_update_head_pos: bool,
    }
}
