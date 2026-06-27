use super::binary::control::{ID_BYTE_SIZE, ParseControlError, SerializeControlError, parse_bool, parse_scalar, serialize_bool, serialize_scalar};

#[derive(Debug)]
pub struct EngineVideoOptionsDef {
    pub hires_texture_memory: i32,
    pub lod_error_tolerance: f32,
    pub character_lod_error_tolerance: f32,
    pub draw_distance_multiplier: f32,
    pub draw_distance_minimum: f32,
    pub draw_distance_maximum: f32,
    pub repeated_mesh_draw_distance_factor: f32,
    pub minimum_z_sprite_as_mesh_distance: f32,
    pub maximum_z_sprite_as_mesh_distance: f32,
    pub z_sprite_draw_distance_multiplier: f32,
    pub shadow_buffer_size: i32,
    pub shadow_distance_scale: f32,
    pub enable_2d_displacement: bool,
    pub enable_3d_displacement: bool,
    pub enable_glow: bool,
    pub enable_radial_blur: bool,
    pub enable_water_reflection: bool,
    pub enable_weather_effects: bool,
    pub enable_colour_filter: bool,
    pub weather_density: f32,
    pub enable_repeated_meshes: bool,
}

impl EngineVideoOptionsDef {
    pub(crate) const BYTE_SIZE: usize = (ID_BYTE_SIZE * 21)
        + (size_of::<f32>() * 11)
        + (size_of::<i32>() * 2)
        + (size_of::<bool>() * 8);

    pub(crate) const fn byte_size(&self) -> usize {
        Self::BYTE_SIZE
    }

    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            hires_texture_memory: parse_scalar::<i32>(cur, "HiresTextureMemory")?,
            lod_error_tolerance: parse_scalar::<f32>(cur, "LODErrorTolerance")?,
            character_lod_error_tolerance: parse_scalar::<f32>(cur, "CharacterLODErrorTolerance")?,
            draw_distance_multiplier: parse_scalar::<f32>(cur, "DrawDistanceMultiplier")?,
            draw_distance_minimum: parse_scalar::<f32>(cur, "DrawDistanceMinimum")?,
            draw_distance_maximum: parse_scalar::<f32>(cur, "DrawDistanceMaximum")?,
            repeated_mesh_draw_distance_factor: parse_scalar::<f32>(cur, "RepeatedMeshDrawDistanceFactor")?,
            minimum_z_sprite_as_mesh_distance: parse_scalar::<f32>(cur, "MinimumZSpriteAsMeshDistance")?,
            maximum_z_sprite_as_mesh_distance: parse_scalar::<f32>(cur, "MaximumZSpriteAsMeshDistance")?,
            z_sprite_draw_distance_multiplier: parse_scalar::<f32>(cur, "ZSpriteDrawDistanceMultiplier")?,
            shadow_buffer_size: parse_scalar::<i32>(cur, "ShadowBufferSize")?,
            shadow_distance_scale: parse_scalar::<f32>(cur, "ShadowDistanceScale")?,
            enable_2d_displacement: parse_bool(cur, "Enable2DDisplacement")?,
            enable_3d_displacement: parse_bool(cur, "Enable3DDisplacement")?,
            enable_glow: parse_bool(cur, "EnableGlow")?,
            enable_radial_blur: parse_bool(cur, "EnableRadialBlur")?,
            enable_water_reflection: parse_bool(cur, "EnableWaterReflection")?,
            enable_weather_effects: parse_bool(cur, "EnableWeatherEffects")?,
            enable_colour_filter: parse_bool(cur, "EnableColourFilter")?,
            weather_density: parse_scalar::<f32>(cur, "WeatherDensity")?,
            enable_repeated_meshes: parse_bool(cur, "EnableRepeatedMeshes")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<i32>(out, "HiresTextureMemory", self.hires_texture_memory)?;
        serialize_scalar::<f32>(out, "LODErrorTolerance", self.lod_error_tolerance)?;
        serialize_scalar::<f32>(out, "CharacterLODErrorTolerance", self.character_lod_error_tolerance)?;
        serialize_scalar::<f32>(out, "DrawDistanceMultiplier", self.draw_distance_multiplier)?;
        serialize_scalar::<f32>(out, "DrawDistanceMinimum", self.draw_distance_minimum)?;
        serialize_scalar::<f32>(out, "DrawDistanceMaximum", self.draw_distance_maximum)?;
        serialize_scalar::<f32>(out, "RepeatedMeshDrawDistanceFactor", self.repeated_mesh_draw_distance_factor)?;
        serialize_scalar::<f32>(out, "MinimumZSpriteAsMeshDistance", self.minimum_z_sprite_as_mesh_distance)?;
        serialize_scalar::<f32>(out, "MaximumZSpriteAsMeshDistance", self.maximum_z_sprite_as_mesh_distance)?;
        serialize_scalar::<f32>(out, "ZSpriteDrawDistanceMultiplier", self.z_sprite_draw_distance_multiplier)?;
        serialize_scalar::<i32>(out, "ShadowBufferSize", self.shadow_buffer_size)?;
        serialize_scalar::<f32>(out, "ShadowDistanceScale", self.shadow_distance_scale)?;
        serialize_bool(out, "Enable2DDisplacement", self.enable_2d_displacement)?;
        serialize_bool(out, "Enable3DDisplacement", self.enable_3d_displacement)?;
        serialize_bool(out, "EnableGlow", self.enable_glow)?;
        serialize_bool(out, "EnableRadialBlur", self.enable_radial_blur)?;
        serialize_bool(out, "EnableWaterReflection", self.enable_water_reflection)?;
        serialize_bool(out, "EnableWeatherEffects", self.enable_weather_effects)?;
        serialize_bool(out, "EnableColourFilter", self.enable_colour_filter)?;
        serialize_scalar::<f32>(out, "WeatherDensity", self.weather_density)?;
        serialize_bool(out, "EnableRepeatedMeshes", self.enable_repeated_meshes)?;
        Ok(())
    }

}
