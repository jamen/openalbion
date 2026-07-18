use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CEngineLocalDetailObjectDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineLocalDetailObjectDef {
        "Mesh" => pub mesh: i32,
        "ShadowMesh" => pub shadow_mesh: i32,
        "Probability" => pub probability: f32 = 1.0,
        "ThemeBlendThreshold" => pub theme_blend_threshold: f32,
        "Scale" => pub scale: f32,
        "ScaleRandomElement" => pub scale_random_element: f32,
        "FadeStart" => pub fade_start: f32,
        "FadeEnd" => pub fade_end: f32,
        "ZSpriteMesh" => pub z_sprite_mesh: i32,
        "ZSpriteFadeStart" => pub z_sprite_fade_start: f32,
        "ZSpriteFadeEnd" => pub z_sprite_fade_end: f32,
        "SlopeFadeStart" => pub slope_fade_start: f32,
        "SlopeFadeEnd" => pub slope_fade_end: f32,
        "AlphaRef" => pub alpha_ref: i32 = -1,
        "AlphaMipBias" => pub alpha_mip_bias: f32,
        "WindSkewConstantFactor" => pub wind_skew_constant_factor: f32,
        "WindSkewRandomFactor" => pub wind_skew_random_factor: f32,
        "WindSkewSpeedFactor" => pub wind_skew_speed_factor: f32,
        "CastShadows" => pub cast_shadows: bool,
        "ReceiveShadows" => pub receive_shadows: bool = true,
        "IsRepeatedMesh" => pub is_repeated_mesh: bool,
        "IsZSprite" => pub is_z_sprite: bool,
        "AlphaIsBoolean" => pub alpha_is_boolean: bool,
        "HasLandscapeNormalLighting" => pub has_landscape_normal_lighting: bool,
        "TiltToSlope" => pub tilt_to_slope: bool,
        "HasWindSkew" => pub has_wind_skew: bool,
    }
}
