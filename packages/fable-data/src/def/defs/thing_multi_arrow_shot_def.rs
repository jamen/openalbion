use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CThingMultiArrowShotDef` — C++ `CThingMultiArrowShotDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThingMultiArrowShotDef {
        "AccelerationScalar" => pub acceleration_scalar: f32,
        "MaxAngle" => pub max_angle: f32,
        "MaxDistance" => pub max_distance: f32,
        "TimeToReachTarget" => pub time_to_reach_target: f32,
        "MaxAngleToTurnDegs" => pub max_angle_to_turn_degs: f32,
        "MultiArrowWeaponTrail" => pub multi_arrow_weapon_trail: i32,
        "MultiArrowHitEffect" => pub multi_arrow_hit_effect: i32,
        "MultiArrowTrailEffect" => pub multi_arrow_trail_effect: i32,
        "MultiArrowSpeed" => pub multi_arrow_speed: f32,
        "MultiArrowDeflectionByMelee" => pub multi_arrow_deflection_by_melee: bool,
        "MultiArrowHitEffectPercentage" => pub multi_arrow_hit_effect_percentage: f32,
    }
}
