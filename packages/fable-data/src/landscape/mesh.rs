//! Building a patch's layer meshes — a port of `CEngineLandscapeMeshBuilder`
//! (`fableengine/engine_landscape_mesh_builder.cpp`).
//!
//! A patch is [`PATCH_CELLS`] square. Each of its 17×17 vertices carries up to three ground
//! themes; each theme contributes a *layer* for [`MappingDirection::Top`] (using the theme's
//! base texture) and four more for the cliff directions (using its cliff texture). Layers
//! sharing a texture set and direction are merged, and each ends up as its own mesh, drawn
//! as its own alpha-blended pass.
//!
//! The per-vertex `Blend` is the theme's weight; the *direction's* weight comes from the
//! blend table (`super::build_blend_table`), which the vertex indexes with its packed normal
//! (`cliff_u` / `cliff_v`). The two multiply in the pixel shader.

use super::{LandscapeMap, MappingDirection, PATCH_CELLS, PATCH_VERTS, mapping_direction_blend};

/// The textures a theme contributes to one set of layers.
///
/// The base set feeds [`MappingDirection::Top`] and the cliff set feeds the other four;
/// `EngineThemeDef` carries them as `BaseTexture`/`CliffBaseTexture` and so on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerTextures {
    /// Sampled for colour, planar-projected along the layer's mapping direction.
    pub foreground: i32,
    pub background: i32,
    pub bump_map: i32,
    pub self_illumination: f32,
}

/// Resolves a `.lev` theme palette slot to the textures its layers should use.
///
/// The engine goes palette slot → `ThemeDefIndex` → `LandscapeThemes[class index]`
/// (`ReadThemesAndCreateLayers` via `PeekThemeId` and `GetDefClassIndexFromGlobalIndex`).
/// We keep the def lookup on the caller's side of the crate boundary, because resolving it
/// needs `game.bin` and this module must stay testable without one.
pub trait ThemeSource {
    /// `None` when the slot names no theme, which drops the layer entirely.
    fn base(&self, slot: u8) -> Option<LayerTextures>;
    fn cliff(&self, slot: u8) -> Option<LayerTextures>;
}

/// One vertex of a layer mesh — `CLandscapeLayerMesh::CVertex`
/// (`engine_landscape_layer_mesh.hpp:71`).
///
/// Position is implied: `x`/`y` are the grid offset within the patch, and height and the
/// lighting normal are looked up from the map when the GPU buffer is built, exactly as
/// `BuildForegroundVertexBuffer` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerVertex {
    /// Grid position within the patch, `0 ..= PATCH_CELLS`.
    pub x: u8,
    pub y: u8,
    /// This layer's theme weight at this vertex.
    pub blend: u8,
    /// The vertex normal's packed horizontal part — the blend table's lookup, *not* a
    /// ground-texture coordinate. See [`super::pack_normal_xy`].
    pub cliff_u: u8,
    pub cliff_v: u8,
}

/// One alpha-blended pass over a patch.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchLayer {
    pub textures: LayerTextures,
    pub mapping_direction: MappingDirection,
    pub vertices: Vec<LayerVertex>,
    /// Triangle list into `vertices`.
    pub indices: Vec<u16>,
}

/// A patch's layers, in draw order.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    /// Patch position in the patch grid.
    pub patch_x: i32,
    pub patch_y: i32,
    /// Vertex `(0, 0)`'s position in map cell coordinates.
    pub origin_x: i32,
    pub origin_y: i32,
    pub layers: Vec<PatchLayer>,
}

/// Layers whose weight at a vertex is at or below this contribute nothing there.
///
/// `ReadThemesAndCreateLayers` gates every layer on `0x10 < blend`
/// (`engine_landscape_mesh_builder.cpp:988` and the cliff branch below it).
const MIN_BLEND: u8 = 0x10;

/// The two triangles of a cell, as `(x, y)` offsets from its lower-left corner.
///
/// Indexed by `(x ^ y) & 1`, so the diagonal alternates in a checkerboard: even cells split
/// corner-to-corner, odd cells across the anti-diagonal. Both `BuildLayerMesh`
/// (`:1404`) and `AddPolysSurroundingPointWithMask` (`:1830`) carry this table; the winding
/// here is `BuildLayerMesh`'s, which is the one that reaches the GPU.
const CELL_TRIANGLES: [[[(u8, u8); 3]; 2]; 2] = [
    // (x ^ y) & 1 == 0 — diagonal from (0,0) to (1,1)
    [
        [(0, 0), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 1)],
    ],
    // (x ^ y) & 1 == 1 — diagonal from (1,0) to (0,1)
    [
        [(0, 0), (1, 0), (0, 1)],
        [(1, 0), (0, 1), (1, 1)],
    ],
];

/// The lighting normal at a map vertex — `CEngineMap::PeekMapNormal`
/// (`fableengine/engine_world_map.cpp:352`).
///
/// Each axis' slope is turned into a 2D vector against a run of 2 cells and **normalised
/// before** the two are combined, so a steep slope in one axis does not swamp the other.
pub fn vertex_normal(map: &LandscapeMap, x: i32, y: i32) -> [f32; 3] {
    let normalise2 = |a: f32, b: f32| {
        let len = (a * a + b * b).sqrt();
        if len > 0.0 { (a / len, b / len) } else { (0.0, 1.0) }
    };

    let (hor_x, hor_y) = normalise2(map.height_at(x - 1, y) - map.height_at(x + 1, y), 2.0);
    let (ver_x, ver_y) = normalise2(map.height_at(x, y - 1) - map.height_at(x, y + 1), 2.0);

    let z = hor_y * ver_y;
    let len = (hor_x * hor_x + ver_x * ver_x + z * z).sqrt();
    if len > 0.0 {
        [hor_x / len, ver_x / len, z / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Which layer a `(direction, texture set)` belongs to, and the per-vertex state we
/// accumulate for it before turning it into a mesh.
struct Layer {
    textures: LayerTextures,
    mapping_direction: MappingDirection,
    /// `[x][y]`, indexed like `CLayer::VertexBlend[17][17]`.
    vertex_blend: [[u8; PATCH_VERTS]; PATCH_VERTS],
    /// `[x][y][triangle]`, indexed like `CLayer::PolyMaskGrid[16][16][2]`.
    poly_mask: [[[bool; 2]; PATCH_CELLS]; PATCH_CELLS],
}

impl Layer {
    fn new(textures: LayerTextures, mapping_direction: MappingDirection) -> Layer {
        Layer {
            textures,
            mapping_direction,
            vertex_blend: [[0; PATCH_VERTS]; PATCH_VERTS],
            poly_mask: [[[false; 2]; PATCH_CELLS]; PATCH_CELLS],
        }
    }
}

/// Build every layer mesh for the patch at `(patch_x, patch_y)`.
///
/// `CEngineLandscapeMeshBuilder::BuildPatchMesh` (`:1321`): mask, then layers, then a mesh
/// per layer.
pub fn build_patch(
    map: &LandscapeMap,
    themes: &dyn ThemeSource,
    patch_x: i32,
    patch_y: i32,
) -> Patch {
    let origin_x = patch_x * PATCH_CELLS as i32;
    let origin_y = patch_y * PATCH_CELLS as i32;

    let mask = DirectionMask::build(map, origin_x, origin_y);
    let layers = read_themes_and_create_layers(map, themes, &mask, origin_x, origin_y);

    Patch {
        patch_x,
        patch_y,
        origin_x,
        origin_y,
        layers: layers
            .into_iter()
            .map(|layer| build_layer_mesh(&mask, &layer))
            .collect(),
    }
}

/// Per-vertex packed normals, and which mapping directions each vertex faces at all.
///
/// `CEngineLandscapeMeshBuilder::BuildMapDirMask` (`:623`).
struct DirectionMask {
    /// `[x][y]` packed normal, the vertex's `CliffLookupU` / `CliffLookupV`.
    cliff: [[(u8, u8); PATCH_VERTS]; PATCH_VERTS],
    /// `[x][y][direction]` — whether this direction's blend is non-zero here.
    faces: [[[bool; 5]; PATCH_VERTS]; PATCH_VERTS],
}

impl DirectionMask {
    fn build(map: &LandscapeMap, origin_x: i32, origin_y: i32) -> DirectionMask {
        let mut mask = DirectionMask {
            cliff: [[(0, 0); PATCH_VERTS]; PATCH_VERTS],
            faces: [[[false; 5]; PATCH_VERTS]; PATCH_VERTS],
        };

        for x in 0..PATCH_VERTS {
            for y in 0..PATCH_VERTS {
                // DIVERGENCE: the original derives this normal from up to eight surrounding
                // face normals under a weighting that de-emphasises steep faces
                // (`engine_landscape_mesh_builder.cpp:623`). That arithmetic is too mangled
                // in the decomp to transcribe honestly -- the face-normal construction reads
                // as stack slots Ghidra failed to map -- so we use the map's own vertex
                // normal, which is exact, sourced, and describes the same surface. The two
                // differ only in smoothing, which shifts how sharply a slope crosses between
                // ground and cliff textures. Revisit if the transition looks wrong.
                let normal = vertex_normal(map, origin_x + x as i32, origin_y + y as i32);
                mask.cliff[x][y] = super::pack_normal_xy(normal);
                for dir in MappingDirection::ALL {
                    mask.faces[x][y][dir.index()] = mapping_direction_blend(dir, normal) > 0.0;
                }
            }
        }

        mask
    }
}

/// Read the three themes at every vertex and accumulate them into layers.
///
/// `CEngineLandscapeMeshBuilder::ReadThemesAndCreateLayers` (`:798`), driven by
/// `BuildLayersFromThemes`' 17×17 loop (`:1111`).
fn read_themes_and_create_layers(
    map: &LandscapeMap,
    themes: &dyn ThemeSource,
    mask: &DirectionMask,
    origin_x: i32,
    origin_y: i32,
) -> Vec<Layer> {
    let mut layers: Vec<Layer> = Vec::new();
    // (direction, texture set) -> layer index. The original threads this through
    // `PassMappingTable[dir][texture]` and a `SharedTextureNextLayerIndex` chain; a linear
    // scan over a handful of layers is the same function.
    let pass_of = |layers: &mut Vec<Layer>,
                   textures: LayerTextures,
                   direction: MappingDirection| {
        if let Some(i) = layers
            .iter()
            .position(|l| l.mapping_direction == direction && l.textures == textures)
        {
            return i;
        }
        layers.push(Layer::new(textures, direction));
        layers.len() - 1
    };

    for y in 0..PATCH_VERTS {
        for x in 0..PATCH_VERTS {
            let (mx, my) = (origin_x + x as i32, origin_y + y as i32);

            let slots = [
                map.theme_slot(mx, my, 0),
                map.theme_slot(mx, my, 1),
                map.theme_slot(mx, my, 2),
            ];
            // The second layer resolving to nothing collapses the weights onto the first.
            let has_second = themes.base(slots[1]).is_some() || themes.cliff(slots[1]).is_some();
            let blends = map.theme_blends(mx, my, has_second);

            // Levels that would produce identical layers are merged and their weights
            // summed, so a vertex whose three themes share a texture contributes once at
            // full strength instead of three times at a third each. The original runs this
            // as two passes over the level triple, one keyed on the base texture set and one
            // on the cliff set, each with its own weight array (`:900-980`).
            let (base, base_blends) = merge_levels(&slots, &blends, |s| themes.base(s));
            let (cliff, cliff_blends) = merge_levels(&slots, &blends, |s| themes.cliff(s));

            for level in 0..3 {
                if let (Some(textures), blend) = (base[level], base_blends[level]) {
                    if blend > MIN_BLEND {
                        let i = pass_of(&mut layers, textures, MappingDirection::Top);
                        layers[i].vertex_blend[x][y] = blend;
                        add_polys_surrounding_point(mask, &mut layers[i], x, y);
                    }
                }
                if let (Some(textures), blend) = (cliff[level], cliff_blends[level]) {
                    if blend > MIN_BLEND {
                        for direction in MappingDirection::CLIFF {
                            let i = pass_of(&mut layers, textures, direction);
                            layers[i].vertex_blend[x][y] = blend;
                            add_polys_surrounding_point(mask, &mut layers[i], x, y);
                        }
                    }
                }
            }
        }
    }

    layers
}

/// Collapse levels that resolve to the same textures, summing their weights onto the first.
///
/// The weights already sum to 255 across the three levels, so the total is bounded and the
/// cast back to a byte cannot overflow.
fn merge_levels(
    slots: &[u8; 3],
    blends: &[u8; 3],
    resolve: impl Fn(u8) -> Option<LayerTextures>,
) -> ([Option<LayerTextures>; 3], [u8; 3]) {
    let mut textures = [resolve(slots[0]), resolve(slots[1]), resolve(slots[2])];
    let mut weights = [blends[0] as u16, blends[1] as u16, blends[2] as u16];

    for i in 0..3 {
        if textures[i].is_none() {
            continue;
        }
        for j in i + 1..3 {
            if textures[j] == textures[i] {
                weights[i] += weights[j];
                weights[j] = 0;
                textures[j] = None;
            }
        }
    }

    let clamped = [
        weights[0].min(255) as u8,
        weights[1].min(255) as u8,
        weights[2].min(255) as u8,
    ];
    (textures, clamped)
}

/// Mark the triangles touching vertex `(x, y)` that face this layer's direction at all.
///
/// `CEngineLandscapeMeshBuilder::AddPolysSurroundingPointWithMask` (`:1830`). A triangle is
/// added when it contains the vertex *and* at least one of its three corners faces the
/// layer's mapping direction — so a layer's mesh stops at the edge of the slope it applies
/// to instead of covering the whole patch.
fn add_polys_surrounding_point(mask: &DirectionMask, layer: &mut Layer, x: usize, y: usize) {
    let dir = layer.mapping_direction.index();

    for cell_y in y.saturating_sub(1)..=y {
        for cell_x in x.saturating_sub(1)..=x {
            if cell_x >= PATCH_CELLS || cell_y >= PATCH_CELLS {
                continue;
            }
            let parity = (cell_x ^ cell_y) & 1;

            for (triangle, corners) in CELL_TRIANGLES[parity].iter().enumerate() {
                if layer.poly_mask[cell_x][cell_y][triangle] {
                    continue;
                }
                let corner = |i: usize| {
                    (cell_x + corners[i].0 as usize, cell_y + corners[i].1 as usize)
                };

                let touches = (0..3).any(|i| corner(i) == (x, y));
                let faces = (0..3).any(|i| {
                    let (cx, cy) = corner(i);
                    mask.faces[cx][cy][dir]
                });

                if touches && faces {
                    layer.poly_mask[cell_x][cell_y][triangle] = true;
                }
            }
        }
    }
}

/// Turn a layer's poly mask into a mesh over only the vertices it actually uses.
///
/// `CEngineLandscapeMeshBuilder::BuildLayerMesh` (`:1404`): walk the mask, allocating a
/// compacted vertex index the first time each grid position is referenced.
fn build_layer_mesh(mask: &DirectionMask, layer: &Layer) -> PatchLayer {
    let mut vertex_id = [[u16::MAX; PATCH_VERTS]; PATCH_VERTS];
    let mut vertices: Vec<LayerVertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for x in 0..PATCH_CELLS {
        for y in 0..PATCH_CELLS {
            let parity = (x ^ y) & 1;
            for (triangle, corners) in CELL_TRIANGLES[parity].iter().enumerate() {
                if !layer.poly_mask[x][y][triangle] {
                    continue;
                }
                for corner in corners {
                    let (vx, vy) = (x + corner.0 as usize, y + corner.1 as usize);
                    let id = &mut vertex_id[vx][vy];
                    if *id == u16::MAX {
                        *id = vertices.len() as u16;
                        let (cliff_u, cliff_v) = mask.cliff[vx][vy];
                        vertices.push(LayerVertex {
                            x: vx as u8,
                            y: vy as u8,
                            blend: layer.vertex_blend[vx][vy],
                            cliff_u,
                            cliff_v,
                        });
                    }
                    indices.push(*id);
                }
            }
        }
    }

    PatchLayer {
        textures: layer.textures,
        mapping_direction: layer.mapping_direction,
        vertices,
        indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::landscape::tests::flat_map;

    /// Every slot resolves to the same theme, so a patch reduces to one TOP layer.
    struct OneTheme;

    impl ThemeSource for OneTheme {
        fn base(&self, slot: u8) -> Option<LayerTextures> {
            (slot != 0).then_some(LayerTextures {
                foreground: 100,
                background: 0,
                bump_map: 0,
                self_illumination: 0.0,
            })
        }
        fn cliff(&self, _slot: u8) -> Option<LayerTextures> {
            None
        }
    }

    /// A theme with distinct base and cliff textures, so both layer families appear.
    struct BaseAndCliff;

    impl ThemeSource for BaseAndCliff {
        fn base(&self, slot: u8) -> Option<LayerTextures> {
            (slot != 0).then_some(LayerTextures {
                foreground: 100,
                background: 0,
                bump_map: 0,
                self_illumination: 0.0,
            })
        }
        fn cliff(&self, slot: u8) -> Option<LayerTextures> {
            (slot != 0).then_some(LayerTextures {
                foreground: 200,
                background: 0,
                bump_map: 0,
                self_illumination: 0.0,
            })
        }
    }

    /// Flat ground faces only TOP, so only the base layer survives the mask — the four
    /// cliff layers are created but every triangle fails the `faces` test.
    #[test]
    fn flat_ground_produces_one_full_top_layer() {
        let lev = flat_map(32, 32, 0.0, (1, 1, 1), (255, 0));
        let map = LandscapeMap::new(&lev);

        let patch = build_patch(&map, &BaseAndCliff, 0, 0);

        let top: Vec<_> = patch
            .layers
            .iter()
            .filter(|l| l.mapping_direction == MappingDirection::Top)
            .collect();
        assert_eq!(top.len(), 1);
        // The whole patch: 16 x 16 cells, two triangles each.
        assert_eq!(top[0].indices.len(), PATCH_CELLS * PATCH_CELLS * 2 * 3);
        assert_eq!(top[0].vertices.len(), PATCH_VERTS * PATCH_VERTS);
        assert_eq!(top[0].textures.foreground, 100);

        for layer in &patch.layers {
            if layer.mapping_direction != MappingDirection::Top {
                assert!(
                    layer.indices.is_empty(),
                    "{:?} should be empty on flat ground",
                    layer.mapping_direction,
                );
            }
        }
    }

    /// Indices must address the layer's own compacted vertex list, not the patch grid.
    #[test]
    fn indices_stay_inside_the_compacted_vertex_list() {
        let lev = flat_map(32, 32, 0.0, (1, 1, 1), (255, 0));
        let map = LandscapeMap::new(&lev);

        let patch = build_patch(&map, &OneTheme, 0, 0);

        for layer in &patch.layers {
            for &i in &layer.indices {
                assert!((i as usize) < layer.vertices.len());
            }
            assert_eq!(layer.indices.len() % 3, 0);
        }
    }

    /// One texture per palette slot, so levels stay distinguishable.
    struct TexturePerSlot;

    impl ThemeSource for TexturePerSlot {
        fn base(&self, slot: u8) -> Option<LayerTextures> {
            (slot != 0).then_some(LayerTextures {
                foreground: slot as i32,
                background: 0,
                bump_map: 0,
                self_illumination: 0.0,
            })
        }
        fn cliff(&self, _slot: u8) -> Option<LayerTextures> {
            None
        }
    }

    /// A weight of 16 or less drops that level's layer (`0x10 < blend`).
    ///
    /// Themes `(1, 2, 2)` with strengths `(b0, b1)` give weights `[b0, b1, 255-b1-b0]`;
    /// levels 1 and 2 share slot 2 and merge, so slot 1's layer stands alone on `b0`.
    #[test]
    fn weights_at_or_below_the_threshold_are_dropped() {
        for (strength, slot_one_drawn) in [((16, 0), false), ((17, 0), true)] {
            let lev = flat_map(32, 32, 0.0, (1, 2, 2), strength);
            let map = LandscapeMap::new(&lev);

            let patch = build_patch(&map, &TexturePerSlot, 0, 0);
            let drawn = |texture: i32| {
                patch
                    .layers
                    .iter()
                    .any(|l| l.textures.foreground == texture && !l.indices.is_empty())
            };

            assert_eq!(drawn(1), slot_one_drawn, "strength {strength:?}");
            // Slot 2 keeps the remainder either way, so it is always drawn.
            assert!(drawn(2), "strength {strength:?}");
        }
    }

    /// Levels sharing a texture sum their weights rather than the last one winning.
    #[test]
    fn merged_levels_sum_their_weights() {
        // Themes (1, 2, 2), strengths (100, 50) -> weights [100, 50, 105]; levels 1 and 2
        // merge onto slot 2 at 155.
        let lev = flat_map(32, 32, 0.0, (1, 2, 2), (100, 50));
        let map = LandscapeMap::new(&lev);

        let patch = build_patch(&map, &TexturePerSlot, 0, 0);
        let blend_of = |texture: i32| {
            patch
                .layers
                .iter()
                .find(|l| l.textures.foreground == texture)
                .map(|l| l.vertices[0].blend)
        };

        assert_eq!(blend_of(1), Some(100));
        assert_eq!(blend_of(2), Some(155));
    }

    /// Layers merge on (direction, texture set), so one theme across three levels is one
    /// layer rather than three.
    #[test]
    fn layers_merge_on_direction_and_texture() {
        let lev = flat_map(32, 32, 0.0, (1, 1, 1), (100, 50));
        let map = LandscapeMap::new(&lev);

        let patch = build_patch(&map, &OneTheme, 0, 0);

        assert_eq!(patch.layers.len(), 1);
        assert_eq!(patch.layers[0].mapping_direction, MappingDirection::Top);
    }

    /// The patch origin is its grid position times the cell count.
    #[test]
    fn patch_origin_follows_the_patch_grid() {
        let lev = flat_map(64, 64, 0.0, (1, 1, 1), (255, 0));
        let map = LandscapeMap::new(&lev);

        let patch = build_patch(&map, &OneTheme, 2, 3);
        assert_eq!((patch.origin_x, patch.origin_y), (32, 48));
    }

    /// A flat heightfield has every normal straight up (`PeekMapNormal`).
    #[test]
    fn flat_ground_normals_point_up() {
        let lev = flat_map(32, 32, 0.5, (1, 1, 1), (255, 0));
        let map = LandscapeMap::new(&lev);

        let n = vertex_normal(&map, 8, 8);
        assert!((n[2] - 1.0).abs() < 1e-6, "{n:?}");
        assert!(n[0].abs() < 1e-6 && n[1].abs() < 1e-6, "{n:?}");
    }

    /// Ground rising towards +X tilts the normal towards -X.
    #[test]
    fn a_slope_tilts_the_normal_away_from_the_rise() {
        let mut lev = flat_map(32, 32, 0.0, (1, 1, 1), (255, 0));
        let stride = 33usize;
        for row in 0..stride {
            for col in 0..stride {
                lev.heightmap_cells[row * stride + col].height =
                    col as f32 / crate::landscape::HEIGHT_SCALE;
            }
        }
        let map = LandscapeMap::new(&lev);

        let n = vertex_normal(&map, 8, 8);
        assert!(n[0] < 0.0, "expected the normal to lean towards -X, got {n:?}");
        assert!(n[2] > 0.0, "{n:?}");
    }
}
