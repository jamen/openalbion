use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, SerializeControlError, parse_scalar, parse_string,
    serialize_scalar, serialize_string, string_control_byte_size,
};

/// Binary definition for an environment (from game.bin / CompiledDefs).
///
/// Field order verified against retail `game.bin`. Based on `CEnvironmentDef` —
/// controls lighting lookup rows, water, sky, clouds, etc.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentDef {
    pub colour_lookup_texture: String,
    pub diffuse_lookup_row: i32,
    pub ambient_lookup_row: i32,
    pub backlight_lookup_row: i32,
    pub reflection_lookup_row: i32,
    pub mist_effect_colour_lookup_row: i32,
    pub fog_colour_lookup_row: i32,
    pub fog_alpha_lookup_row: i32,
    pub sun_colour_lookup_row: i32,
    pub cloud_colour_lookup_row: i32,
    pub moon_colour_lookup_row: i32,
    pub stars_colour_lookup_row: i32,
    pub sun_flare_colour_lookup_row: i32,
    pub lens_flare_colour_lookup_row: i32,
    pub sky_gradient_top_lookup_row: i32,
    pub sky_gradient_top_alpha_lookup_row: i32,
    pub sky_gradient_bottom_lookup_row: i32,
    pub sky_gradient_bottom_alpha_lookup_row: i32,
    pub sunlight_attenuator_colour_lookup_row: i32,
    pub diffuse_clamp_angle: f32,
    pub sunlight_attenuator_angle_fade_start: f32,
    pub sunlight_attenuator_angle_fade_end: f32,
    pub water_colour_lookup_row: i32,
    pub sea_colour_lookup_row: i32,
    pub glow_threshold_colour_lookup_row: i32,
    pub glow_bloom_colour_lookup_row: i32,
    pub sea_texture: i32,
    pub sea_radius: i32,
    pub sea_flat_section_start: i32,
    pub sea_flat_section_end: i32,
    pub day_start_time: f32,
    pub day_speed: f32,
    pub mist_alpha_graphic: i32,
    pub mist_alpha_graphic_pc: i32,
    pub ice_bump_map: i32,
    pub ice_bump_map_pc: i32,
    pub ice_texture: i32,
    pub water_edge_alpha_map: i32,
    pub water_surf_map: i32,
    pub water_bump_map_pc: i32,
    pub water_bump_map: i32,
    pub water_bump_map_2: i32,
    pub sea_bump_map_pc: i32,
    pub sea_bump_map: i32,
    pub sea_bump_map_2: i32,
    pub water_env_map_overlay_texture: i32,
    pub rain_texture: i32,
    pub rain_texture_pc: i32,
    pub snow_texture: i32,
    pub rain_splash_particle: i32,
    pub lightning_fade_in_duration: f32,
    pub lightning_fade_out_duration: f32,
    pub lightning_flash_duration: f32,
    pub lightning_rain_threshold: f32,
    pub lightning_theme: i32,
    pub cloud_speed_multiplier: f32,
    pub cloud_max_speed: f32,
    pub cloud_texture_coord_multiplier: f32,
    pub cloud_texture_coord_offset: f32,
    pub cloud_height_offset: f32,
    pub water_lake_minimum_flow_speed: f32,
    pub water_lake_maximum_flow_speed: f32,
}

impl EnvironmentDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            colour_lookup_texture: parse_string(cur, "ColourLookupTexture")?.to_string(),
            diffuse_lookup_row: parse_scalar::<i32>(cur, "DiffuseLookupRow")?,
            ambient_lookup_row: parse_scalar::<i32>(cur, "AmbientLookupRow")?,
            backlight_lookup_row: parse_scalar::<i32>(cur, "BacklightLookupRow")?,
            reflection_lookup_row: parse_scalar::<i32>(cur, "ReflectionLookupRow")?,
            mist_effect_colour_lookup_row: parse_scalar::<i32>(cur, "MistEffectColourLookupRow")?,
            fog_colour_lookup_row: parse_scalar::<i32>(cur, "FogColourLookupRow")?,
            fog_alpha_lookup_row: parse_scalar::<i32>(cur, "FogAlphaLookupRow")?,
            sun_colour_lookup_row: parse_scalar::<i32>(cur, "SunColourLookupRow")?,
            cloud_colour_lookup_row: parse_scalar::<i32>(cur, "CloudColourLookupRow")?,
            moon_colour_lookup_row: parse_scalar::<i32>(cur, "MoonColourLookupRow")?,
            stars_colour_lookup_row: parse_scalar::<i32>(cur, "StarsColourLookupRow")?,
            sun_flare_colour_lookup_row: parse_scalar::<i32>(cur, "SunFlareColourLookupRow")?,
            lens_flare_colour_lookup_row: parse_scalar::<i32>(cur, "LensFlareColourLookupRow")?,
            sky_gradient_top_lookup_row: parse_scalar::<i32>(cur, "SkyGradientTopLookupRow")?,
            sky_gradient_top_alpha_lookup_row: parse_scalar::<i32>(cur, "SkyGradientTopAlphaLookupRow")?,
            sky_gradient_bottom_lookup_row: parse_scalar::<i32>(cur, "SkyGradientBottomLookupRow")?,
            sky_gradient_bottom_alpha_lookup_row: parse_scalar::<i32>(cur, "SkyGradientBottomAlphaLookupRow")?,
            sunlight_attenuator_colour_lookup_row: parse_scalar::<i32>(cur, "SunlightAttenuatorColourLookupRow")?,
            diffuse_clamp_angle: parse_scalar::<f32>(cur, "DiffuseClampAngle")?,
            sunlight_attenuator_angle_fade_start: parse_scalar::<f32>(cur, "SunlightAttenuatorAngleFadeStart")?,
            sunlight_attenuator_angle_fade_end: parse_scalar::<f32>(cur, "SunlightAttenuatorAngleFadeEnd")?,
            water_colour_lookup_row: parse_scalar::<i32>(cur, "WaterColourLookupRow")?,
            sea_colour_lookup_row: parse_scalar::<i32>(cur, "SeaColourLookupRow")?,
            glow_threshold_colour_lookup_row: parse_scalar::<i32>(cur, "GlowThresholdColourLookupRow")?,
            glow_bloom_colour_lookup_row: parse_scalar::<i32>(cur, "GlowBloomColourLookupRow")?,
            sea_texture: parse_scalar::<i32>(cur, "SeaTexture")?,
            sea_radius: parse_scalar::<i32>(cur, "SeaRadius")?,
            sea_flat_section_start: parse_scalar::<i32>(cur, "SeaFlatSectionStart")?,
            sea_flat_section_end: parse_scalar::<i32>(cur, "SeaFlatSectionEnd")?,
            day_start_time: parse_scalar::<f32>(cur, "DayStartTime")?,
            day_speed: parse_scalar::<f32>(cur, "DaySpeed")?,
            mist_alpha_graphic: parse_scalar::<i32>(cur, "MistAlphaGraphic")?,
            mist_alpha_graphic_pc: parse_scalar::<i32>(cur, "MistAlphaGraphicPC")?,
            ice_bump_map: parse_scalar::<i32>(cur, "IceBumpMap")?,
            ice_bump_map_pc: parse_scalar::<i32>(cur, "IceBumpMapPC")?,
            ice_texture: parse_scalar::<i32>(cur, "IceTexture")?,
            water_edge_alpha_map: parse_scalar::<i32>(cur, "WaterEdgeAlphaMap")?,
            water_surf_map: parse_scalar::<i32>(cur, "WaterSurfMap")?,
            water_bump_map_pc: parse_scalar::<i32>(cur, "WaterBumpMapPC")?,
            water_bump_map: parse_scalar::<i32>(cur, "WaterBumpMap")?,
            water_bump_map_2: parse_scalar::<i32>(cur, "WaterBumpMap2")?,
            sea_bump_map_pc: parse_scalar::<i32>(cur, "SeaBumpMapPC")?,
            sea_bump_map: parse_scalar::<i32>(cur, "SeaBumpMap")?,
            sea_bump_map_2: parse_scalar::<i32>(cur, "SeaBumpMap2")?,
            water_env_map_overlay_texture: parse_scalar::<i32>(cur, "WaterEnvMapOverlayTexture")?,
            rain_texture: parse_scalar::<i32>(cur, "RainTexture")?,
            rain_texture_pc: parse_scalar::<i32>(cur, "RainTexturePC")?,
            snow_texture: parse_scalar::<i32>(cur, "SnowTexture")?,
            rain_splash_particle: parse_scalar::<i32>(cur, "RainSplashParticle")?,
            lightning_fade_in_duration: parse_scalar::<f32>(cur, "LightningFadeInDuration")?,
            lightning_fade_out_duration: parse_scalar::<f32>(cur, "LightningFadeOutDuration")?,
            lightning_flash_duration: parse_scalar::<f32>(cur, "LightningFlashDuration")?,
            lightning_rain_threshold: parse_scalar::<f32>(cur, "LightningRainThreshold")?,
            lightning_theme: parse_scalar::<i32>(cur, "LightningTheme")?,
            cloud_speed_multiplier: parse_scalar::<f32>(cur, "CloudSpeedMultiplier")?,
            cloud_max_speed: parse_scalar::<f32>(cur, "CloudMaxSpeed")?,
            cloud_texture_coord_multiplier: parse_scalar::<f32>(cur, "CloudTextureCoordMultiplier")?,
            cloud_texture_coord_offset: parse_scalar::<f32>(cur, "CloudTextureCoordOffset")?,
            cloud_height_offset: parse_scalar::<f32>(cur, "CloudHeightOffset")?,
            water_lake_minimum_flow_speed: parse_scalar::<f32>(cur, "WaterLakeMinimumFlowSpeed")?,
            water_lake_maximum_flow_speed: parse_scalar::<f32>(cur, "WaterLakeMaximumFlowSpeed")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_string(out, "ColourLookupTexture", &self.colour_lookup_texture)?;
        serialize_scalar::<i32>(out, "DiffuseLookupRow", self.diffuse_lookup_row)?;
        serialize_scalar::<i32>(out, "AmbientLookupRow", self.ambient_lookup_row)?;
        serialize_scalar::<i32>(out, "BacklightLookupRow", self.backlight_lookup_row)?;
        serialize_scalar::<i32>(out, "ReflectionLookupRow", self.reflection_lookup_row)?;
        serialize_scalar::<i32>(out, "MistEffectColourLookupRow", self.mist_effect_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "FogColourLookupRow", self.fog_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "FogAlphaLookupRow", self.fog_alpha_lookup_row)?;
        serialize_scalar::<i32>(out, "SunColourLookupRow", self.sun_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "CloudColourLookupRow", self.cloud_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "MoonColourLookupRow", self.moon_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "StarsColourLookupRow", self.stars_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "SunFlareColourLookupRow", self.sun_flare_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "LensFlareColourLookupRow", self.lens_flare_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "SkyGradientTopLookupRow", self.sky_gradient_top_lookup_row)?;
        serialize_scalar::<i32>(out, "SkyGradientTopAlphaLookupRow", self.sky_gradient_top_alpha_lookup_row)?;
        serialize_scalar::<i32>(out, "SkyGradientBottomLookupRow", self.sky_gradient_bottom_lookup_row)?;
        serialize_scalar::<i32>(out, "SkyGradientBottomAlphaLookupRow", self.sky_gradient_bottom_alpha_lookup_row)?;
        serialize_scalar::<i32>(out, "SunlightAttenuatorColourLookupRow", self.sunlight_attenuator_colour_lookup_row)?;
        serialize_scalar::<f32>(out, "DiffuseClampAngle", self.diffuse_clamp_angle)?;
        serialize_scalar::<f32>(out, "SunlightAttenuatorAngleFadeStart", self.sunlight_attenuator_angle_fade_start)?;
        serialize_scalar::<f32>(out, "SunlightAttenuatorAngleFadeEnd", self.sunlight_attenuator_angle_fade_end)?;
        serialize_scalar::<i32>(out, "WaterColourLookupRow", self.water_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "SeaColourLookupRow", self.sea_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "GlowThresholdColourLookupRow", self.glow_threshold_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "GlowBloomColourLookupRow", self.glow_bloom_colour_lookup_row)?;
        serialize_scalar::<i32>(out, "SeaTexture", self.sea_texture)?;
        serialize_scalar::<i32>(out, "SeaRadius", self.sea_radius)?;
        serialize_scalar::<i32>(out, "SeaFlatSectionStart", self.sea_flat_section_start)?;
        serialize_scalar::<i32>(out, "SeaFlatSectionEnd", self.sea_flat_section_end)?;
        serialize_scalar::<f32>(out, "DayStartTime", self.day_start_time)?;
        serialize_scalar::<f32>(out, "DaySpeed", self.day_speed)?;
        serialize_scalar::<i32>(out, "MistAlphaGraphic", self.mist_alpha_graphic)?;
        serialize_scalar::<i32>(out, "MistAlphaGraphicPC", self.mist_alpha_graphic_pc)?;
        serialize_scalar::<i32>(out, "IceBumpMap", self.ice_bump_map)?;
        serialize_scalar::<i32>(out, "IceBumpMapPC", self.ice_bump_map_pc)?;
        serialize_scalar::<i32>(out, "IceTexture", self.ice_texture)?;
        serialize_scalar::<i32>(out, "WaterEdgeAlphaMap", self.water_edge_alpha_map)?;
        serialize_scalar::<i32>(out, "WaterSurfMap", self.water_surf_map)?;
        serialize_scalar::<i32>(out, "WaterBumpMapPC", self.water_bump_map_pc)?;
        serialize_scalar::<i32>(out, "WaterBumpMap", self.water_bump_map)?;
        serialize_scalar::<i32>(out, "WaterBumpMap2", self.water_bump_map_2)?;
        serialize_scalar::<i32>(out, "SeaBumpMapPC", self.sea_bump_map_pc)?;
        serialize_scalar::<i32>(out, "SeaBumpMap", self.sea_bump_map)?;
        serialize_scalar::<i32>(out, "SeaBumpMap2", self.sea_bump_map_2)?;
        serialize_scalar::<i32>(out, "WaterEnvMapOverlayTexture", self.water_env_map_overlay_texture)?;
        serialize_scalar::<i32>(out, "RainTexture", self.rain_texture)?;
        serialize_scalar::<i32>(out, "RainTexturePC", self.rain_texture_pc)?;
        serialize_scalar::<i32>(out, "SnowTexture", self.snow_texture)?;
        serialize_scalar::<i32>(out, "RainSplashParticle", self.rain_splash_particle)?;
        serialize_scalar::<f32>(out, "LightningFadeInDuration", self.lightning_fade_in_duration)?;
        serialize_scalar::<f32>(out, "LightningFadeOutDuration", self.lightning_fade_out_duration)?;
        serialize_scalar::<f32>(out, "LightningFlashDuration", self.lightning_flash_duration)?;
        serialize_scalar::<f32>(out, "LightningRainThreshold", self.lightning_rain_threshold)?;
        serialize_scalar::<i32>(out, "LightningTheme", self.lightning_theme)?;
        serialize_scalar::<f32>(out, "CloudSpeedMultiplier", self.cloud_speed_multiplier)?;
        serialize_scalar::<f32>(out, "CloudMaxSpeed", self.cloud_max_speed)?;
        serialize_scalar::<f32>(out, "CloudTextureCoordMultiplier", self.cloud_texture_coord_multiplier)?;
        serialize_scalar::<f32>(out, "CloudTextureCoordOffset", self.cloud_texture_coord_offset)?;
        serialize_scalar::<f32>(out, "CloudHeightOffset", self.cloud_height_offset)?;
        serialize_scalar::<f32>(out, "WaterLakeMinimumFlowSpeed", self.water_lake_minimum_flow_speed)?;
        serialize_scalar::<f32>(out, "WaterLakeMaximumFlowSpeed", self.water_lake_maximum_flow_speed)?;
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        string_control_byte_size(&self.colour_lookup_texture)
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<i32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
            + (ID_BYTE_SIZE + size_of::<f32>())
    }
}
