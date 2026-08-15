//! `fable_data::mesh::Mesh` → [`Model`].

use renderer::{
    AlphaMode, Model, ModelMaterial, ModelPrimitive, ModelSubMesh, ModelVertex,
};
use derive_more::{Display, Error};
use fable_data::{big::AssetMetadata, mesh::Mesh};

#[derive(Debug, Display, Error)]
pub enum BuildModelError {
    #[display("mesh has no primitives")]
    NoPrimitives,
}

/// Convert a decoded mesh and its resolved material textures into a renderable model.
///
/// `material_textures` is aligned 1:1 with `mesh.materials` (see `Files::read_mesh`).
/// A material whose texture is missing or fails to decode falls back to no diffuse map,
/// which the renderer draws with a white texture — a visible gap, not a silent one. Roughly
/// a quarter of the materials in `graphics.big` have `base_texture_id == 0`, so this is the
/// normal case, not an error.
///
/// `is_resident` answers "does the renderer already hold this asset id" — a texture it says
/// yes to is **not decoded at all**, only named. That is where most of the dedup win is:
/// across a level, between a half and three quarters of material texture references are
/// repeats (AGENTS.md §12.1, §12.6), and the archive read plus the BC slice is the larger part
/// of the work the renderer's own registry would only have deduplicated at upload.
///
/// Where the model stands is not this function's business — that is a `ModelInstance`.
pub fn build_model(
    mesh: &Mesh,
    material_textures: &[Option<(AssetMetadata, Vec<u8>)>],
    is_resident: impl Fn(u32) -> bool,
) -> Result<Model, BuildModelError> {
    if mesh.primitives.is_empty() {
        return Err(BuildModelError::NoPrimitives);
    }

    let materials = mesh
        .materials
        .iter()
        .enumerate()
        .map(|(i, material)| {
            let resolved = material_textures.get(i).and_then(|t| t.as_ref());
            // The id the renderer keys its registry by — the asset's own id, which is what
            // makes the same texture shared between two meshes one upload (AGENTS.md §3.11).
            let diffuse_id = resolved.map(|(asset, _)| asset.id);
            // Asked exactly once per material: it is the thing being counted, and a
            // double call would flatter the numbers it produces.
            let resident = diffuse_id.is_some_and(&is_resident);

            let diffuse = if resident {
                // Already there: name it and skip the read and the decode entirely.
                None
            } else {
                resolved.and_then(|(asset, data)| match super::decode_texture(asset, data) {
                    Ok(image) => Some(image),
                    Err(error) => {
                        tracing::warn!("Material {i} diffuse texture: {error}");
                        None
                    }
                })
            };

            ModelMaterial {
                // A texture that failed to decode has no usable map, so it must not claim a
                // registry slot it will never fill — otherwise every later mesh sharing that
                // id would be told it is resident and draw the fallback silently.
                diffuse_id: diffuse_id.filter(|_| resident || diffuse.is_some()),
                diffuse,
                alpha_mode: alpha_mode(material.boolean_alpha, material.transparent),
                two_sided: material.two_sided,
            }
        })
        .collect();

    let primitives = mesh
        .primitives
        .iter()
        .map(|primitive| ModelPrimitive {
            vertices: primitive
                .vertices
                .iter()
                .map(|v| ModelVertex {
                    position: v.pos,
                    normal: v.normal,
                    uv: v.uv,
                })
                .collect(),
            indices: primitive.indices.clone(),
            sub_meshes: primitive
                .sub_meshes
                .iter()
                .map(|s| ModelSubMesh {
                    material: s.material_index as u32,
                    index_start: s.index_start,
                    index_count: s.index_count,
                })
                .collect(),
        })
        .collect();

    Ok(Model {
        primitives,
        materials,
    })
}

/// Fable's two material alpha flags, mapped to the renderer's three-way mode.
/// `boolean_alpha` wins: an alpha-tested material is a cutout even if it is also flagged
/// transparent.
fn alpha_mode(boolean_alpha: bool, transparent: bool) -> AlphaMode {
    match (boolean_alpha, transparent) {
        (true, _) => AlphaMode::Cutout,
        (false, true) => AlphaMode::Blend,
        (false, false) => AlphaMode::Opaque,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_alpha_takes_precedence_over_transparent() {
        assert_eq!(alpha_mode(false, false), AlphaMode::Opaque);
        assert_eq!(alpha_mode(false, true), AlphaMode::Blend);
        assert_eq!(alpha_mode(true, false), AlphaMode::Cutout);
        assert_eq!(alpha_mode(true, true), AlphaMode::Cutout);
    }
}
