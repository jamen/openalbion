use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSummonerDef` — C++ `CSummonerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SummonerDef {
        "LightningRangeForAutomaticUse" => pub lightning_range_for_automatic_use: f32,
        "LightningPercentChanceForRandomUse" => pub lightning_percent_chance_for_random_use: f32,
        "LightningPercentChanceForBoastUse" => pub lightning_percent_chance_for_boast_use: f32,
        "LightningRandomChanceTestSeconds" => pub lightning_random_chance_test_seconds: f32,
        "LightningSecondsToWaitBeforeReuse" => pub lightning_seconds_to_wait_before_reuse: f32,
        "ChargeRangeNear" => pub charge_range_near: f32,
        "ChargeRangeFar" => pub charge_range_far: f32,
        "ChargeRangeAttackInterrupt" => pub charge_range_attack_interrupt: f32,
        "ChargeSecondsToWaitBeforeReuse" => pub charge_seconds_to_wait_before_reuse: f32,
        "SummonerDeathExplosion" => pub summoner_death_explosion: i32,
        "LightningOrb" => pub lightning_orb: i32,
        "LightningOrbAttackFireheart" => pub lightning_orb_attack_fireheart: i32,
        "LightningOrbLifeSeconds" => pub lightning_orb_life_seconds: f32,
    }
}
