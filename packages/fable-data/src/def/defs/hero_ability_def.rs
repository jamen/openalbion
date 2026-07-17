use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `HERO_ABILITY` — C++ `CHeroAbilityDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroAbilityDef {
        "Ability" => pub ability: HeroAbility,
        "Stat" => pub stat: HeroExperienceStatCategory,
        "IconGraphicIndex" => pub icon_graphic_index: i32,
        "IconEffectIndex" => pub icon_effect_index: i32,
        "IconActiveEffectIndex" => pub icon_active_effect_index: i32,
        "IconEffectOffset" => pub icon_effect_offset: Vector2D,
        "Name" => pub name: i32,
        "Description" => pub description: i32,
        "StaminaCost" => pub stamina_cost: Vec<i32>,
        "StatExperienceGainedOnUse" => pub stat_experience_gained_on_use: i32,
        "ExperienceCostsToUpgrade" => pub experience_costs_to_upgrade: Vec<i32>,
        "LevDescription" => pub lev_description: Vec<i32>,
        "ABXYPriority" => pub abxy_priority: i32,
        "MoralityCostFactor" => pub morality_cost_factor: f32,
        "Aggressive" => pub aggressive: bool,
        "MaxedOutDescription" => pub maxed_out_description: i32,
        "DummyObject" => pub dummy_object: DefIndex,
    }
}
