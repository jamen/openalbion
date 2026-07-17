use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CKrakenTentacleDef` — C++ `CKrakenTentacleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct KrakenTentacleDef {
        "TentacleSmackDownTime" => pub tentacle_smack_down_time: f32,
        "TentacleTurnSpeed" => pub tentacle_turn_speed: f32,
        "TentacleMoveSpeed" => pub tentacle_move_speed: f32,
        "TentacleStrikeAngleTolerence" => pub tentacle_strike_angle_tolerence: f32,
        "TentacleStrikeDistTolerence" => pub tentacle_strike_dist_tolerence: f32,
        "TentacleHeightOffset" => pub tentacle_height_offset: f32,
        "TentacleTimeBeforeExplosion" => pub tentacle_time_before_explosion: f32,
        "TentacleTimeBetweenAttacks" => pub tentacle_time_between_attacks: f32,
        "TentacleTimeBeforeObstructionRemoval" => pub tentacle_time_before_obstruction_removal: f32,
        "TentacleObstructionRadius" => pub tentacle_obstruction_radius: f32,
        "TentacleExplosionBodgeAngle" => pub tentacle_explosion_bodge_angle: f32,
        "TentacleExplosionRadius" => pub tentacle_explosion_radius: f32,
        "TentacleObstructionRadiusMin" => pub tentacle_obstruction_radius_min: f32,
        "TentacleObstructionRadiusMax" => pub tentacle_obstruction_radius_max: f32,
    }
}
