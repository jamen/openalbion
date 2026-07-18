use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTattooDef` — C++ `CTattooDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TattooDef {
        "TattooName" => pub tattoo_name: String,
        "TextureLayer" => pub texture_layer: i32,
        "ReplacingTextureIndex" => pub replacing_texture_index: i32,
        "BankIndex" => pub bank_index: i32,
        "Blend" => pub blend: CompositeBlendType = CompositeBlendType::ALPHA,
        "RemovesAllTattoos" => pub removes_all_tattoos: bool,
        "Permanent" => pub permanent: bool,
        "CoversBodyAreaFlags" => pub covers_body_area_flags: i32,
        "Scariness" => pub scariness: f32,
        "Attractiveness" => pub attractiveness: f32,
        "Goodstrength" => pub goodstrength: f32,
        "VisibilityMultiplier" => pub visibility_multiplier: f32 = 1.0,
        "SpecificCoversBodyAreaFlags" => pub specific_covers_body_area_flags: i32,
        "CustomFileName" => pub custom_file_name: String,
        "CustomFileNameBase" => pub custom_file_name_base: String,
    }
}
