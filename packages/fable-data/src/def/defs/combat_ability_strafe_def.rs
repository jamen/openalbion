use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCombatAbilityStrafeDef` — C++ `CCombatAbilityStrafeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatAbilityStrafeDef {
        "RotationSpeed" => pub rotation_speed: f32,
        "CombatAbility" => pub combat_ability: CombatAbilityData,
    }
}
