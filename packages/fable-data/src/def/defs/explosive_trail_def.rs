use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExplosiveTrailDef` — C++ `CExplosiveTrailDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExplosiveTrailDef {
        "TrapDefIndex" => pub trap_def_index: i32,
        "SecondsBetweenTraps" => pub seconds_between_traps: f32,
        "DistanceBetweenTraps" => pub distance_between_traps: f32,
        "SecondsForTrapPersistance" => pub seconds_for_trap_persistance: f32,
        "SecondsTrailLifetime" => pub seconds_trail_lifetime: f32,
        "DistanceForLateralDeviation" => pub distance_for_lateral_deviation: f32,
        "DegreesMaxTurnTowardsTargetPerTrap" => pub degrees_max_turn_towards_target_per_trap: f32,
        "FractionOfLifetimeToTurnTowardsTarget" => pub fraction_of_lifetime_to_turn_towards_target: f32,
        "SecondsIcicleAttackTargetPositionPredictionTime" => pub seconds_icicle_attack_target_position_prediction_time: f32,
    }
}
