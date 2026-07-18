use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `ENGINE_VIDEO_OPTIONS` — C++ `CEngineVideoOptionsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineVideoOptionsDef {
        "HiresTextureMemory" => pub hires_texture_memory: i32,
        "LODErrorTolerance" => pub lod_error_tolerance: f32,
        "CharacterLODErrorTolerance" => pub character_lod_error_tolerance: f32,
        "DrawDistanceMultiplier" => pub draw_distance_multiplier: f32,
        "DrawDistanceMinimum" => pub draw_distance_minimum: f32,
        "DrawDistanceMaximum" => pub draw_distance_maximum: f32,
        "RepeatedMeshDrawDistanceFactor" => pub repeated_mesh_draw_distance_factor: f32,
        "MinimumZSpriteAsMeshDistance" => pub minimum_z_sprite_as_mesh_distance: f32,
        "MaximumZSpriteAsMeshDistance" => pub maximum_z_sprite_as_mesh_distance: f32,
        "ZSpriteDrawDistanceMultiplier" => pub z_sprite_draw_distance_multiplier: f32,
        "ShadowBufferSize" => pub shadow_buffer_size: i32 = 1024,
        "ShadowDistanceScale" => pub shadow_distance_scale: f32 = 1.0,
        "Enable2DDisplacement" => pub enable2_d_displacement: bool,
        "Enable3DDisplacement" => pub enable3_d_displacement: bool,
        "EnableGlow" => pub enable_glow: bool,
        "EnableRadialBlur" => pub enable_radial_blur: bool,
        "EnableWaterReflection" => pub enable_water_reflection: bool,
        "EnableWeatherEffects" => pub enable_weather_effects: bool,
        "EnableColourFilter" => pub enable_colour_filter: bool,
        "WeatherDensity" => pub weather_density: f32 = 1.0,
        "EnableRepeatedMeshes" => pub enable_repeated_meshes: bool = true,
    }
}

