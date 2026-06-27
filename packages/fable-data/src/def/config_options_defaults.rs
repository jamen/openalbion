use super::binary::control::{ID_BYTE_SIZE, ParseControlError, SerializeControlError, parse_scalar, serialize_scalar};

#[derive(Debug)]
pub struct ConfigOptionsDefaultsDef {
    pub antialiasing: i32,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub bit_depth: u32,
    pub texture_detail: f32,
    pub max_texture_detail: f32,
    pub shadow_detail: f32,
    pub max_shadow_detail: f32,
    pub mesh_detail: f32,
    pub max_mesh_detail: f32,
    pub effects_detail: f32,
    pub max_effects_detail: f32,
    pub min_resolution_width: i32,
    pub min_resolution_height: i32,
}

impl ConfigOptionsDefaultsDef {
    pub(crate) const BYTE_SIZE: usize = ID_BYTE_SIZE * 12 + size_of::<ConfigOptionsDefaultsDef>();

    pub(crate) const fn byte_size(&self) -> usize {
        Self::BYTE_SIZE
    }

    pub(crate) fn parse(cur: &mut &[u8]) -> Result<Self, ParseControlError> {
        Ok(Self {
            antialiasing: parse_scalar::<i32>(cur, "Antialiasing")?,
            resolution_width: parse_scalar::<u32>(cur, "ResolutionWidth")?,
            resolution_height: parse_scalar::<u32>(cur, "ResolutionHeight")?,
            bit_depth: parse_scalar::<u32>(cur, "BitDepth")?,
            texture_detail: parse_scalar::<f32>(cur, "TextureDetail")?,
            max_texture_detail: parse_scalar::<f32>(cur, "MaxTextureDetail")?,
            shadow_detail: parse_scalar::<f32>(cur, "ShadowDetail")?,
            max_shadow_detail: parse_scalar::<f32>(cur, "MaxShadowDetail")?,
            mesh_detail: parse_scalar::<f32>(cur, "MeshDetail")?,
            max_mesh_detail: parse_scalar::<f32>(cur, "MaxMeshDetail")?,
            effects_detail: parse_scalar::<f32>(cur, "EffectsDetail")?,
            max_effects_detail: parse_scalar::<f32>(cur, "MaxEffectsDetail")?,
            min_resolution_width: parse_scalar::<i32>(cur, "MinResolutionWidth")?,
            min_resolution_height: parse_scalar::<i32>(cur, "MinResolutionHeight")?,
        })
    }

    pub(crate) fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeControlError> {
        serialize_scalar::<i32>(out, "Antialiasing", self.antialiasing)?;
        serialize_scalar::<u32>(out, "ResolutionWidth", self.resolution_width)?;
        serialize_scalar::<u32>(out, "ResolutionHeight", self.resolution_height)?;
        serialize_scalar::<u32>(out, "BitDepth", self.bit_depth)?;
        serialize_scalar::<f32>(out, "TextureDetail", self.texture_detail)?;
        serialize_scalar::<f32>(out, "MaxTextureDetail", self.max_texture_detail)?;
        serialize_scalar::<f32>(out, "ShadowDetail", self.shadow_detail)?;
        serialize_scalar::<f32>(out, "MaxShadowDetail", self.max_shadow_detail)?;
        serialize_scalar::<f32>(out, "MeshDetail", self.mesh_detail)?;
        serialize_scalar::<f32>(out, "MaxMeshDetail", self.max_mesh_detail)?;
        serialize_scalar::<f32>(out, "EffectsDetail", self.effects_detail)?;
        serialize_scalar::<f32>(out, "MaxEffectsDetail", self.max_effects_detail)?;
        serialize_scalar::<i32>(out, "MinResolutionWidth", self.min_resolution_width)?;
        serialize_scalar::<i32>(out, "MinResolutionHeight", self.min_resolution_height)?;
        Ok(())
    }

}
