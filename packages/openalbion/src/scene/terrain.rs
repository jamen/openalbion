//! `.lev` heightmap → [`TerrainData`].
//!
//! The mesh building itself is `fable_data::landscape::mesh`, a port of
//! `CEngineLandscapeMeshBuilder`. This module is the conversion layer either side of it:
//! resolving the level's theme palette to `ENGINE_THEME` defs and their textures on the way
//! in, and expanding the patches' layer meshes into renderer vertices and draws on the way
//! out (which is what `CLandscapeLayerMesh::BuildForegroundVertexBuffer` does).

use crate::files::Files;
use fable_data::def::EngineThemeDef;
use fable_data::landscape::{
    BLEND_TABLE_SIZE, LandscapeMap, MappingDirection, Neighbour, build_blend_table,
    mesh::{self, LayerTextures, ThemeSource},
};
use fable_data::lev::Lev;
use fable_data::wld::RegionMap;
use renderer::{ImageFormat, TerrainData, TerrainDraw, TerrainVertex, TextureImage};
use std::collections::HashMap;

/// Build and merge a whole region's terrain in one pass — every map in `maps`, its own
/// `LandscapeMap` and theme palette resolved once, then queried across its own boundary into
/// whichever of the others actually borders it (AGENTS.md §3.4/§6.12). `maps` is exactly
/// [`Files::region_maps`]'s output: the requested level's `ContainsMap`s and `SeesMap`
/// fillers alike draw the same way — the `.wld`'s region *is* the input, not an
/// after-the-fact neighbour list assembled by the caller.
///
/// Resolving every map's data up front (rather than one at a time) is what makes the
/// boundary queries possible at all: a vertex on map A's outer edge needs map B's heightmap
/// *and* map B's own theme palette, both already built, before A's mesh can be assembled.
pub fn build_region_terrain(files: &mut Files, maps: &[(RegionMap, Lev)]) -> TerrainData {
    let landscape_maps: Vec<LandscapeMap> = maps.iter().map(|(_, lev)| LandscapeMap::new(lev)).collect();
    let themes: Vec<PaletteThemes> = maps.iter().map(|(_, lev)| PaletteThemes::resolve(files, lev)).collect();

    let mut terrains = Vec::with_capacity(maps.len());
    for i in 0..maps.len() {
        let neighbours: Vec<Neighbour> = (0..maps.len())
            .filter(|&j| j != i)
            .map(|j| Neighbour {
                map: &landscape_maps[j],
                themes: &themes[j],
                offset: (maps[i].0.origin.0 - maps[j].0.origin.0, maps[i].0.origin.1 - maps[j].0.origin.1),
            })
            .collect();
        terrains.push(build_one(
            files,
            &landscape_maps[i],
            &themes[i],
            maps[i].0.origin,
            &neighbours,
        ));
    }

    merge(terrains)
}

/// Build one map's geometry, layer passes and textures.
///
/// `origin` is the map's position in world cells (`MapX`/`MapY`, AGENTS.md §6.12) — added to
/// every vertex so multiple maps assemble into one world rather than stacking at (0, 0).
///
/// `neighbours` are the other loaded maps in this map's region. Without them, every vertex
/// on this map's own outer edge clamps to a repeat of its own last row instead of reading
/// the neighbour's true height, normal and theme there, which is what showed up first as a
/// crack and then as a mismatched texture at the seam between adjacent maps.
fn build_one(
    files: &mut Files,
    map: &LandscapeMap,
    themes: &PaletteThemes,
    origin: (i32, i32),
    neighbours: &[Neighbour],
) -> TerrainData {
    // One mesh per patch, then merged across patches by (texture, mapping direction) so the
    // whole level draws in as many passes as it has distinct layers rather than as many as
    // it has patches. The original sorts its patches for the same reason
    // (`RenderSortedPatches`); we can do it once up front because nothing streams yet.
    let mut batches: HashMap<(i32, MappingDirection), Batch> = HashMap::new();

    for patch_y in 0..map.patch_grid_height() {
        for patch_x in 0..map.patch_grid_width() {
            let patch = mesh::build_patch(map, themes, patch_x, patch_y, neighbours);

            for layer in &patch.layers {
                if layer.indices.is_empty() {
                    continue;
                }
                let batch = batches
                    .entry((layer.textures.foreground, layer.mapping_direction))
                    .or_default();

                let base = batch.vertices.len() as u32;
                batch.vertices.extend(layer.vertices.iter().map(|v| {
                    // Map-local for the height/normal lookups, which index this map's own
                    // heightmap; world for the position the vertex actually draws at.
                    let (x, y) = (
                        patch.origin_x + v.x as i32,
                        patch.origin_y + v.y as i32,
                    );
                    let (wx, wy) = (x + origin.0, y + origin.1);
                    TerrainVertex {
                        position: [wx as f32, wy as f32, map.height_at_stitched(x, y, neighbours)],
                        normal: mesh::vertex_normal_stitched(map, x, y, neighbours),
                        blend: v.blend as f32 / 255.0,
                        // The blend table is addressed over its whole extent, so the packed
                        // byte maps straight onto 0..1.
                        cliff_uv: [v.cliff_u as f32 / 255.0, v.cliff_v as f32 / 255.0],
                    }
                }));
                batch
                    .indices
                    .extend(layer.indices.iter().map(|&i| base + i as u32));
            }
        }
    }

    assemble(files, batches, themes)
}

/// Concatenate several maps' terrain into one upload, so a region draws through one
/// `TerrainPass::set_terrain` call rather than one per map.
///
/// Ground textures are rebased per map (each map has its own theme palette, so they are not
/// shared); the five direction blend tables are pure data — [`MappingDirection::ALL`] built
/// with no map-specific input — so every map's are byte-identical and only the first map's
/// copy is kept, with every draw's `blend_table` index (already `0..5`) left alone.
fn merge(terrains: Vec<TerrainData>) -> TerrainData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut textures = Vec::new();
    let mut draws = Vec::new();
    let mut blend_tables = Vec::new();

    for terrain in terrains {
        let vertex_base = vertices.len() as u32;
        let index_base = indices.len() as u32;
        let texture_base = textures.len() as u32;

        vertices.extend(terrain.vertices);
        indices.extend(terrain.indices.iter().map(|&i| i + vertex_base));
        textures.extend(terrain.textures);

        draws.extend(terrain.draws.into_iter().map(|draw| TerrainDraw {
            first_index: draw.first_index + index_base,
            texture: draw.texture + texture_base,
            ..draw
        }));

        if blend_tables.is_empty() {
            blend_tables = terrain.blend_tables;
        }
    }

    TerrainData {
        vertices,
        indices,
        draws,
        textures,
        blend_tables,
    }
}

#[derive(Default)]
struct Batch {
    vertices: Vec<TerrainVertex>,
    indices: Vec<u32>,
}

/// Flatten the per-(texture, direction) batches into one vertex buffer, one index buffer and
/// a draw list, loading each distinct ground texture once.
fn assemble(
    files: &mut Files,
    batches: HashMap<(i32, MappingDirection), Batch>,
    themes: &PaletteThemes,
) -> TerrainData {
    // Deterministic order, so a run is reproducible and the log reads the same twice.
    let mut keys: Vec<_> = batches.keys().copied().collect();
    keys.sort();

    let mut vertices: Vec<TerrainVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut textures: Vec<TextureImage> = Vec::new();
    let mut texture_slot: HashMap<i32, u32> = HashMap::new();
    let mut draws: Vec<TerrainDraw> = Vec::new();

    for key in keys {
        let (texture_id, direction) = key;
        let batch = &batches[&key];

        let slot = *texture_slot.entry(texture_id).or_insert_with(|| {
            let image = load_ground_texture(files, texture_id).unwrap_or_else(|error| {
                tracing::warn!("Ground texture {texture_id}: {error} — placeholder");
                placeholder_texture()
            });
            textures.push(image);
            (textures.len() - 1) as u32
        });

        let base = vertices.len() as u32;
        vertices.extend_from_slice(&batch.vertices);
        let first_index = indices.len() as u32;
        indices.extend(batch.indices.iter().map(|&i| base + i));

        let (u, v) = direction.uv_transform();
        draws.push(TerrainDraw {
            first_index,
            index_count: indices.len() as u32 - first_index,
            texture: slot,
            blend_table: direction.index() as u32,
            // `w` is the per-patch UV localisation offset — always a whole number of
            // texture tiles, so zero samples identically (tools/landscape-statics.md).
            uv_transform_u: [u[0], u[1], u[2], 0.0],
            uv_transform_v: [v[0], v[1], v[2], 0.0],
        });
    }

    tracing::info!(
        "Terrain: {} layer passes over {} themes, {} vertices, {} triangles",
        draws.len(),
        themes.by_slot.len(),
        vertices.len(),
        indices.len() / 3,
    );

    TerrainData {
        vertices,
        indices,
        draws,
        textures,
        blend_tables: MappingDirection::ALL
            .iter()
            // Single-mip, deliberately: the blend table is a lookup indexed by the packed
            // vertex normal, not a surface parameterisation, and §3.4's additive compositing
            // is correct only because the five directions' blends partition unity at every
            // texel. Mip-filtering it would break the partition and leak between layers.
            .map(|&d| {
                TextureImage::single(
                    BLEND_TABLE_SIZE as u32,
                    BLEND_TABLE_SIZE as u32,
                    ImageFormat::R8,
                    build_blend_table(d),
                )
            })
            .collect(),
    }
}

/// The level's theme palette, resolved to `ENGINE_THEME` defs.
struct PaletteThemes {
    by_slot: HashMap<u8, EngineThemeDef>,
}

impl PaletteThemes {
    /// Resolve every palette slot **by name**.
    ///
    /// `CMap::LoadFromFile` calls `GetDefGlobalIndexFromName` on the palette entry's name
    /// (`fablelib/map.cpp:2561`) rather than trusting the index stored beside it — and it is
    /// right not to: in retail data that index is stale, off by a constant 702 for every one
    /// of LookoutPoint's 38 entries. Resolving by index finds nothing, which is why the
    /// landscape has been untextured.
    fn resolve(files: &Files, lev: &Lev) -> PaletteThemes {
        let mut by_slot = HashMap::new();
        let mut unresolved = Vec::new();

        for (slot, entry) in lev.header.heightmap_palette.entries.iter().enumerate() {
            if entry.name.is_empty() || entry.name == "NO_THEME" {
                continue;
            }
            match files.engine_theme_by_name(&entry.name) {
                Some(def) => {
                    by_slot.insert(slot as u8, def.clone());
                }
                None => unresolved.push(entry.name.clone()),
            }
        }

        if !unresolved.is_empty() {
            tracing::warn!(
                "{} theme palette entries did not resolve to an ENGINE_THEME def: {:?}",
                unresolved.len(),
                &unresolved[..unresolved.len().min(8)],
            );
        }
        tracing::debug!("Theme palette: {} slots resolved by name", by_slot.len());

        PaletteThemes { by_slot }
    }
}

impl ThemeSource for PaletteThemes {
    fn base(&self, slot: u8) -> Option<LayerTextures> {
        let theme = self.by_slot.get(&slot)?;
        (theme.base_texture > 0).then_some(LayerTextures {
            foreground: theme.base_texture,
            background: theme.background_texture,
            bump_map: theme.base_bump_map,
            self_illumination: theme.base_texture_self_illumination,
        })
    }

    fn cliff(&self, slot: u8) -> Option<LayerTextures> {
        let theme = self.by_slot.get(&slot)?;
        (theme.cliff_base_texture > 0).then_some(LayerTextures {
            foreground: theme.cliff_base_texture,
            background: theme.cliff_background_texture,
            bump_map: theme.cliff_bump_map,
            self_illumination: theme.cliff_texture_self_illumination,
        })
    }
}

/// Ground textures keep their block compression, so the mip chain that ships in the archive
/// uploads verbatim. Each one is its own 2D texture bound per draw — there is no layer array,
/// and so no reason to decompress to a common format first.
fn load_ground_texture(files: &mut Files, texture_id: i32) -> Result<TextureImage, String> {
    let (asset, data) = files.read_texture_by_id(texture_id as u32)?;
    super::decode_texture(&asset, &data).map_err(|e| e.to_string())
}

/// Flat magenta — an obviously wrong texture beats a silently missing layer.
fn placeholder_texture() -> TextureImage {
    TextureImage::single(4, 4, ImageFormat::Rgba8, [255u8, 0, 255, 255].repeat(16))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f32) -> TerrainVertex {
        TerrainVertex {
            position: [x, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            blend: 1.0,
            cliff_uv: [0.0, 0.0],
        }
    }

    /// One map, one vertex, one triangle-less draw — just enough to check `merge`'s
    /// index and texture rebasing without needing a `.lev` or a GPU.
    fn one_map(x: f32, texture_id: u32) -> TerrainData {
        TerrainData {
            vertices: vec![vertex(x), vertex(x + 1.0)],
            indices: vec![0, 1],
            draws: vec![TerrainDraw {
                first_index: 0,
                index_count: 2,
                texture: texture_id,
                blend_table: 2,
                uv_transform_u: [1.0, 0.0, 0.0, 0.0],
                uv_transform_v: [0.0, 1.0, 0.0, 0.0],
            }],
            textures: vec![placeholder_texture()],
            blend_tables: MappingDirection::ALL
                .iter()
                .map(|&d| {
                    TextureImage::single(
                        BLEND_TABLE_SIZE as u32,
                        BLEND_TABLE_SIZE as u32,
                        ImageFormat::R8,
                        build_blend_table(d),
                    )
                })
                .collect(),
        }
    }

    /// Vertices and indices concatenate with indices rebased onto the growing vertex buffer,
    /// and each map's own ground textures shift by the running texture count.
    #[test]
    fn concatenates_vertices_and_rebases_indices_and_textures() {
        let merged = merge(vec![one_map(0.0, 0), one_map(100.0, 0)]);

        assert_eq!(merged.vertices.len(), 4);
        assert_eq!(merged.vertices[2].position[0], 100.0);

        assert_eq!(merged.draws.len(), 2);
        assert_eq!(merged.draws[0].first_index, 0);
        assert_eq!(merged.draws[0].texture, 0);
        assert_eq!(merged.draws[1].first_index, 2);
        assert_eq!(merged.draws[1].texture, 1, "second map's texture must rebase past the first");
        assert_eq!(
            &merged.indices[2..4],
            &[2, 3],
            "second map's indices must rebase onto the growing vertex buffer"
        );
    }

    /// The five direction blend tables are pure data — the same five tables regardless of
    /// which map built them — so merging N maps must not upload N copies.
    #[test]
    fn blend_tables_are_not_duplicated_per_map() {
        let merged = merge(vec![one_map(0.0, 0), one_map(1.0, 0), one_map(2.0, 0)]);
        assert_eq!(merged.blend_tables.len(), 5);
        // Every draw's blend_table index is untouched — it already indexes the shared five.
        assert!(merged.draws.iter().all(|d| d.blend_table == 2));
    }
}
