//! `fable_data::mesh::Mesh` → [`Model`].

use renderer::{
    AlphaMode, Model, ModelMaterial, ModelPrimitive, ModelSkin, ModelSubMesh, ModelVertex,
    SkinnedPrimitive, SkinnedVertex,
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

    // A skinned mesh gets a second set of primitives whose vertices carry their bones. The
    // palette slot is resolved through the block's `Groups[]` here rather than on the GPU, so
    // the shader indexes the mesh's bones directly and needs no palette of its own.
    let skin = build_skin(mesh);

    Ok(Model {
        primitives,
        materials,
        skin,
    })
}

/// The skinned half of a model, or `None` for a static mesh.
///
/// A vertex's `slots()` index the *block's* palette (`Groups[]`, at most 18 entries), and each
/// entry is a bone index. Resolving that here keeps the shader's array a plain per-instance
/// bone palette.
fn build_skin(mesh: &Mesh) -> Option<ModelSkin> {
    if mesh.bones.is_empty() || !mesh.primitives.iter().any(|p| !p.blends.is_empty()) {
        return None;
    }

    let primitives = mesh
        .primitives
        .iter()
        .filter(|p| !p.blends.is_empty() && !p.indices.is_empty())
        .map(|primitive| {
            // **Each animated block has its own palette, and they do not agree.** The blocks
            // partition the primitive's vertex stream in order — `Σ block.vertex_count ==
            // primitive.vertex_count` on 282 of 282 animated primitives — and of the 37 with
            // more than one block, **0** share a palette. Resolving every vertex through the
            // first block's `Groups[]` therefore mis-binds everything past the first block,
            // which shows up as detached forearms and hands: `MESH_GRANNY`'s second block
            // covers the arms with `[29, 39, 41, …]` where the first holds `[6, 7, 4, …]`.
            let mut vertices = Vec::with_capacity(primitive.vertices.len());
            let mut start = 0usize;
            for block in &primitive.animated_blocks {
                let palette = block.groups.as_slice();
                let end = (start + block.vertex_count as usize).min(primitive.vertices.len());
                for i in start..end {
                    let (v, blend) = (&primitive.vertices[i], &primitive.blends[i]);
                    let slots = blend.slots();
                    let mut bones = [0u32; 4];
                    for k in 0..4 {
                        // A slot with no palette entry falls back to bone 0, whose matrix is
                        // the identity when undriven — an unresolvable weight then leaves the
                        // vertex in bind pose rather than flinging it to the origin.
                        bones[k] = palette
                            .get(slots[k] as usize)
                            .map(|&b| b as u32)
                            .unwrap_or(0);
                    }
                    vertices.push(SkinnedVertex {
                        position: v.pos,
                        normal: v.normal,
                        uv: v.uv,
                        bones,
                        weights: blend.weights(),
                    });
                }
                start = end;
            }
            // Any tail the blocks did not claim keeps its bind pose rather than being dropped,
            // which would leave holes in the mesh.
            for i in start..primitive.vertices.len() {
                let v = &primitive.vertices[i];
                vertices.push(SkinnedVertex {
                    position: v.pos,
                    normal: v.normal,
                    uv: v.uv,
                    bones: [0; 4],
                    weights: [1.0, 0.0, 0.0, 0.0],
                });
            }

            SkinnedPrimitive {
                vertices,
                indices: primitive.indices.clone(),
                sub_meshes: primitive
                    .sub_meshes
                    .iter()
                    .map(|s| ModelSubMesh {
                        material: s.material_index,
                        index_start: s.index_start,
                        index_count: s.index_count,
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    if primitives.is_empty() {
        return None;
    }
    Some(ModelSkin {
        primitives,
        bone_count: mesh.bones.len(),
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
