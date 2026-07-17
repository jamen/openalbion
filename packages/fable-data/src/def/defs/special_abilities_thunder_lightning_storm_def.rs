use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_THUNDER_LIGHTNING_STORM_DEF` — C++ `CSpecialAbilitiesThunderLightningStormDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesThunderLightningStormDef {
        "NumBeams" => pub num_beams: i32,
        "StartRadius" => pub start_radius: f32,
        "EndRadius" => pub end_radius: f32,
        "SkyBeamStartTime" => pub sky_beam_start_time: f32,
        "MainBeamsStartTime" => pub main_beams_start_time: f32,
        "BeamRotationStartSpeed" => pub beam_rotation_start_speed: f32,
        "BeamRotationEndSpeed" => pub beam_rotation_end_speed: f32,
        "TotalLifetime" => pub total_lifetime: f32,
        "Damage" => pub damage: f32,
        "InitSound" => pub init_sound: DefString,
        "LoopingSound" => pub looping_sound: DefString,
    }
}
