use crate::def_struct;

def_struct! {
    /// `CSnowTrollDef` — C++ `CSnowTrollDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SnowTrollDef {
        "IcicleAttackIndex" => pub icicle_attack_index: i32,
        "SecondsIcicleAttackTargetPositionPredictionTime" => pub seconds_icicle_attack_target_position_prediction_time: f32,
        "DistancePoundGroundRange" => pub distance_pound_ground_range: f32,
        "SecondsForIcicleAssault" => pub seconds_for_icicle_assault: f32,
        "SecondsLeadIntoIcicles" => pub seconds_lead_into_icicles: f32,
        "SecondsBetweenIcicles" => pub seconds_between_icicles: f32,
        "SecondsBetweenWarningFXAndTrigger" => pub seconds_between_warning_fx_and_trigger: f32,
        "SecondsForTrapToRemain" => pub seconds_for_trap_to_remain: f32,
        "DistanceRange" => pub distance_range: f32,
        "DistanceAroundTarget" => pub distance_around_target: f32,
        "DistanceExclusionAroundIcicles" => pub distance_exclusion_around_icicles: f32,
        "IcicleAssaultWarningFXIndex" => pub icicle_assault_warning_fx_index: i32,
        "IcicleAssaultTrapIndex" => pub icicle_assault_trap_index: i32,
        "IceTrailDistanceLimit" => pub ice_trail_distance_limit: f32 = -1.0,
    }
}
