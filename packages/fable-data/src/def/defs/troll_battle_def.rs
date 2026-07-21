use crate::def_struct;

def_struct! {
    /// `CTrollBattleDef` — C++ `CTrollBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TrollBattleDef {
        "GroundPoundSpeedMultiplierStart" => pub ground_pound_speed_multiplier_start: f32 = 1.0,
        "GroundPoundSpeedMultiplierEnd" => pub ground_pound_speed_multiplier_end: f32 = 1.0,
        "AttackPosPredictiveness" => pub attack_pos_predictiveness: f32,
        "PercentageChanceToUseSecondaryShotType" => pub percentage_chance_to_use_secondary_shot_type: f32,
        "SecondaryShotType" => pub secondary_shot_type: i32,
        "MinLaughDistance" => pub min_laugh_distance: f32 = 8.0,
        "MaxTimeBetweenKnockdownAndBoast" => pub max_time_between_knockdown_and_boast: f32 = 10.5,
        "FallDownExplosionIndex" => pub fall_down_explosion_index: i32,
        "DeathExplosionIndex" => pub death_explosion_index: i32,
    }
}
