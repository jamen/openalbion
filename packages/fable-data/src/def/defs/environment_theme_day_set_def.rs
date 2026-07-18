use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `ENVIRONMENT_THEME_DAY` — C++ `CEnvironmentThemeDaySetDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EnvironmentThemeDaySetDef {
        "Time" => pub time: Vec<EnvironmentThemeDef>,
        "SunTilt" => pub sun_tilt: f32,
        "SunRotate" => pub sun_rotate: f32,
        "SunHeight" => pub sun_height: f32,
        "MoonTilt" => pub moon_tilt: f32,
        "MoonRotate" => pub moon_rotate: f32,
        "MoonHeight" => pub moon_height: f32,
        "ColourLookupColumn" => pub colour_lookup_column: i32,
        "EditorColour" => pub editor_colour: RGBColour,
        "FishWeightMult" => pub fish_weight_mult: f32,
    }
}

