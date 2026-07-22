use crate::DefStruct;
use crate::def::prelude::*;

/// `CSpecialEffectsDef` — C++ `CSpecialEffectsDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SpecialEffectsDef {
    #[def("SpecialEffects")]
    pub special_effects: SpecialEffectsStringMap,
}
