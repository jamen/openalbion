use crate::DefStruct;
use crate::def::prelude::*;

/// `CCombatAbilityBlockCounterAttackDef` | `CCombatAbilityFlourishCounterAttackDef` | `CCombatAbilityGetHitCounterAttackDef` — C++ `CCombatAbilityAttackBase`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CombatAbilityAttackBase {
    #[def("Blockable", default = true)]
    pub blockable: bool,
    #[def("Knockdown")]
    pub knockdown: bool,
    #[def("Damage", default = 1.0)]
    pub damage: f32,
    #[def("CombatAbility")]
    pub combat_ability: CombatAbilityData,
}
