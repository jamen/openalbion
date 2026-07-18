use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CLightDef` — C++ `CLightDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct LightDef {
        "Colour" => pub colour: RGBColour,
        "HaloGraphic" => pub halo_graphic: i32,
        "InnerRadius" => pub inner_radius: f32,
        "OuterRadius" => pub outer_radius: f32,
        "Flicker" => pub flicker: f32,
        "FlickerSpeed" => pub flicker_speed: f32 = 0.3,
        "StartActive" => pub start_active: bool = true,
        "SunlightAttenuator" => pub sunlight_attenuator: bool,
        "DaylightAttenuator" => pub daylight_attenuator: bool,
        "EffectChannel" => pub effect_channel: PointLightChannelEffect,
        "Inverted" => pub inverted: bool,
        "DaylightAttenuatorFadeInStart" => pub daylight_attenuator_fade_in_start: f32,
        "DaylightAttenuatorFadeInEnd" => pub daylight_attenuator_fade_in_end: f32 = 12.0,
        "DaylightAttenuatorFadeOutStart" => pub daylight_attenuator_fade_out_start: f32 = 12.0,
        "DaylightAttenuatorFadeOutEnd" => pub daylight_attenuator_fade_out_end: f32,
    }
}
