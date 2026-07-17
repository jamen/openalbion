use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_BURNT_EFFECT_DEF` — C++ `CSpecialAbilitiesBurntEffectDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesBurntEffectDef {
        "FadeInTimeSecs" => pub fade_in_time_secs: f32,
        "FadeOutTimeStartSecs" => pub fade_out_time_start_secs: f32,
        "FadeOutTimeSecs" => pub fade_out_time_secs: f32,
        "Color" => pub color: Vec<i32>,
    }
}
