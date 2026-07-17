use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_ELECTROCUTED_EFFECT_DEF` — C++ `CSpecialAbilitiesElectrocutedEffectDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesElectrocutedEffectDef {
        "FadeInTimeSecs" => pub fade_in_time_secs: f32,
        "FadeOutTimeStartSecs" => pub fade_out_time_start_secs: f32,
        "FadeOutTimeSecs" => pub fade_out_time_secs: f32,
        "Color" => pub color: Vec<i32>,
        "HighlightColor" => pub highlight_color: Vec<i32>,
        "HighlightWidth" => pub highlight_width: f32,
        "HeightSmall" => pub height_small: f32,
        "HeightMedium" => pub height_medium: f32,
    }
}
