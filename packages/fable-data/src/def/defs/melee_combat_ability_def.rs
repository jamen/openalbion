use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `HERO_MELEE_COMBAT_ABILITY` — C++ `CMeleeCombatAbilityDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MeleeCombatAbilityDef {
        "FlourishAnimName" => pub flourish_anim_name: DefString,
        "FlourishResponseAnimName" => pub flourish_response_anim_name: DefString,
        "DamageMultiplier" => pub damage_multiplier: f32,
        "Flourish" => pub flourish: bool,
        "BlockCounter" => pub block_counter: bool,
        "AbilityButtonGraphicOverride" => pub ability_button_graphic_override: i32,
        "ValidWeaponClasses" => pub valid_weapon_classes: Vec<WeaponClass>,
        "Decapitate" => pub decapitate: bool,
        "KnockDownEffects" => pub knock_down_effects: i32,
        "EvadeAllHits" => pub evade_all_hits: bool,
        "CauseRecoil" => pub cause_recoil: bool,
        "StatsNeeded" => pub stats_needed: VecMap<i32, IdleStateGroup>,
        "ComboMultiplierLowerBound" => pub combo_multiplier_lower_bound: i32,
        "ComboMultiplierUpperBound" => pub combo_multiplier_upper_bound: i32,
    }
}
