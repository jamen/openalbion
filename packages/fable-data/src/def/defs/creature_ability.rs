use crate::DefStruct;
use crate::def::enums::CreatureAbility;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CreatureAbilityDef {
    #[def("Type")]
    pub type_: CreatureAbility,
}
