use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CONFIG_OPTIONS_DEFAULTS_DEF` — C++ `CConfigOptionsDefaultsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConfigOptionsDefaultsDef {
        "Antialiasing" => pub antialiasing: u32,
        "ResolutionWidth" => pub resolution_width: i32,
        "ResolutionHeight" => pub resolution_height: i32,
        "BitDepth" => pub bit_depth: i32,
        "TextureDetail" => pub texture_detail: f32,
        "MaxTextureDetail" => pub max_texture_detail: f32,
        "ShadowDetail" => pub shadow_detail: f32,
        "MaxShadowDetail" => pub max_shadow_detail: f32,
        "MeshDetail" => pub mesh_detail: f32,
        "MaxMeshDetail" => pub max_mesh_detail: f32,
        "EffectsDetail" => pub effects_detail: f32,
        "MaxEffectsDetail" => pub max_effects_detail: f32,
        "MinResolutionWidth" => pub min_resolution_width: i32,
        "MinResolutionHeight" => pub min_resolution_height: i32,
    }
}

