use crate::def_struct;

def_struct! {
    /// `CCreatureStatsDef` — C++ `CCreatureStatsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureStatsDef {
        "MoralityForKill" => pub morality_for_kill: i32 = -100,
        "MoralityForPunch" => pub morality_for_punch: i32 = -1,
        "MoralityForWeaponHit" => pub morality_for_weapon_hit: i32 = -5,
        "ExperienceWorth" => pub experience_worth: i32,
        "RenownWorth" => pub renown_worth: i32,
        "Speed" => pub speed: f32 = 0.5,
        "Dexterity" => pub dexterity: f32 = 0.5,
        "StrengthDamageMultiplier" => pub strength_damage_multiplier: f32 = 1.0,
    }
}
