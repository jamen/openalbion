use crate::DefStruct;
use crate::def::prelude::*;

/// `CAbilityDef` — C++ `CAbilityDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AbilityDef {
    #[def("Ability")]
    pub ability: HeroAbility,
}
