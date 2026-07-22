use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SpecialEffectsDef {
    #[def("SpecialEffects")]
    pub special_effects: SpecialEffectsStringMap,
}
