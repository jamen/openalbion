use crate::DefStruct;

/// `CSummonDef` — C++ `CSummonDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SummonDef {
    #[def("CreatureToSummon")]
    pub creature_to_summon: i32,
    #[def("CreatureFamilyToSummon")]
    pub creature_family_to_summon: i32,
    #[def("NumberToSummon")]
    pub number_to_summon: i32,
    #[def("SummonRadius")]
    pub summon_radius: f32,
    #[def("MySummonLimit")]
    pub my_summon_limit: i32,
    #[def("MinimumSummon")]
    pub minimum_summon: i32,
    #[def("TimeBetweenSummons", default = 3.0)]
    pub time_between_summons: f32,
    #[def("SummonEffect")]
    pub summon_effect: i32,
    #[def("SummoneeEffect")]
    pub summonee_effect: i32,
}
