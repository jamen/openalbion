use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCombatAbilityBlockHeavyWeaponAttackDef` | `CCombatAbilityBlockLightWeaponAttackDef` | `CCombatAbilityBlockProjectileWeaponAttackDef` | `CCombatAbilityBlockUnarmedAttackDef` — C++ `CCombatAbilityBlockDefBase`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatAbilityBlockDefBase {
        "BlockAngle" => pub block_angle: f32,
        "BlockCanBeBroken" => pub block_can_be_broken: bool,
        "ForceRecoilWhenBlocking" => pub force_recoil_when_blocking: bool,
        "CombatAbility" => pub combat_ability: CombatAbilityData,
        "ValidBlockWeaponTypes" => pub valid_block_weapon_types: Vec<WeaponClass>,
    }
}
