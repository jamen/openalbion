use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCreatureStatsDef` — C++ `CCreatureStatsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureStatsDef {
        "MoralityForKill" => pub morality_for_kill: i32,
        "MoralityForPunch" => pub morality_for_punch: i32,
        "MoralityForWeaponHit" => pub morality_for_weapon_hit: i32,
        "ExperienceWorth" => pub experience_worth: i32,
        "RenownWorth" => pub renown_worth: i32,
        "Speed" => pub speed: f32,
        "Dexterity" => pub dexterity: f32,
        "StrengthDamageMultiplier" => pub strength_damage_multiplier: f32,
    }
}
