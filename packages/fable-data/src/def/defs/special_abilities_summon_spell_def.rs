use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_SUMMON_SPELL_DEF` — C++ `CSpecialAbilitiesSummonSpellDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesSummonSpellDef {
        "TimeOfSummon" => pub time_of_summon: Vec<f32>,
        "BaseSummonCreatureName" => pub base_summon_creature_name: DefString,
        "FadeInTime" => pub fade_in_time: f32,
        "FadeOutTime" => pub fade_out_time: f32,
        "ResetCreatureManaCost" => pub reset_creature_mana_cost: i32,
        "SummonTintColour" => pub summon_tint_colour: Vec<i32>,
        "EyeLeftDummyPointName" => pub eye_left_dummy_point_name: String,
        "EyeRightDummyPointName" => pub eye_right_dummy_point_name: String,
    }
}
