use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCombatAbilityBlockCounterAttackDef` | `CCombatAbilityFlourishCounterAttackDef` | `CCombatAbilityGetHitCounterAttackDef` — C++ `CCombatAbilityAttackBase`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatAbilityAttackBase {
        "Blockable" => pub blockable: bool = true,
        "Knockdown" => pub knockdown: bool,
        "Damage" => pub damage: f32 = 1.0,
        "CombatAbility" => pub combat_ability: CombatAbilityData,
    }
}
