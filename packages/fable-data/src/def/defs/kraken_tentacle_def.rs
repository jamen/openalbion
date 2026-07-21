use crate::def_struct;

def_struct! {
    /// `CKrakenTentacleDef` — C++ `CKrakenTentacleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct KrakenTentacleDef {
        "TentacleSmackDownTime" => pub tentacle_smack_down_time: f32 = 8.0,
        "TentacleTurnSpeed" => pub tentacle_turn_speed: f32 = 3.0,
        "TentacleMoveSpeed" => pub tentacle_move_speed: f32 = 0.1,
        "TentacleStrikeAngleTolerence" => pub tentacle_strike_angle_tolerence: f32 = 5.0,
        "TentacleStrikeDistTolerence" => pub tentacle_strike_dist_tolerence: f32 = 0.5,
        "TentacleHeightOffset" => pub tentacle_height_offset: f32 = 0.3,
        "TentacleTimeBeforeExplosion" => pub tentacle_time_before_explosion: f32 = 1.0,
        "TentacleTimeBetweenAttacks" => pub tentacle_time_between_attacks: f32 = 10.0,
        "TentacleTimeBeforeObstructionRemoval" => pub tentacle_time_before_obstruction_removal: f32 = 0.5,
        "TentacleObstructionRadius" => pub tentacle_obstruction_radius: f32 = 1.5,
        "TentacleExplosionBodgeAngle" => pub tentacle_explosion_bodge_angle: f32 = 5.0,
        "TentacleExplosionRadius" => pub tentacle_explosion_radius: f32 = 10.0,
        "TentacleObstructionRadiusMin" => pub tentacle_obstruction_radius_min: f32 = 5.0,
        "TentacleObstructionRadiusMax" => pub tentacle_obstruction_radius_max: f32 = 15.0,
    }
}
