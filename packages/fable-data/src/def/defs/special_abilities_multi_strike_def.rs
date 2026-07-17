use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_MULTI_STRIKE_DEF` — C++ `CSpecialAbilitiesMultiStrikeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesMultiStrikeDef {
        "SoundLevel0" => pub sound_level0: DefString,
        "SoundLevel1" => pub sound_level1: DefString,
        "SoundLevel2" => pub sound_level2: DefString,
        "SoundLevel3" => pub sound_level3: DefString,
        "AnimationLevel0" => pub animation_level0: DefString,
        "AnimationLevel1" => pub animation_level1: DefString,
        "AnimationLevel2" => pub animation_level2: DefString,
        "AnimationLevel3" => pub animation_level3: DefString,
        "HighlightWidth" => pub highlight_width: f32,
        "HighlightColor" => pub highlight_color: Vec<i32>,
        "DelayTimeSecs" => pub delay_time_secs: Vec<f32>,
        "NumHits" => pub num_hits: Vec<i32>,
        "PerHitDamagerMultiplier" => pub per_hit_damager_multiplier: Vec<f32>,
        "AttachmentPoint" => pub attachment_point: Vec<String>,
    }
}
