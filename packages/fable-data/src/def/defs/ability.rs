use crate::DefStruct;
use crate::def::enums::HeroAbility;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AbilityDef {
    #[def("Ability")]
    pub ability: HeroAbility,
}
