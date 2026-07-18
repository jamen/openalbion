use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CJackOfBladesBattleDef` — C++ `CJackOfBladesBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct JackOfBladesBattleDef {
        "IdealShootDistance" => pub ideal_shoot_distance: f32,
        "NumGetHitsBeforeRush" => pub num_get_hits_before_rush: i32,
        "TimeBetweenShots" => pub time_between_shots: f32,
        "ScreamInAirTime" => pub scream_in_air_time: f32 = 3.0,
        "TimeBetweenPillars" => pub time_between_pillars: f32 = 0.5,
        "BattleFloatHeight" => pub battle_float_height: f32,
        "BattleScale" => pub battle_scale: f32 = 1.0,
        "ScaleUpTime" => pub scale_up_time: f32,
        "ScaleDownTime" => pub scale_down_time: f32,
        "DeathScreamTime" => pub death_scream_time: f32 = 5.0,
        "DeathHeight" => pub death_height: f32 = 5.0,
        "BurnDamage" => pub burn_damage: f32 = 3.0,
        "FlashDuration" => pub flash_duration: f32 = 2.0,
        "FlashBuildUpTime" => pub flash_build_up_time: f32 = 1.4,
        "RushTriggerDistance" => pub rush_trigger_distance: f32 = 25.0,
        "RushToDistance" => pub rush_to_distance: f32 = 19.0,
        "RushAwayDistance" => pub rush_away_distance: f32 = 15.0,
    }
}
