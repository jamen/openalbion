use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SKY` — C++ `CSkyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SkyDef {
        "SunRadius" => pub sun_radius: i32,
        "SunTexture" => pub sun_texture: i32,
        "SunFlareRadius" => pub sun_flare_radius: i32,
        "SunFlareTexture" => pub sun_flare_texture: i32,
        "MoonRadius" => pub moon_radius: i32,
        "MoonTexture" => pub moon_texture: i32,
        "StarTexture" => pub star_texture: i32,
        "StarChartTextureSize" => pub star_chart_texture_size: i32,
        "StarSize" => pub star_size: f32,
        "StarChartFilter" => pub star_chart_filter: i32,
        "TwinkleInterval" => pub twinkle_interval: i32,
        "TwinkleSpeed" => pub twinkle_speed: f32,
        "TwinkleMin" => pub twinkle_min: f32,
        "TwinkleMax" => pub twinkle_max: f32,
        "FlareElements" => pub flare_elements: Vec<LensFlareElementDef>,
    }
}
