use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CKrakenDef` — C++ `CKrakenDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct KrakenDef {
        "HeightOffset" => pub height_offset: f32,
        "PoolRadius" => pub pool_radius: f32,
        "HeadRiseDelay" => pub head_rise_delay: f32,
        "HeadIdleTime" => pub head_idle_time: f32,
        "LowStrikesBeforeHigh" => pub low_strikes_before_high: i32,
        "LowStrikeChargeLoops" => pub low_strike_charge_loops: i32,
        "LowStrikeAttackLoops" => pub low_strike_attack_loops: i32,
        "HighStrikeAttackLoops" => pub high_strike_attack_loops: i32,
        "RoarLoops" => pub roar_loops: i32,
        "DelayBeforeLowBeam" => pub delay_before_low_beam: f32,
        "DelayBeforeHighBeam" => pub delay_before_high_beam: f32,
        "BeamExplosionRadius" => pub beam_explosion_radius: f32,
        "BeamTrackSpeed" => pub beam_track_speed: f32,
        "TargetingTrackSpeed" => pub targeting_track_speed: f32,
        "BeamMouthDistLow" => pub beam_mouth_dist_low: f32,
        "BeamMouthDistHigh" => pub beam_mouth_dist_high: f32,
        "TicksBetweenBeamExplosions" => pub ticks_between_beam_explosions: i32,
        "TentacleDefIndex" => pub tentacle_def_index: Vec<i32>,
        "TentacleMaxRiseDelay" => pub tentacle_max_rise_delay: f32,
    }
}
