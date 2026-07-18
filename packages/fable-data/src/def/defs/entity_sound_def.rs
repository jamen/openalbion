use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CEntitySoundDef` — C++ `CEntitySoundDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EntitySoundDef {
        "SoundMap" => pub sound_map: SoundMap,
        "SoundIdentifier" => pub sound_identifier: String,
        "VoicePitchOverride" => pub voice_pitch_override: f32 = 1.0,
        "AnimCriteriaClipDistance" => pub anim_criteria_clip_distance: i32 = 22,
    }
}
