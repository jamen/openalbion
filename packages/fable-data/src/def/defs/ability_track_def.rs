use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CAbilityTrackDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct AbilityTrackDef {
        "ExperienceStat" => pub experience_stat: HeroExperienceStatCategory,
        "Abilities" => pub abilities: Vec<AbilityTrackLevelDef>,
        "MaxedOutDescription" => pub maxed_out_description: i32,
    }
}
