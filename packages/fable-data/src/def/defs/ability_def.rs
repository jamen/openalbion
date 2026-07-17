use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAbilityDef` — C++ `CAbilityDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AbilityDef {
        "Ability" => pub ability: HeroAbility,
    }
}
