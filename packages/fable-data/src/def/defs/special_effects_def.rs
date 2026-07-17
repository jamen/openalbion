use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSpecialEffectsDef` — C++ `CSpecialEffectsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialEffectsDef {
        "SpecialEffects" => pub special_effects: SpecialEffectsStringMap,
    }
}
