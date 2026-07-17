use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSpotLightDef` — C++ `CSpotLightDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpotLightDef {
        "Colour" => pub colour: RGBColour,
        "InnerRadius" => pub inner_radius: f32,
        "OuterRadius" => pub outer_radius: f32,
        "Width" => pub width: f32,
        "Flicker" => pub flicker: f32,
        "FlickerSpeed" => pub flicker_speed: f32,
        "Angle" => pub angle: f32,
    }
}
