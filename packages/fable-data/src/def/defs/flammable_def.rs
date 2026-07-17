use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CFlammableDef` — C++ `CFlammableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FlammableDef {
        "FireResistance" => pub fire_resistance: i32,
        "MaxDamageCaused" => pub max_damage_caused: f32,
        "FireDamagePerCycle" => pub fire_damage_per_cycle: f32,
        "FireDamagePeriod" => pub fire_damage_period: f32,
        "EffectCreationSet" => pub effect_creation_set: FireEffectCreationSet,
    }
}
