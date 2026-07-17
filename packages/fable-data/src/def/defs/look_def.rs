use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CLookDef` — C++ `CLookDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct LookDef {
        "EyesTurnRangeXY" => pub eyes_turn_range_xy: f32,
        "EyesTurnRangeYZ" => pub eyes_turn_range_yz: f32,
        "HeadTurnRangeXY" => pub head_turn_range_xy: f32,
        "HeadTurnRangeYZUp" => pub head_turn_range_yz_up: f32,
        "HeadTurnRangeYZDown" => pub head_turn_range_yz_down: f32,
        "MaxTurnSpeed" => pub max_turn_speed: f32,
        "AllowTurningViaAnimation" => pub allow_turning_via_animation: bool,
        "AnimationTurnSpeedMultiplier" => pub animation_turn_speed_multiplier: f32,
        "CombatMaxTurnSpeed" => pub combat_max_turn_speed: f32,
        "CombatAllowTurningViaAnimation" => pub combat_allow_turning_via_animation: bool,
        "AllowOversteer" => pub allow_oversteer: bool,
        "AllowUndersteer" => pub allow_understeer: bool,
        "DrawEyePrimitives" => pub draw_eye_primitives: bool,
        "CutOffNoLooking" => pub cut_off_no_looking: f32,
        "AlwaysUpdateHeadPos" => pub always_update_head_pos: bool,
    }
}
