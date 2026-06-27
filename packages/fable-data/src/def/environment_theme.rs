use super::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason, parse_bool, parse_id, parse_scalar, serialize_bool, serialize_id,
    serialize_scalar,
};
use crate::bytes::{put_le, take_le};

/// One time-of-day keyframe within an [`EnvironmentThemeDaySetDef`] (from game.bin).
///
/// Based on `CEnvironmentThemeDef`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentThemeDef {
    pub time_of_day: f32,
    pub moon_lit: bool,
    pub fog_start_z: f32,
    pub fog_end_z: f32,
    pub sky_texture_0: i32,
    pub sky_texture_1: i32,
    pub sky_texture_1_blend: f32,
    pub cloud_lower_layer_texture_0: i32,
    pub cloud_lower_layer_texture_1: i32,
    pub cloud_lower_layer_texture_1_blend: f32,
    pub cloud_lower_layer_speed_multiplier: f32,
    pub cloud_upper_layer_texture_0: i32,
    pub cloud_upper_layer_texture_1: i32,
    pub cloud_upper_layer_texture_1_blend: f32,
    pub cloud_upper_layer_speed_multiplier: f32,
    pub water_colour_to_reflection_blend: f32,
    pub water_alpha_factor: f32,
    pub water_specular_highlight_factor: f32,
    pub rain_strength: f32,
    pub snow_strength: f32,
    pub mist_max_alpha: f32,
    pub lightning_frequency: f32,
    pub glow_threshold_scale: f32,
    pub glow_bloom_scale: f32,
    pub glow_motion_blur: f32,
    pub shadow_factor: f32,
    pub faded_shadow_factor: f32,
    pub water_refraction_blend_start: f32,
    pub water_refraction_blend_end: f32,
    pub water_flow_speed_factor: f32,
    pub water_oscillation_speed: f32,
    pub water_swell_factor: f32,
    pub water_shore_swell_factor: f32,
    pub water_reflection_offset: f32,
}

impl EnvironmentThemeDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            time_of_day: parse_scalar::<f32>(cur, "TimeOfDay")?,
            moon_lit: parse_bool(cur, "MoonLit")?,
            fog_start_z: parse_scalar::<f32>(cur, "FogStartZ")?,
            fog_end_z: parse_scalar::<f32>(cur, "FogEndZ")?,
            sky_texture_0: parse_scalar::<i32>(cur, "SkyTexture0")?,
            sky_texture_1: parse_scalar::<i32>(cur, "SkyTexture1")?,
            sky_texture_1_blend: parse_scalar::<f32>(cur, "SkyTexture1Blend")?,
            cloud_lower_layer_texture_0: parse_scalar::<i32>(cur, "CloudLowerLayerTexture0")?,
            cloud_lower_layer_texture_1: parse_scalar::<i32>(cur, "CloudLowerLayerTexture1")?,
            cloud_lower_layer_texture_1_blend: parse_scalar::<f32>(cur, "CloudLowerLayerTexture1Blend")?,
            cloud_lower_layer_speed_multiplier: parse_scalar::<f32>(cur, "CloudLowerLayerSpeedMultiplier")?,
            cloud_upper_layer_texture_0: parse_scalar::<i32>(cur, "CloudUpperLayerTexture0")?,
            cloud_upper_layer_texture_1: parse_scalar::<i32>(cur, "CloudUpperLayerTexture1")?,
            cloud_upper_layer_texture_1_blend: parse_scalar::<f32>(cur, "CloudUpperLayerTexture1Blend")?,
            cloud_upper_layer_speed_multiplier: parse_scalar::<f32>(cur, "CloudUpperLayerSpeedMultiplier")?,
            water_colour_to_reflection_blend: parse_scalar::<f32>(cur, "WaterColourToReflectionBlend")?,
            water_alpha_factor: parse_scalar::<f32>(cur, "WaterAlphaFactor")?,
            water_specular_highlight_factor: parse_scalar::<f32>(cur, "WaterSpecularHightlightFactor")?,
            rain_strength: parse_scalar::<f32>(cur, "RainStrength")?,
            snow_strength: parse_scalar::<f32>(cur, "SnowStrength")?,
            mist_max_alpha: parse_scalar::<f32>(cur, "MistAlpha")?,
            lightning_frequency: parse_scalar::<f32>(cur, "LightningFrequency")?,
            glow_threshold_scale: parse_scalar::<f32>(cur, "GlowThresholdScale")?,
            glow_bloom_scale: parse_scalar::<f32>(cur, "GlowBloomScale")?,
            glow_motion_blur: parse_scalar::<f32>(cur, "GlowMotionBlur")?,
            shadow_factor: parse_scalar::<f32>(cur, "ShadowFactor")?,
            faded_shadow_factor: parse_scalar::<f32>(cur, "FadedShadowFactor")?,
            water_refraction_blend_start: parse_scalar::<f32>(cur, "WaterRefractionBlendStart")?,
            water_refraction_blend_end: parse_scalar::<f32>(cur, "WaterRefractionBlendEnd")?,
            water_flow_speed_factor: parse_scalar::<f32>(cur, "WaterFlowSpeedFactor")?,
            water_oscillation_speed: parse_scalar::<f32>(cur, "WaterOscilationSpeed")?,
            water_swell_factor: parse_scalar::<f32>(cur, "WaterSwellFactor")?,
            water_shore_swell_factor: parse_scalar::<f32>(cur, "WaterShoreSwellFactor")?,
            water_reflection_offset: parse_scalar::<f32>(cur, "WaterReflectionOffset")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<f32>(out, "TimeOfDay", self.time_of_day)?;
        serialize_bool(out, "MoonLit", self.moon_lit)?;
        serialize_scalar::<f32>(out, "FogStartZ", self.fog_start_z)?;
        serialize_scalar::<f32>(out, "FogEndZ", self.fog_end_z)?;
        serialize_scalar::<i32>(out, "SkyTexture0", self.sky_texture_0)?;
        serialize_scalar::<i32>(out, "SkyTexture1", self.sky_texture_1)?;
        serialize_scalar::<f32>(out, "SkyTexture1Blend", self.sky_texture_1_blend)?;
        serialize_scalar::<i32>(out, "CloudLowerLayerTexture0", self.cloud_lower_layer_texture_0)?;
        serialize_scalar::<i32>(out, "CloudLowerLayerTexture1", self.cloud_lower_layer_texture_1)?;
        serialize_scalar::<f32>(out, "CloudLowerLayerTexture1Blend", self.cloud_lower_layer_texture_1_blend)?;
        serialize_scalar::<f32>(out, "CloudLowerLayerSpeedMultiplier", self.cloud_lower_layer_speed_multiplier)?;
        serialize_scalar::<i32>(out, "CloudUpperLayerTexture0", self.cloud_upper_layer_texture_0)?;
        serialize_scalar::<i32>(out, "CloudUpperLayerTexture1", self.cloud_upper_layer_texture_1)?;
        serialize_scalar::<f32>(out, "CloudUpperLayerTexture1Blend", self.cloud_upper_layer_texture_1_blend)?;
        serialize_scalar::<f32>(out, "CloudUpperLayerSpeedMultiplier", self.cloud_upper_layer_speed_multiplier)?;
        serialize_scalar::<f32>(out, "WaterColourToReflectionBlend", self.water_colour_to_reflection_blend)?;
        serialize_scalar::<f32>(out, "WaterAlphaFactor", self.water_alpha_factor)?;
        serialize_scalar::<f32>(out, "WaterSpecularHightlightFactor", self.water_specular_highlight_factor)?;
        serialize_scalar::<f32>(out, "RainStrength", self.rain_strength)?;
        serialize_scalar::<f32>(out, "SnowStrength", self.snow_strength)?;
        serialize_scalar::<f32>(out, "MistAlpha", self.mist_max_alpha)?;
        serialize_scalar::<f32>(out, "LightningFrequency", self.lightning_frequency)?;
        serialize_scalar::<f32>(out, "GlowThresholdScale", self.glow_threshold_scale)?;
        serialize_scalar::<f32>(out, "GlowBloomScale", self.glow_bloom_scale)?;
        serialize_scalar::<f32>(out, "GlowMotionBlur", self.glow_motion_blur)?;
        serialize_scalar::<f32>(out, "ShadowFactor", self.shadow_factor)?;
        serialize_scalar::<f32>(out, "FadedShadowFactor", self.faded_shadow_factor)?;
        serialize_scalar::<f32>(out, "WaterRefractionBlendStart", self.water_refraction_blend_start)?;
        serialize_scalar::<f32>(out, "WaterRefractionBlendEnd", self.water_refraction_blend_end)?;
        serialize_scalar::<f32>(out, "WaterFlowSpeedFactor", self.water_flow_speed_factor)?;
        serialize_scalar::<f32>(out, "WaterOscilationSpeed", self.water_oscillation_speed)?;
        serialize_scalar::<f32>(out, "WaterSwellFactor", self.water_swell_factor)?;
        serialize_scalar::<f32>(out, "WaterShoreSwellFactor", self.water_shore_swell_factor)?;
        serialize_scalar::<f32>(out, "WaterReflectionOffset", self.water_reflection_offset)?;
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        // 6 i32 + 27 f32 (all 4 bytes) + 1 bool (1 byte), each preceded by an id.
        33 * (ID_BYTE_SIZE + size_of::<i32>()) + (ID_BYTE_SIZE + size_of::<u8>())
    }
}

/// A full day's set of environment themes (the `ENVIRONMENT_THEME_DAY` def).
///
/// Based on `CEnvironmentThemeDaySetDef`: a `Time` array of per-time-of-day
/// keyframes plus sun/moon placement and editor metadata.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentThemeDaySetDef {
    pub time: Vec<EnvironmentThemeDef>,
    pub sun_tilt: f32,
    pub sun_rotate: f32,
    pub sun_height: f32,
    pub moon_tilt: f32,
    pub moon_rotate: f32,
    pub moon_height: f32,
    pub colour_lookup_column: i32,
    /// `CRGBColour` packed as RGBA bytes.
    pub editor_colour: u32,
    pub fish_weight_mult: f32,
}

impl EnvironmentThemeDaySetDef {
    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        let time = Self::parse_time_list(cur)?;
        Ok(Self {
            time,
            sun_tilt: parse_scalar::<f32>(cur, "SunTilt")?,
            sun_rotate: parse_scalar::<f32>(cur, "SunRotate")?,
            sun_height: parse_scalar::<f32>(cur, "SunHeight")?,
            moon_tilt: parse_scalar::<f32>(cur, "MoonTilt")?,
            moon_rotate: parse_scalar::<f32>(cur, "MoonRotate")?,
            moon_height: parse_scalar::<f32>(cur, "MoonHeight")?,
            colour_lookup_column: parse_scalar::<i32>(cur, "ColourLookupColumn")?,
            editor_colour: parse_scalar::<u32>(cur, "EditorColour")?,
            fish_weight_mult: parse_scalar::<f32>(cur, "FishWeightMult")?,
        })
    }

    /// The `Time` control: an id, then a `u32` count, then that many inline
    /// keyframe bodies (each a sequence of [`EnvironmentThemeDef`] controls).
    fn parse_time_list(cur: &mut &[u8]) -> Result<Vec<EnvironmentThemeDef>, ParseControlError> {
        let name = "Time";
        let _id = parse_id(cur, name)?;
        let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
            name,
            reason: ParseControlErrorReason::ListCount(inner),
        })?;
        let mut time = Vec::with_capacity(count as usize);
        for _ in 0..count {
            time.push(EnvironmentThemeDef::parse(cur)?);
        }
        Ok(time)
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        self.serialize_time_list(out)?;
        serialize_scalar::<f32>(out, "SunTilt", self.sun_tilt)?;
        serialize_scalar::<f32>(out, "SunRotate", self.sun_rotate)?;
        serialize_scalar::<f32>(out, "SunHeight", self.sun_height)?;
        serialize_scalar::<f32>(out, "MoonTilt", self.moon_tilt)?;
        serialize_scalar::<f32>(out, "MoonRotate", self.moon_rotate)?;
        serialize_scalar::<f32>(out, "MoonHeight", self.moon_height)?;
        serialize_scalar::<i32>(out, "ColourLookupColumn", self.colour_lookup_column)?;
        serialize_scalar::<u32>(out, "EditorColour", self.editor_colour)?;
        serialize_scalar::<f32>(out, "FishWeightMult", self.fish_weight_mult)?;
        Ok(())
    }

    fn serialize_time_list(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        let name = "Time";
        serialize_id(out, name)?;
        put_le(out, &(self.time.len() as u32)).map_err(|inner| SerializeControlError {
            name,
            reason: SerializeControlErrorReason::ListCount(inner),
        })?;
        for keyframe in &self.time {
            keyframe.serialize(out)?;
        }
        Ok(())
    }

    pub(crate) fn byte_size(&self) -> usize {
        let time = ID_BYTE_SIZE
            + size_of::<u32>()
            + self.time.iter().map(|k| k.byte_size()).sum::<usize>();
        // 6 trailing f32 + ColourLookupColumn(i32) + EditorColour(u32) + FishWeightMult(f32) = 9
        time + 9 * (ID_BYTE_SIZE + size_of::<u32>())
    }
}
