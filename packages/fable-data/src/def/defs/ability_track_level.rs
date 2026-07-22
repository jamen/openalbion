use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AbilityTrackLevelDef {
    #[def("ExperienceCost")]
    pub experience_cost: i32,
    #[def("AbilityUnlocked")]
    pub ability_unlocked: HeroAbility,
    #[def("Description")]
    pub description: i32,
}
