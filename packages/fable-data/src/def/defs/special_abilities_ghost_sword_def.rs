use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_GHOST_SWORD_DEF` — C++ `CSpecialAbilitiesGhostSwordDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesGhostSwordDef {
        "TimeOfSummonLevel0" => pub time_of_summon_level0: f32,
        "TimeOfSummonLevel1" => pub time_of_summon_level1: f32,
        "TimeOfSummonLevel2" => pub time_of_summon_level2: f32,
        "TimeOfSummonLevel3" => pub time_of_summon_level3: f32,
        "FadeInTime" => pub fade_in_time: f32,
        "FadeOutTime" => pub fade_out_time: f32,
        "CreationDistance" => pub creation_distance: f32,
    }
}
