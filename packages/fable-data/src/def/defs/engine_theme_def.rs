use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `ENGINE_THEME` — C++ `CEngineThemeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct EngineThemeDef {
        "BaseTexture" => pub base_texture: i32,
        "BackgroundTexture" => pub background_texture: i32,
        "CliffBaseTexture" => pub cliff_base_texture: i32,
        "CliffBackgroundTexture" => pub cliff_background_texture: i32,
        "BaseBumpMap" => pub base_bump_map: i32,
        "CliffBumpMap" => pub cliff_bump_map: i32,
        "BaseTextureMaxSize" => pub base_texture_max_size: i32,
        "CliffTextureMaxSize" => pub cliff_texture_max_size: i32,
        "BaseBumpMapMaxSize" => pub base_bump_map_max_size: i32,
        "CliffBumpMapMaxSize" => pub cliff_bump_map_max_size: i32,
        "BaseTextureSelfIllumination" => pub base_texture_self_illumination: f32,
        "CliffTextureSelfIllumination" => pub cliff_texture_self_illumination: f32,
        "GroupDef" => pub group_def: i32,
        "Height" => pub height: f32,
        "MinCameraHeight" => pub min_camera_height: f32,
        "CoverValue" => pub cover_value: f32,
        "MaterialDef" => pub material_def: i32,
        "MinimapTheme" => pub minimap_theme: MinimapThemeType,
        "LocalDetailGeneratorDef" => pub local_detail_generator_def: i32,
        "DestructionEffectDef" => pub destruction_effect_def: i32,
        "DestructionThemeDef" => pub destruction_theme_def: i32,
        "Friction" => pub friction: f32 = 1.0,
        "WaterHeight" => pub water_height: f32,
        "WaterType" => pub water_type: WaterType,
        "NoWaterThemeDef" => pub no_water_theme_def: i32,
        "Passable" => pub passable: bool = true,
    }
}
