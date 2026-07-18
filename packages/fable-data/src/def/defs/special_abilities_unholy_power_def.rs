use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_DIVINE_WRATH_DEF` | `SPECIAL_ABILITIES_UNHOLY_POWER_DEF` — C++ `CSpecialAbilitiesUnholyPowerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesUnholyPowerDef {
        "StartPowerupSound" => pub start_powerup_sound: Vec<DefString>,
        "LoopingPowerupSound" => pub looping_powerup_sound: Vec<DefString>,
        "ReleaseSound" => pub release_sound: Vec<DefString>,
        "ReleaseLoopSound" => pub release_loop_sound: Vec<DefString>,
        "FinishSound" => pub finish_sound: Vec<DefString>,
        "CancelSound" => pub cancel_sound: Vec<DefString>,
        "RandomFactorForNumSecsBetweenDamageHit" => pub random_factor_for_num_secs_between_damage_hit: f32 = 1.0,
        "NumSecsBetweenDamageHit" => pub num_secs_between_damage_hit: Vec<f32>,
        "DamagePerHit" => pub damage_per_hit: Vec<f32>,
        "DurationOfSpellSecs" => pub duration_of_spell_secs: Vec<f32>,
        "RadiusOfEffect" => pub radius_of_effect: Vec<f32>,
        "EnvChangeSecs" => pub env_change_secs: Vec<f32>,
        "EnvReturnSecs" => pub env_return_secs: Vec<f32>,
        "DoLifting" => pub do_lifting: bool,
        "VelocityOfLift" => pub velocity_of_lift: f32 = 0.1,
        "DistanceToLift" => pub distance_to_lift: f32 = 1.0,
        "StaminaCostPerSec" => pub stamina_cost_per_sec: Vec<f32>,
        "DamageTakenMultiplier" => pub damage_taken_multiplier: Vec<f32>,
        "SmallRingRadius" => pub small_ring_radius: f32,
        "LargeRingRadius" => pub large_ring_radius: f32,
    }
}
