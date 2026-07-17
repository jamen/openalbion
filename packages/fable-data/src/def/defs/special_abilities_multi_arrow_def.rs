use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_MULTI_ARROW_DEF` — C++ `CSpecialAbilitiesMultiArrowDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesMultiArrowDef {
        "NumberOfShotsLevel0" => pub number_of_shots_level0: i32,
        "NumberOfShotsLevel1" => pub number_of_shots_level1: i32,
        "NumberOfShotsLevel2" => pub number_of_shots_level2: i32,
        "NumberOfShotsLevel3" => pub number_of_shots_level3: i32,
        "NumberInVolleyLevel0" => pub number_in_volley_level0: i32,
        "NumberInVolleyLevel1" => pub number_in_volley_level1: i32,
        "NumberInVolleyLevel2" => pub number_in_volley_level2: i32,
        "NumberInVolleyLevel3" => pub number_in_volley_level3: i32,
        "AmmoEffectCreationDelaySecs" => pub ammo_effect_creation_delay_secs: f32,
        "BowHighlightWidth" => pub bow_highlight_width: f32,
        "BowHighlightColor" => pub bow_highlight_color: Vec<i32>,
        "MultiArrowPerArrowDamageMultiplier" => pub multi_arrow_per_arrow_damage_multiplier: Vec<f32>,
        "CastSound" => pub cast_sound: DefString,
    }
}
