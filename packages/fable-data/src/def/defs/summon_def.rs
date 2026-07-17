use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSummonDef` — C++ `CSummonDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SummonDef {
        "CreatureToSummon" => pub creature_to_summon: i32,
        "CreatureFamilyToSummon" => pub creature_family_to_summon: i32,
        "NumberToSummon" => pub number_to_summon: i32,
        "SummonRadius" => pub summon_radius: f32,
        "MySummonLimit" => pub my_summon_limit: i32,
        "MinimumSummon" => pub minimum_summon: i32,
        "TimeBetweenSummons" => pub time_between_summons: f32,
        "SummonEffect" => pub summon_effect: i32,
        "SummoneeEffect" => pub summonee_effect: i32,
    }
}
