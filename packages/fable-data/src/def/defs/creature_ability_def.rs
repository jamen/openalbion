use crate::DefStruct;
use crate::def::prelude::*;

/// `CREATURE_ABILITY` — C++ `CCreatureAbilityDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CreatureAbilityDef {
    #[def("Type")]
    pub type_: CreatureAbility,
}
