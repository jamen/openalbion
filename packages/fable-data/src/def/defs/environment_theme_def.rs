use crate::def_struct;

def_struct! {
    /// `CEnvironmentThemeDef` — a sub-component of `ENVIRONMENT_THEME_DAY`
    /// (`CEnvironmentThemeDaySetDef::Time`), not a top-level def type.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EnvironmentThemeDef {
        "TimeOfDay" => pub time_of_day: f32,
        "MoonLit" => pub moon_lit: bool,
        "FogStartZ" => pub fog_start_z: f32,
        "FogEndZ" => pub fog_end_z: f32,
        "SkyTexture0" => pub sky_texture0: i32,
        "SkyTexture1" => pub sky_texture1: i32,
        "SkyTexture1Blend" => pub sky_texture1_blend: f32,
        "CloudLowerLayerTexture0" => pub cloud_lower_layer_texture0: i32,
        "CloudLowerLayerTexture1" => pub cloud_lower_layer_texture1: i32,
        "CloudLowerLayerTexture1Blend" => pub cloud_lower_layer_texture1_blend: f32,
        "CloudLowerLayerSpeedMultiplier" => pub cloud_lower_layer_speed_multiplier: f32,
        "CloudUpperLayerTexture0" => pub cloud_upper_layer_texture0: i32,
        "CloudUpperLayerTexture1" => pub cloud_upper_layer_texture1: i32,
        "CloudUpperLayerTexture1Blend" => pub cloud_upper_layer_texture1_blend: f32,
        "CloudUpperLayerSpeedMultiplier" => pub cloud_upper_layer_speed_multiplier: f32,
        "WaterColourToReflectionBlend" => pub water_colour_to_reflection_blend: f32,
        "WaterAlphaFactor" => pub water_alpha_factor: f32,
        "WaterSpecularHightlightFactor" => pub water_specular_hightlight_factor: f32,
        "RainStrength" => pub rain_strength: f32,
        "SnowStrength" => pub snow_strength: f32,
        "MistAlpha" => pub mist_alpha: f32,
        "LightningFrequency" => pub lightning_frequency: f32,
        "GlowThresholdScale" => pub glow_threshold_scale: f32,
        "GlowBloomScale" => pub glow_bloom_scale: f32,
        "GlowMotionBlur" => pub glow_motion_blur: f32,
        "ShadowFactor" => pub shadow_factor: f32,
        "FadedShadowFactor" => pub faded_shadow_factor: f32,
        "WaterRefractionBlendStart" => pub water_refraction_blend_start: f32,
        // Non-zero C++ constructor defaults (verified constant across all retail
        // keyframes; the Anniversary text comments these out and relies on them).
        "WaterRefractionBlendEnd" => pub water_refraction_blend_end: f32 = 15.0,
        "WaterFlowSpeedFactor" => pub water_flow_speed_factor: f32 = 0.25,
        "WaterOscilationSpeed" => pub water_oscilation_speed: f32 = 0.05,
        "WaterSwellFactor" => pub water_swell_factor: f32 = 0.0625,
        "WaterShoreSwellFactor" => pub water_shore_swell_factor: f32 = 0.041666668,
        "WaterReflectionOffset" => pub water_reflection_offset: f32 = 0.01,
    }
}

