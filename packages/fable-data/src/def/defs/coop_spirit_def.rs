use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCoopSpiritDef` — C++ `CCoopSpiritDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CoopSpiritDef {
        "OnHitParticle" => pub on_hit_particle: DefString,
        "OnStrikeParticle" => pub on_strike_particle: DefString,
        "DefaultParticle" => pub default_particle: DefString,
        "MeleeTargetRange" => pub melee_target_range: f32,
        "AttackTargetParticle" => pub attack_target_particle: DefString,
    }
}
