use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CThunderBattleDef` — C++ `CThunderBattleDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThunderBattleDef {
        "NumGetHitsBeforeStormAttack" => pub num_get_hits_before_storm_attack: i32,
        "NumMissesBeforeStormAttack" => pub num_misses_before_storm_attack: i32,
        "LightningDamage" => pub lightning_damage: f32,
        "BattleChargeLevel" => pub battle_charge_level: i32,
    }
}
