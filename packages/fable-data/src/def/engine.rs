use super::binary::control::{ID_BYTE_SIZE, ParseControlError, SerializeControlError, parse_scalar, serialize_scalar};

#[derive(Debug, PartialEq, Default)]
pub struct EngineDef {
    pub lod_error_tolerance: f32,
    pub character_lod_error_tolerance: f32,
    pub lod_error_factor: f32,
    pub sea_height: f32,
    pub local_detail_boolean_alpha_default_alpha_ref: i32,
    pub default_primitive_alpha_ref: i32,
    pub game_primitive_default_fade_start: f32,
    pub game_primitive_default_fade_range_ratio: f32,
    pub local_detail_default_fade_start: f32,
    pub local_detail_default_fade_range_ratio: f32,
    pub test_static_mesh: i32,
    pub test_animated_mesh: i32,
    pub test_anim: i32,
    pub test_graphic: i32,
    pub fov_2d: f32,
    pub invalid_texture_standin: i32,
    pub invalid_theme_standin: i32,
}

impl EngineDef {
    pub(crate) const BYTE_SIZE: usize = ID_BYTE_SIZE * 17 + size_of::<EngineDef>();

    pub(crate) const fn byte_size(&self) -> usize {
        Self::BYTE_SIZE
    }

    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            lod_error_tolerance: parse_scalar::<f32>(cur, "LODErrorTolerance")?,
            character_lod_error_tolerance: parse_scalar::<f32>(cur, "CharacterLODErrorTolerance")?,
            lod_error_factor: parse_scalar::<f32>(cur, "LODErrorFactor")?,
            sea_height: parse_scalar::<f32>(cur, "SeaHeight")?,
            local_detail_boolean_alpha_default_alpha_ref: parse_scalar::<i32>(cur, "LocalDetailBooleanAlphaDefaultAlphaRef")?,
            default_primitive_alpha_ref: parse_scalar::<i32>(cur, "DefaultPrimitiveAlphaRef")?,
            game_primitive_default_fade_start: parse_scalar::<f32>(cur, "GamePrimitiveDefaultFadeStart")?,
            game_primitive_default_fade_range_ratio: parse_scalar::<f32>(cur, "GamePrimitiveDefaultFadeRangeRatio")?,
            local_detail_default_fade_start: parse_scalar::<f32>(cur, "LocalDetailDefaultFadeStart")?,
            local_detail_default_fade_range_ratio: parse_scalar::<f32>(cur, "LocalDetailDefaultFadeRangeRatio")?,
            test_static_mesh: parse_scalar::<i32>(cur, "TestStaticMesh")?,
            test_animated_mesh: parse_scalar::<i32>(cur, "TestAnimatedMesh")?,
            test_anim: parse_scalar::<i32>(cur, "TestAnim")?,
            test_graphic: parse_scalar::<i32>(cur, "TestGraphic")?,
            fov_2d: parse_scalar::<f32>(cur, "FOV_2D")?,
            invalid_texture_standin: parse_scalar::<i32>(cur, "InvalidTextureStandin")?,
            invalid_theme_standin: parse_scalar::<i32>(cur, "InvalidThemeStandin")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<f32>(out, "LODErrorTolerance", self.lod_error_tolerance)?;
        serialize_scalar::<f32>(out, "CharacterLODErrorTolerance", self.character_lod_error_tolerance)?;
        serialize_scalar::<f32>(out, "LODErrorFactor", self.lod_error_factor)?;
        serialize_scalar::<f32>(out, "SeaHeight", self.sea_height)?;
        serialize_scalar::<i32>(out, "LocalDetailBooleanAlphaDefaultAlphaRef", self.local_detail_boolean_alpha_default_alpha_ref)?;
        serialize_scalar::<i32>(out, "DefaultPrimitiveAlphaRef", self.default_primitive_alpha_ref)?;
        serialize_scalar::<f32>(out, "GamePrimitiveDefaultFadeStart", self.game_primitive_default_fade_start)?;
        serialize_scalar::<f32>(out, "GamePrimitiveDefaultFadeRangeRatio", self.game_primitive_default_fade_range_ratio)?;
        serialize_scalar::<f32>(out, "LocalDetailDefaultFadeStart", self.local_detail_default_fade_start)?;
        serialize_scalar::<f32>(out, "LocalDetailDefaultFadeRangeRatio", self.local_detail_default_fade_range_ratio)?;
        serialize_scalar::<i32>(out, "TestStaticMesh", self.test_static_mesh)?;
        serialize_scalar::<i32>(out, "TestAnimatedMesh", self.test_animated_mesh)?;
        serialize_scalar::<i32>(out, "TestAnim", self.test_anim)?;
        serialize_scalar::<i32>(out, "TestGraphic", self.test_graphic)?;
        serialize_scalar::<f32>(out, "FOV_2D", self.fov_2d)?;
        serialize_scalar::<i32>(out, "InvalidTextureStandin", self.invalid_texture_standin)?;
        serialize_scalar::<i32>(out, "InvalidThemeStandin", self.invalid_theme_standin)?;
        Ok(())
    }

}
