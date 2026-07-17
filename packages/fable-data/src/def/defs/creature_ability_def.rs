use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CREATURE_ABILITY` — C++ `CCreatureAbilityDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureAbilityDef {
        "Type" => pub type_: CreatureAbility,
    }
}
