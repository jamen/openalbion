use crate::def_struct;

def_struct! {
    /// `CONFIG_OPTIONS_DEFAULTS_DEF` — C++ `CConfigOptionsDefaultsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ConfigOptionsDefaultsDef {
        "Antialiasing" => pub antialiasing: u32,
        "ResolutionWidth" => pub resolution_width: i32 = 1024,
        "ResolutionHeight" => pub resolution_height: i32 = 768,
        "BitDepth" => pub bit_depth: i32 = 16,
        "TextureDetail" => pub texture_detail: f32 = 1.0,
        "MaxTextureDetail" => pub max_texture_detail: f32 = 3.0,
        "ShadowDetail" => pub shadow_detail: f32 = 1.0,
        "MaxShadowDetail" => pub max_shadow_detail: f32 = 3.0,
        "MeshDetail" => pub mesh_detail: f32 = 1.0,
        "MaxMeshDetail" => pub max_mesh_detail: f32 = 3.0,
        "EffectsDetail" => pub effects_detail: f32 = 1.0,
        "MaxEffectsDetail" => pub max_effects_detail: f32 = 3.0,
        "MinResolutionWidth" => pub min_resolution_width: i32 = 1024,
        "MinResolutionHeight" => pub min_resolution_height: i32 = 768,
    }
}

