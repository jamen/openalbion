use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CAbilityTrackLevelDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct AbilityTrackLevelDef {
        "ExperienceCost" => pub experience_cost: i32,
        "AbilityUnlocked" => pub ability_unlocked: HeroAbility,
        "Description" => pub description: i32,
    }
}
