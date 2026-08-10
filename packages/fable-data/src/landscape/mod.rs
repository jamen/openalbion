//! The landscape as the engine sees it — `CEngineMap`'s accessors over a `.lev` heightmap.
//!
//! A `.lev` stores a `(width + 1) x (height + 1)` grid of [`LevHeightCell`]s: vertices, not
//! cells. [`LandscapeMap`] wraps one and reproduces the four accessors the landscape mesh
//! builder reads the world through — `PeekLandscapeHeight`, `PeekThemeId`, `PeekThemeBlend`
//! and the row indexing they share (`fableengine/engine_world_map_inline.cpp`).
//!
//! Two things about that indexing are easy to get wrong and are the reason this type exists
//! rather than callers touching `Lev::heightmap_cells` directly: the array is stored with
//! **row 0 at maximum Y**, and out-of-range samples **clamp to the cell grid**, which is one
//! smaller than the vertex grid.

pub mod mesh;

use crate::lev::{Lev, LevHeightCell};

/// `.lev` stores height as a small normalised float; `CMap::LoadFromFile` scales it into
/// world units while reading (`fablelib/map.cpp:2594` — `cell->Height = <file f32> * 2048.0f`).
pub const HEIGHT_SCALE: f32 = 2048.0;

/// A heightmap cell is one world unit square.
///
/// `CTVertexLandscapeForeground` stores X and Y as `unsigned short` and the foreground vertex
/// shader uses them as world position directly (`v0.xy`, then `r1 = r0 - c4` against
/// `CameraPos`), so grid coordinates *are* world coordinates. `FinalAlbion.wld` agrees: its
/// maps are 128 cells wide and neighbouring `MapX` values differ by 128.
pub const CELL_SIZE: f32 = 1.0;

/// Cells along one edge of a landscape patch.
///
/// `CEngineLandscapeMeshBuilder::CLayer::PolyMaskGrid[16][16][2]` and
/// `CEngineLandscapeMap::CEngineLandscapeMap`'s `PatchGridWidth = MapWidth >> 4`.
pub const PATCH_CELLS: usize = 16;

/// Vertices along one edge of a landscape patch — cells plus the shared far edge.
///
/// `CLayer::VertexBlend[17][17]`, `CCliffLookupTable::U[17][17]`, and
/// `BuildLayersFromThemes`' `while (i < 0x11)` loops.
pub const PATCH_VERTS: usize = PATCH_CELLS + 1;

/// The five directions a landscape layer's texture can be projected along.
///
/// `LANDSCAPE_TEXTURE_MAPPING_DIRECTION` (`fablelib/gui_var_transfer_struct.hpp:3072`). The
/// discriminants are the enum's own, and index every table below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum MappingDirection {
    Top = 0,
    Front = 1,
    Back = 2,
    Left = 3,
    Right = 4,
}

impl MappingDirection {
    pub const ALL: [MappingDirection; 5] = [
        MappingDirection::Top,
        MappingDirection::Front,
        MappingDirection::Back,
        MappingDirection::Left,
        MappingDirection::Right,
    ];

    /// The four directions a theme's *cliff* texture is applied along — every direction but
    /// [`MappingDirection::Top`], which takes the base texture instead
    /// (`ReadThemesAndCreateLayers`, `engine_landscape_mesh_builder.cpp:798`).
    pub const CLIFF: [MappingDirection; 4] = [
        MappingDirection::Front,
        MappingDirection::Back,
        MappingDirection::Left,
        MappingDirection::Right,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// The axis this direction faces, Z-up.
    ///
    /// `CEngineLandscapeMeshBuilder::MappingDirNormals` — read out of `ego_r.exe` at
    /// `.data` rva `0x00e188d0`; see `tools/landscape-statics.md`.
    pub fn normal(self) -> [f32; 3] {
        match self {
            MappingDirection::Top => [0.0, 0.0, 1.0],
            MappingDirection::Front => [0.0, -1.0, 0.0],
            MappingDirection::Back => [0.0, 1.0, 0.0],
            MappingDirection::Left => [-1.0, 0.0, 0.0],
            MappingDirection::Right => [1.0, 0.0, 0.0],
        }
    }

    /// The planar projection that turns a world position into this direction's texture UV,
    /// as the `(u, v)` rows the foreground vertex shader dot-products with (`c40` / `c41`).
    ///
    /// `CEngineLandscapePatch::PositionToTextureUVTransformU` / `…V` — read out of
    /// `ego_r.exe` at `.data` rva `0x00e19b64` / `0x00e19ba0`; see
    /// `tools/landscape-statics.md`. The `0.125` puts one texture tile across 8 world cells;
    /// the sign flips mirror FRONT against BACK and RIGHT against LEFT.
    ///
    /// The `w` of each register is a per-patch UV localisation offset which always resolves
    /// to a whole number of texture tiles, so under wrap addressing it samples identically
    /// and is omitted — see `tools/landscape-statics.md` for the derivation.
    pub fn uv_transform(self) -> ([f32; 3], [f32; 3]) {
        match self {
            MappingDirection::Top => ([0.125, 0.0, 0.0], [0.0, 0.125, 0.0]),
            MappingDirection::Front => ([-0.125, 0.0, 0.0], [0.0, 0.0, -0.125]),
            MappingDirection::Back => ([0.125, 0.0, 0.0], [0.0, 0.0, -0.125]),
            MappingDirection::Left => ([0.0, 0.125, 0.0], [0.0, 0.0, -0.125]),
            MappingDirection::Right => ([0.0, -0.125, 0.0], [0.0, 0.0, -0.125]),
        }
    }
}

/// Edge length of a blend table. `BuildBlendingTables` (`engine_landscape.cpp:826`)
/// initialises all five at `0x80` square.
pub const BLEND_TABLE_SIZE: usize = 128;

/// How much of a layer mapped along `direction` applies to a surface with this `normal`.
///
/// `CEngineLandscapeMeshBuilder::GetMappingDirectionBlend`
/// (`engine_landscape_mesh_builder.cpp:561`). Two ramps, both expressed as an angle over a
/// right angle:
///
/// - *topness* rises 0 → 1 as the normal tilts from 45° to 67.5° above horizontal, and is
///   the whole answer for [`MappingDirection::Top`];
/// - *sideness* falls 1 → 0 as the normal's heading turns 22.5° to 67.5° away from the
///   direction it faces, and is scaled by `1 - topness`.
///
/// The four side directions are 90° apart and their ramps are linear in angle across
/// exactly that span, so for any normal the five values sum to 1 — see the tests.
///
/// The arguments to the original's `asin` / `acos` are FPU-register-passed and so invisible
/// in the decomp; they are `normal.z` and the heading dot product respectively, which is the
/// only reading under which the constants (`0.5`, `4`, `0.25`, doubling) produce a
/// partition of unity.
pub fn mapping_direction_blend(direction: MappingDirection, normal: [f32; 3]) -> f32 {
    use std::f32::consts::FRAC_PI_2;

    // (asin(z) / (pi/2) - 0.5) * 4, clamped — 0 below 45°, 1 above 67.5°.
    let topness = ((normal[2].clamp(-1.0, 1.0).asin() / FRAC_PI_2 - 0.5) * 4.0).clamp(0.0, 1.0);

    if direction == MappingDirection::Top {
        return topness;
    }
    // The original tests this before normalising the horizontal part, which is what keeps a
    // perfectly flat normal (whose xy is zero, and un-normalisable) from producing a NaN.
    if topness == 1.0 {
        return 0.0;
    }

    let len = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
    if len == 0.0 {
        return 0.0;
    }
    let facing = direction.normal();
    let dot = ((normal[0] / len) * facing[0] + (normal[1] / len) * facing[1]).clamp(-1.0, 1.0);

    // 1 - 2 * (acos(dot) / (pi/2) - 0.25), clamped.
    let sideness = (1.0 - 2.0 * (dot.acos() / FRAC_PI_2 - 0.25)).clamp(0.0, 1.0);

    (1.0 - topness) * sideness
}

/// One direction's `BLEND_TABLE_SIZE` square alpha table, row-major, one byte per texel.
///
/// `CEngineLandscapeRenderer::BuildBlendingTables` (`engine_landscape.cpp:826`) builds these
/// once at startup and binds `ForegroundBlendTables[layer.MappingDirection]` to texture
/// stage 0 of the foreground pass (`engine_landscape_patch.cpp:1076`). Texel `(x, y)` holds
/// the blend for the normal that `(x, y)` decodes to, so a vertex looks its own blend up by
/// its packed normal — see [`pack_normal_xy`].
pub fn build_blend_table(direction: MappingDirection) -> Vec<u8> {
    let n = BLEND_TABLE_SIZE;
    let half = (n / 2) as f32;
    let mut out = vec![0u8; n * n];

    for y in 0..n {
        for x in 0..n {
            let nx = (x as f32 - half) / half;
            let ny = (y as f32 - half) / half;
            // The hemisphere the packed normal came from. Outside the unit circle there is
            // no such normal; the radicand clamps rather than producing a NaN.
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();

            let blend = mapping_direction_blend(direction, normalise([nx, ny, nz]));

            // `round(v * 255 - 0.5)`, then snap up when `v * 255` is exactly the next
            // integer. x87 rounds ties to even, which Rust spells `round_ties_even`;
            // plain `round` would send 0.0 to -1 and wrap the byte.
            let scaled = blend * 255.0;
            let mut rounded = (scaled - 0.5).round_ties_even();
            if scaled == rounded + 1.0 {
                rounded += 1.0;
            }
            out[y * n + x] = rounded.clamp(0.0, 255.0) as u8;
        }
    }

    out
}

/// Pack a vertex normal's horizontal part into the byte pair that indexes a blend table.
///
/// `CEngineLandscapeMeshBuilder::BuildMapDirMask` (`engine_landscape_mesh_builder.cpp:623`)
/// stores `round((normal.x * 0.5 + 0.5) * 255)` and the same for Y as the vertex's
/// `CliffLookupU` / `CliffLookupV`. Despite the name these are not texture coordinates for
/// any ground texture — they are the blend table lookup, and it is what the foreground
/// vertex shader passes through as `oT0.xy`.
pub fn pack_normal_xy(normal: [f32; 3]) -> (u8, u8) {
    let pack = |v: f32| ((v * 0.5 + 0.5) * 255.0).round_ties_even().clamp(0.0, 255.0) as u8;
    (pack(normal[0]), pack(normal[1]))
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// A `.lev`'s heightmap, addressed the way the engine addresses it.
///
/// Coordinates are **vertex** coordinates: `0 ..= width` by `0 ..= height`, with `+X` east
/// and `+Y` north, matching the world grid. Height is `+Z`.
pub struct LandscapeMap<'a> {
    lev: &'a Lev,
    /// Cell counts — one less than the vertex grid in each axis.
    width: i32,
    height: i32,
}

impl<'a> LandscapeMap<'a> {
    pub fn new(lev: &'a Lev) -> LandscapeMap<'a> {
        LandscapeMap {
            lev,
            width: lev.header.width as i32,
            height: lev.header.height as i32,
        }
    }

    /// Cells across. The vertex grid is one wider.
    pub fn cell_width(&self) -> i32 {
        self.width
    }

    /// Cells down. The vertex grid is one taller.
    pub fn cell_height(&self) -> i32 {
        self.height
    }

    /// Patches across, `MapWidth >> 4` (`CEngineLandscapeMap::CEngineLandscapeMap`).
    pub fn patch_grid_width(&self) -> i32 {
        self.width >> 4
    }

    pub fn patch_grid_height(&self) -> i32 {
        self.height >> 4
    }

    /// The cell backing vertex `(x, y)`, with the engine's clamp.
    ///
    /// `PeekThemeId` / `PeekThemeBlend` / `PeekLandscapeHeight` all share this preamble: a
    /// sample outside `0 .. MapWidth` (the *cell* count) is resolved against the
    /// neighbouring map through `CEngineWorldMap`'s grid, and clamped to `MapWidth - 1` only
    /// when no neighbour supplies it. We load one map at a time, so the clamp always
    /// applies — including at `x == width`, the seam column a neighbour would own.
    ///
    /// Loading neighbours is what turns this back into the in-game result; until then this
    /// is exactly what the original does with an isolated map.
    fn cell(&self, x: i32, y: i32) -> &LevHeightCell {
        let x = x.clamp(0, self.width - 1);
        let y = y.clamp(0, self.height - 1);
        // **World Y is the file's row index, with no flip.**
        //
        // The engine's accessors do flip — `CMap::GetGroundSizeZAtIncludingExtraCells`
        // (`fablelib/map.cpp:2135`) and `CMap::DrawBlockGetSizeZAt`
        // (`fablelib/map_render.cpp:119`) both index `(SizeY - y) * (SizeX + 1) + x`. But so
        // does the **load loop** (`fablelib/map.cpp:2600`), which walks the file
        // sequentially and writes each cell to
        //
        // ```c
        // iVar13 = index / rowWidth;                                 // file row
        // Cells + (SizeY - iVar13) * (SizeX + 1) + iVar16            // flipped on WRITE
        // ```
        //
        // so file row `f` lands at array row `SizeY - f`. Reading world `Y` back out of
        // array row `SizeY - Y` therefore returns **file row `Y`**: the two flips cancel,
        // and `(SizeY - y)` is a property of the engine's in-memory layout, not of the file.
        //
        // We keep the cells in file order, so applying the read-side flip on its own mirrors
        // the terrain in Y. It did, for as long as nothing else was drawn in world space to
        // disagree — the `.tng` placements were the first independent witness, and they are
        // decisive: unflipped puts 90% of LookoutPoint's things within a metre of the ground
        // where flipped manages 28%, and the mean deviation falls from 2.85 to 0.29.
        let row = y as usize;
        let stride = (self.width + 1) as usize;
        &self.lev.heightmap_cells[row * stride + x as usize]
    }

    /// World-space Z at vertex `(x, y)`.
    ///
    /// `CEngineMap::PeekLandscapeHeight` quantises to 1/128 of a world unit
    /// (`round(h * 128) * 0.0078125`) after fetching, which we reproduce so that heights
    /// derived here match the ones the original's meshes were built from.
    pub fn height_at(&self, x: i32, y: i32) -> f32 {
        let raw = self.cell(x, y).height * HEIGHT_SCALE;
        (raw * 128.0).round() * (1.0 / 128.0)
    }

    /// The theme palette slot at vertex `(x, y)` for blend level `0..3`.
    pub fn theme_slot(&self, x: i32, y: i32, level: usize) -> u8 {
        let cell = self.cell(x, y);
        let (a, b, c) = cell.ground_theme;
        match level {
            0 => a,
            1 => b,
            _ => c,
        }
    }

    /// The stored blend byte at vertex `(x, y)` for level `0..2`.
    ///
    /// Only two are stored; the third layer's weight is the remainder — see
    /// [`LandscapeMap::theme_blends`].
    pub fn theme_blend(&self, x: i32, y: i32, level: usize) -> u8 {
        let cell = self.cell(x, y);
        let (a, b) = cell.ground_theme_strength;
        if level == 0 { a } else { b }
    }

    /// The three layer weights at vertex `(x, y)`, renormalised to sum to 255.
    ///
    /// `ReadThemesAndCreateLayers` (`engine_landscape_mesh_builder.cpp:798`) reads the two
    /// stored bytes, derives the third as `255 - b1 - b0`, then scales all three by
    /// `255 / total`. `has_second_layer` is false when the level-1 theme resolves to no
    /// def, in which case levels 1 and 2 carry no weight at all.
    pub fn theme_blends(&self, x: i32, y: i32, has_second_layer: bool) -> [u8; 3] {
        let b0 = self.theme_blend(x, y, 0) as i32;
        let (b1, b2) = if has_second_layer {
            let b1 = self.theme_blend(x, y, 1) as i32;
            (b1, 255 - b1 - b0)
        } else {
            (0, 0)
        };

        let total = b0 + b1 + b2;
        if total <= 0 {
            return [0, 0, 0];
        }
        [
            (b0 * 255 / total) as u8,
            (b1 * 255 / total) as u8,
            (b2 * 255 / total) as u8,
        ]
    }

    /// The palette entry name for a slot, or `""` for the empty sentinel.
    ///
    /// The `.lev` palette stores a name beside a def index, and `CMap::LoadFromFile`
    /// re-resolves the index from the name at load time
    /// (`GetDefGlobalIndexFromName`, `fablelib/map.cpp:2561`). The stored index is stale in
    /// retail data — every one of LookoutPoint's 38 entries is off by a constant 702 — so
    /// **the name is the identifier** and the index must not be used.
    pub fn palette_name(&self, slot: u8) -> &'a str {
        self.lev
            .header
            .heightmap_palette
            .entries
            .get(slot as usize)
            .map(|e| e.name.as_str())
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lev::{LevHeader, LevNavigation, ThemePalette, ThemePaletteEntry};

    fn cell(height: f32, theme: (u8, u8, u8), strength: (u8, u8)) -> LevHeightCell {
        LevHeightCell {
            size: 0,
            version: 0,
            height,
            ground_theme: theme,
            ground_theme_strength: strength,
            walkable: true,
            passover: false,
            sound_theme: 0,
            shore: false,
        }
    }

    /// A `w x h` cell grid with `(w+1) * (h+1)` vertices, where each vertex's height encodes
    /// its own array position so the indexing can be checked exactly.
    fn map_of(w: u32, h: u32) -> Lev {
        let cells = (0..(w + 1) * (h + 1))
            .map(|i| cell(i as f32 / HEIGHT_SCALE, (0, 0, 0), (255, 0)))
            .collect();
        let palette = ThemePalette {
            entries: (0..256)
                .map(|i| ThemePaletteEntry {
                    name: format!("THEME_{i}"),
                    def_index: i,
                })
                .collect(),
        };
        lev_of(w, h, cells, palette)
    }

    /// A `w x h` cell grid at a uniform height, theme and blend strength.
    pub(crate) fn flat_map(
        w: u32,
        h: u32,
        world_height: f32,
        theme: (u8, u8, u8),
        strength: (u8, u8),
    ) -> Lev {
        let cells = (0..(w + 1) * (h + 1))
            .map(|_| cell(world_height / HEIGHT_SCALE, theme, strength))
            .collect();
        let palette = ThemePalette {
            entries: (0..256)
                .map(|i| ThemePaletteEntry {
                    name: if i == 0 {
                        String::new()
                    } else {
                        format!("THEME_{i}")
                    },
                    def_index: i,
                })
                .collect(),
        };
        lev_of(w, h, cells, palette)
    }

    fn lev_of(w: u32, h: u32, cells: Vec<LevHeightCell>, palette: ThemePalette) -> Lev {
        Lev {
            header: LevHeader {
                version: 0,
                obsolete_offset: 0,
                navigation_offset: 0,
                unique_id_count: 0,
                width: w,
                height: h,
                map_version: 0,
                ambient_sound_version: 0,
                checksum: 0,
                sound_themes: Vec::new(),
                heightmap_palette: palette.clone(),
                sound_palette: palette,
            },
            heightmap_cells: cells,
            soundmap_cells: Vec::new(),
            navigation: LevNavigation {
                sections_start: 0,
                section_names: Vec::new(),
                sections: Vec::new(),
            },
        }
    }

    /// **World Y is the file's row index, with no flip.** The engine's `(SizeY - y)` appears
    /// on both sides — the load loop writes file row `f` to array row `SizeY - f`
    /// (`fablelib/map.cpp:2600`) and the accessors read world `Y` from array row `SizeY - Y`
    /// (`map.cpp:2135`, `map_render.cpp:119`) — so the two cancel. See `cell`.
    #[test]
    fn world_y_is_the_file_row() {
        let lev = map_of(4, 3);
        let map = LandscapeMap::new(&lev);
        let stride = 5;

        // y = 0 is the *first* row of the file, and Y counts forward through it.
        assert_eq!(map.height_at(0, 0), 0.0);
        assert_eq!(map.height_at(0, 1), stride as f32);
        assert_eq!(map.height_at(2, 1), (stride + 2) as f32);

        // The last file row is the far seam, which only a neighbouring map can reach:
        // y = height clamps to height - 1 and lands on the row before it.
        assert_eq!(map.height_at(0, 3), (2 * stride) as f32);
    }

    /// Samples clamp to the *cell* grid, which is one smaller than the vertex grid, so the
    /// seam vertex a neighbouring map would own reads its inboard neighbour.
    #[test]
    fn samples_clamp_to_the_cell_grid() {
        let lev = map_of(4, 3);
        let map = LandscapeMap::new(&lev);

        assert_eq!(map.height_at(4, 0), map.height_at(3, 0));
        assert_eq!(map.height_at(99, 0), map.height_at(3, 0));
        assert_eq!(map.height_at(-1, 0), map.height_at(0, 0));
        assert_eq!(map.height_at(0, 3), map.height_at(0, 2));
    }

    /// `cell->Height = <file f32> * 2048.0f` (`fablelib/map.cpp:2594`), then quantised to
    /// 1/128 by `PeekLandscapeHeight`.
    #[test]
    fn height_is_scaled_by_2048_and_quantised() {
        let mut lev = map_of(1, 1);
        for c in &mut lev.heightmap_cells {
            c.height = 0.0221;
        }
        let map = LandscapeMap::new(&lev);

        let expected = (0.0221f32 * 2048.0 * 128.0).round() / 128.0;
        assert_eq!(map.height_at(0, 0), expected);
        // The quantisation is real: 0.0221 * 2048 is not a multiple of 1/128.
        assert_ne!(map.height_at(0, 0), 0.0221 * 2048.0);
    }

    /// Two stored bytes, three weights, renormalised to 255.
    #[test]
    fn blends_renormalise_to_255() {
        let mut lev = map_of(1, 1);
        for c in &mut lev.heightmap_cells {
            c.ground_theme_strength = (100, 50);
        }
        let map = LandscapeMap::new(&lev);

        // b0 = 100, b1 = 50, b2 = 255 - 50 - 100 = 105; total is already 255.
        assert_eq!(map.theme_blends(0, 0, true), [100, 50, 105]);
        // Without a second layer the first takes everything.
        assert_eq!(map.theme_blends(0, 0, false), [255, 0, 0]);
    }

    #[test]
    fn patch_grid_is_the_cell_count_over_sixteen() {
        let lev = map_of(128, 128);
        let map = LandscapeMap::new(&lev);
        assert_eq!(map.patch_grid_width(), 8);
        assert_eq!(map.patch_grid_height(), 8);
    }

    /// Flat ground is entirely TOP, and nothing else.
    #[test]
    fn flat_ground_is_all_top() {
        let up = [0.0, 0.0, 1.0];
        assert_eq!(mapping_direction_blend(MappingDirection::Top, up), 1.0);
        for dir in MappingDirection::CLIFF {
            assert_eq!(mapping_direction_blend(dir, up), 0.0, "{dir:?}");
        }
    }

    /// A vertical face is entirely the direction it points at. `MappingDirNormals` says
    /// FRONT faces `-Y`, so a wall whose normal is `-Y` is pure FRONT.
    #[test]
    fn a_vertical_face_is_all_its_own_direction() {
        for dir in MappingDirection::CLIFF {
            let n = dir.normal();
            assert_eq!(mapping_direction_blend(dir, n), 1.0, "{dir:?}");
            assert_eq!(mapping_direction_blend(MappingDirection::Top, n), 0.0);
            for other in MappingDirection::CLIFF {
                if other != dir {
                    assert_eq!(mapping_direction_blend(other, n), 0.0, "{dir:?} vs {other:?}");
                }
            }
        }
    }

    /// The whole point of the five ramps: every surface is covered exactly once, so no
    /// normal can leave a hole or double-darken through the `x2` in the pixel shader.
    #[test]
    fn the_five_directions_partition_unity() {
        // A spread of directions over the upper hemisphere, including the ramp shoulders
        // at 45 / 67.5 degrees of tilt and 22.5 / 45 / 67.5 of heading.
        for tilt_deg in [0.0f32, 10.0, 22.5, 45.0, 55.0, 67.5, 80.0, 90.0] {
            for heading_deg in [0.0f32, 15.0, 22.5, 45.0, 67.5, 90.0, 137.0, 200.0, 315.0] {
                let tilt = tilt_deg.to_radians();
                let heading = heading_deg.to_radians();
                let n = [
                    tilt.cos() * heading.cos(),
                    tilt.cos() * heading.sin(),
                    tilt.sin(),
                ];
                let sum: f32 = MappingDirection::ALL
                    .iter()
                    .map(|d| mapping_direction_blend(*d, n))
                    .sum();
                assert!(
                    (sum - 1.0).abs() < 1e-5,
                    "tilt {tilt_deg} heading {heading_deg}: blends sum to {sum}, not 1",
                );
            }
        }
    }

    /// The two ramp shoulders, read straight off the constants in the decomp.
    #[test]
    fn topness_ramps_between_45_and_675_degrees() {
        let at = |tilt_deg: f32| {
            let t = tilt_deg.to_radians();
            mapping_direction_blend(MappingDirection::Top, [t.cos(), 0.0, t.sin()])
        };
        assert_eq!(at(45.0), 0.0);
        assert!((at(56.25) - 0.5).abs() < 1e-5, "midpoint should be half");
        assert!((at(67.5) - 1.0).abs() < 1e-5, "should be saturated by 67.5°");
        assert_eq!(at(80.0), 1.0);
    }

    /// A table is indexed by the packed normal, so the texel a vertex lands on must carry
    /// the blend that vertex's normal deserves.
    #[test]
    fn table_texels_agree_with_the_blend_function() {
        let table = build_blend_table(MappingDirection::Top);
        assert_eq!(table.len(), BLEND_TABLE_SIZE * BLEND_TABLE_SIZE);

        // Straight up packs to the middle of the table and is fully TOP.
        let (u, v) = pack_normal_xy([0.0, 0.0, 1.0]);
        let texel = |u: u8, v: u8| {
            let x = (u as usize * BLEND_TABLE_SIZE) / 256;
            let y = (v as usize * BLEND_TABLE_SIZE) / 256;
            table[y * BLEND_TABLE_SIZE + x]
        };
        assert_eq!(texel(u, v), 255);

        // A wall packs to an edge and is not TOP at all.
        let (u, v) = pack_normal_xy([0.0, -1.0, 0.0]);
        assert_eq!(texel(u, v), 0);
    }

    /// `round(v * 255 - 0.5)` with the snap-up correction must not wrap at either end.
    #[test]
    fn table_quantisation_covers_the_full_byte_range() {
        for dir in MappingDirection::ALL {
            let table = build_blend_table(dir);
            assert!(table.contains(&255), "{dir:?} never reaches full blend");
            assert!(table.contains(&0), "{dir:?} never reaches zero blend");
        }
    }

    /// The tables read out of `ego_r.exe`: each direction projects onto the plane its normal
    /// faces, so the projection rows are orthogonal to it.
    #[test]
    fn uv_transforms_are_orthogonal_to_their_normal() {
        for dir in MappingDirection::ALL {
            let n = dir.normal();
            let (u, v) = dir.uv_transform();
            let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
            assert_eq!(dot(u, n), 0.0, "{dir:?} U is not in the projection plane");
            assert_eq!(dot(v, n), 0.0, "{dir:?} V is not in the projection plane");
            // One tile per 8 world cells, in both axes.
            assert_eq!(dot(u, u).sqrt(), 0.125);
            assert_eq!(dot(v, v).sqrt(), 0.125);
        }
    }
}
