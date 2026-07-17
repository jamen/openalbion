use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CWallMountEffectsDef` — C++ `CWallMountEffectsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WallMountEffectsDef {
        "Effect" => pub effect: WallMountEffects,
        "HealingAmount" => pub healing_amount: f32,
        "SecondsBetweenEffectUsesAllowed" => pub seconds_between_effect_uses_allowed: DefIndex,
    }
}
