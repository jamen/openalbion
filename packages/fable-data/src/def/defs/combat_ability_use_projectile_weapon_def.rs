use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCombatAbilityUseProjectileWeaponDef` — C++ `CCombatAbilityUseProjectileWeaponDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatAbilityUseProjectileWeaponDef {
        "AbleToStrafeAndShoot" => pub able_to_strafe_and_shoot: bool,
        "ShotAccuracyPercentage" => pub shot_accuracy_percentage: i32,
        "CombatAbility" => pub combat_ability: CombatAbilityData,
    }
}
