use crate::def_struct;

def_struct! {
    /// `ENGINE` — C++ `CEngineDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineDef {
        "LODErrorTolerance" => pub lod_error_tolerance: f32,
        "CharacterLODErrorTolerance" => pub character_lod_error_tolerance: f32,
        "LODErrorFactor" => pub lod_error_factor: f32,
        "SeaHeight" => pub sea_height: f32,
        "LocalDetailBooleanAlphaDefaultAlphaRef" => pub local_detail_boolean_alpha_default_alpha_ref: i32,
        "DefaultPrimitiveAlphaRef" => pub default_primitive_alpha_ref: i32,
        "GamePrimitiveDefaultFadeStart" => pub game_primitive_default_fade_start: f32,
        "GamePrimitiveDefaultFadeRangeRatio" => pub game_primitive_default_fade_range_ratio: f32,
        "LocalDetailDefaultFadeStart" => pub local_detail_default_fade_start: f32,
        "LocalDetailDefaultFadeRangeRatio" => pub local_detail_default_fade_range_ratio: f32,
        "TestStaticMesh" => pub test_static_mesh: i32,
        "TestAnimatedMesh" => pub test_animated_mesh: i32,
        "TestAnim" => pub test_anim: i32,
        "TestGraphic" => pub test_graphic: i32,
        "FOV_2D" => pub fov_2_d: f32,
        "InvalidTextureStandin" => pub invalid_texture_standin: i32,
        "InvalidThemeStandin" => pub invalid_theme_standin: i32,
    }
}

