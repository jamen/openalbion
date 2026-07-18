use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_THUNDER_LIGHTNING_STORM_DEF` — C++ `CSpecialAbilitiesThunderLightningStormDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesThunderLightningStormDef {
        "NumBeams" => pub num_beams: i32 = 5,
        "StartRadius" => pub start_radius: f32 = 1.0,
        "EndRadius" => pub end_radius: f32 = 10.0,
        "SkyBeamStartTime" => pub sky_beam_start_time: f32 = 1.0,
        "MainBeamsStartTime" => pub main_beams_start_time: f32 = 3.0,
        "BeamRotationStartSpeed" => pub beam_rotation_start_speed: f32 = 0.5,
        "BeamRotationEndSpeed" => pub beam_rotation_end_speed: f32 = 0.5,
        "TotalLifetime" => pub total_lifetime: f32 = 6.0,
        "Damage" => pub damage: f32 = 5.0,
        "InitSound" => pub init_sound: DefString,
        "LoopingSound" => pub looping_sound: DefString,
    }
}
