//! The placement loop — a map's cells in, a level's worth of objects out.
//!
//! Ports `CLocalDetailCacheMap::CQuadTreeElement::AddObjectsFromBlendedThemes` and
//! `::AddObjectsFromLayerElement` (`fableengine/engine_local_detail_cache.cpp:3956`, `:3256`)
//! over [`LandscapeMap`], without the quadtree, the cache groups or the file blocks those
//! two live inside. Those exist to page a thirty-map world through an Xbox's memory; we load
//! one map at a time, and a whole map's objects fit in a `Vec`.
//!
//! Everything here is a pure function of the map, the defs and the displacement table, so a
//! level's foliage is reproducible byte for byte — which is the point.

use super::generator::{Generator, LayerObject, ObjectType, PrimitiveType};
use super::rng::DisplacementTable;
use crate::landscape::{LandscapeMap, mesh::vertex_normal};

/// Mesh coordinates are a hundred times world coordinates.
///
/// The same constant `.tng` things carry (AGENTS.md §3.11), sourced here a second time and
/// independently: `AddObjectsFromLayerElement` builds its scale as
/// `(Scale + (2·rand − 1)·ScaleRandomElement) * 9.999999776482582e-3`
/// (`engine_local_detail_cache.cpp:3341`). Local detail has no `RenderSizeX` — it is not
/// going through `CTCGraphicAppearance` — so this is the whole of it.
const MESH_UNITS_PER_WORLD_UNIT: f32 = 0.01;

/// Entries in the fast cosine table, and the divisor turning a `[0, 1)` draw into an index.
///
/// `GFastCosTable` is 1024 `cos(2πi/1024)` filled at startup
/// (`bbblibrary/lib_maths.cpp:41-47`, a loop of `0x1000` bytes in steps of 4). Sine is the
/// same table read 256 entries earlier: `cos(θ − π/2) = sin θ`.
const COS_TABLE_SIZE: i32 = 1024;

/// The generators a level's theme palette resolves to.
///
/// Built by the caller, because getting from a palette slot to a `LOCAL_DETAIL_GENERATOR`
/// def runs through `ENGINE_THEME` and the compiled defs, which this crate's landscape layer
/// deliberately does not know about.
pub struct GeneratorSet {
    generators: Vec<Generator>,
    /// Palette slot → index into `generators`.
    by_slot: [Option<u16>; 256],
}

impl Default for GeneratorSet {
    fn default() -> GeneratorSet {
        GeneratorSet {
            generators: Vec::new(),
            by_slot: [None; 256],
        }
    }
}

impl GeneratorSet {
    pub fn new() -> GeneratorSet {
        GeneratorSet::default()
    }

    /// Add a generator and point one or more palette slots at it.
    ///
    /// Returns the index it was stored at, so several slots can share one — themes routinely
    /// name the same generator, and building its placement grids twice would be wasted work.
    pub fn insert(&mut self, generator: Generator, slots: &[u8]) -> usize {
        let index = self.generators.len();
        self.generators.push(generator);
        for &slot in slots {
            self.by_slot[slot as usize] = Some(index as u16);
        }
        index
    }

    pub fn point_slot_at(&mut self, slot: u8, index: usize) {
        self.by_slot[slot as usize] = Some(index as u16);
    }

    pub fn for_slot(&self, slot: u8) -> Option<&Generator> {
        self.by_slot[slot as usize].map(|i| &self.generators[i as usize])
    }

    fn index_for_slot(&self, slot: u8) -> Option<u16> {
        self.by_slot[slot as usize]
    }

    pub fn get(&self, index: u16) -> &Generator {
        &self.generators[index as usize]
    }

    pub fn len(&self) -> usize {
        self.generators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }
}

/// One object, placed.
#[derive(Debug, Clone, Copy)]
pub struct PlacedObject {
    /// Which generator in the [`GeneratorSet`] produced it.
    pub generator: u16,
    /// Which of that generator's `object_types` it is.
    pub object_type: u16,
    /// Object → world, column-major, Z-up — the same convention as
    /// [`crate::tng::Placement::object_matrix`]. Scale is already folded into the rotation.
    pub transform: [[f32; 4]; 4],
    /// The uniform scale in the transform, kept separately because the engine passes it to
    /// `AddObject` alongside the matrix and the renderer wants it for bounding volumes.
    pub scale: f32,
    /// The interpolated landscape normal under the object. Objects with
    /// `LandscapeNormalLighting` — every repeated mesh — are lit once from this rather than
    /// per vertex.
    pub normal: [f32; 3],
    /// The object's rotation about Z, as a `[0, 1)` fraction of a turn.
    ///
    /// Already folded into `transform`, and kept because two other things need it:
    /// `NLocalDetailCache::CSourceObject::Angle` is what a repeated mesh's four-float
    /// `ObjectMatricies` entry is rebuilt from, and
    /// `CLocalDetailPrimitiveRepeatedMesh::SetupWindAnimation` turns it into the per-object
    /// `WindDelayArray` byte — so wind animation needs no second draw.
    pub angle: f32,
}

impl PlacedObject {
    pub fn object_type<'a>(&self, set: &'a GeneratorSet) -> &'a ObjectType {
        &set.get(self.generator).object_types[self.object_type as usize]
    }

    pub fn primitive_type(&self, set: &GeneratorSet) -> PrimitiveType {
        self.object_type(set).primitive_type
    }

    /// World position — the transform's translation column.
    pub fn position(&self) -> [f32; 3] {
        let t = self.transform[3];
        [t[0], t[1], t[2]]
    }
}

/// Every local detail object on `map`, positioned in **map-local** cells.
///
/// `origin` is the map's position in world cells (`CEngineMap::WorldPosX`/`WorldPosY`, which
/// `FinalAlbion.wld` calls `MapX`/`MapY`), and it affects **only the random draws**. That
/// split is worth being explicit about, because the engine does not make it: it works in one
/// world space where a map's terrain and its foliage are both offset by `MapX`/`MapY`, and
/// `AddObjectsFromLayerElement` accordingly writes world coordinates into the object matrix.
/// We build one map at a time at the world origin — the landscape pass does the same, its
/// vertices being plain cell indices — so a world-positioned object would land thousands of
/// cells off the terrain. The draws still use world coordinates, because that is what decides
/// *which* foliage grows and getting it wrong would give a map the wrong plants.
///
/// When neighbouring maps load (AGENTS.md step 6.12) each one is offset by its own origin,
/// terrain and foliage together, and the two spaces become one again.
pub fn place_map(
    map: &LandscapeMap,
    generators: &GeneratorSet,
    origin: (i32, i32),
    table: &DisplacementTable,
) -> Vec<PlacedObject> {
    let mut placed = Vec::new();
    if generators.is_empty() {
        return placed;
    }

    for y in 0..map.cell_height() {
        for x in 0..map.cell_width() {
            add_objects_from_blended_themes(map, generators, origin, table, x, y, &mut placed);
        }
    }

    placed
}

/// `AddObjectsFromBlendedThemes` (`engine_local_detail_cache.cpp:3956`).
///
/// The three ground themes blended at this cell each get a share of the cell's placement
/// points proportional to their weight — chosen per point by a random draw against the blend
/// bytes, not by splitting the points up. Layers are walked outermost, points innermost, and
/// both loops run while *any* of the three themes still has something to contribute.
fn add_objects_from_blended_themes(
    map: &LandscapeMap,
    generators: &GeneratorSet,
    origin: (i32, i32),
    table: &DisplacementTable,
    x: i32,
    y: i32,
    out: &mut Vec<PlacedObject>,
) {
    let slots = [
        map.theme_slot(x, y, 0),
        map.theme_slot(x, y, 1),
        map.theme_slot(x, y, 2),
    ];
    let indices = slots.map(|slot| generators.index_for_slot(slot));
    if indices.iter().all(Option::is_none) {
        return;
    }

    // `has_second_layer` is false when the level-1 theme resolves to no def, exactly as the
    // landscape mesh builder decides it — there, "resolves" means a texture; here it means a
    // generator, which is the same question asked of the same palette slot.
    let has_second = indices[1].is_some() || indices[2].is_some();
    let blends = map.theme_blends(x, y, has_second);

    let world = (origin.0 + x, origin.1 + y);
    // The draw counter, threaded through both functions as a `long*`. It is masked to five
    // bits by `GetRandomDisplacement`, so a cell has only 32 distinct random values however
    // many objects it grows — which is why Fable's grass clumps.
    //
    // UNVERIFIED: the counter is passed in a register and never appears in the decompiled
    // bodies. That it is incremented immediately before each draw is visible
    // (`*counter = *counter + 1` precedes all four draws in `AddObjectsFromLayerElement`);
    // that it starts at zero once per cell, and that the theme draw below increments it too,
    // are the reading, not the reading of the code.
    let mut counter = 0i32;

    let mut layer = 0usize;
    loop {
        let mut any_theme_has_layer = false;
        let mut element = 0usize;

        loop {
            let mut any_theme_has_element = false;

            counter += 1;
            let pick = (table.get(world.0, world.1, counter) * 255.0).floor() as i32;
            let theme = if pick <= blends[0] as i32 {
                0
            } else if pick <= blends[0] as i32 + blends[1] as i32 {
                1
            } else {
                2
            };

            if let Some(index) = indices[theme] {
                let generator = generators.get(index);
                if layer < generator.layers.len() {
                    let grid = &generator.grids[layer];
                    if element < grid.element_count(x, y) {
                        add_objects_from_layer_element(
                            map,
                            generators,
                            index,
                            layer,
                            grid.element(x, y, element),
                            blends[theme],
                            origin,
                            table,
                            (x, y),
                            &mut counter,
                            out,
                        );
                    }
                }
            }

            for index in indices.into_iter().flatten() {
                let generator = generators.get(index);
                if layer < generator.layers.len() {
                    any_theme_has_layer = true;
                    if element < generator.grids[layer].element_count(x, y) {
                        any_theme_has_element = true;
                    }
                }
            }

            element += 1;
            if !any_theme_has_element {
                break;
            }
        }

        layer += 1;
        if !any_theme_has_layer {
            break;
        }
    }
}

/// `AddObjectsFromLayerElement` (`engine_local_detail_cache.cpp:3256`) — one placement point
/// into at most one object.
///
/// Four draws, in this order: the object, its scale, the slope rejection, its rotation. Each
/// is preceded by an increment of the shared counter, so the sequence is fixed by the loop
/// structure above and not by which branches are taken.
#[allow(clippy::too_many_arguments)]
fn add_objects_from_layer_element(
    map: &LandscapeMap,
    generators: &GeneratorSet,
    generator_index: u16,
    layer_index: usize,
    element: (f32, f32),
    blend: u8,
    origin: (i32, i32),
    table: &DisplacementTable,
    cell: (i32, i32),
    counter: &mut i32,
    out: &mut Vec<PlacedObject>,
) {
    // `origin` reaches this function only through the random draws below.
    let generator = generators.get(generator_index);
    let layer = &generator.layers[layer_index];
    let world_cell = (origin.0 + cell.0, origin.1 + cell.1);
    let mut draw = || {
        *counter += 1;
        table.get(world_cell.0, world_cell.1, *counter)
    };

    // 1. Which object. An empty layer selects nothing.
    let Some(object) = layer.select(draw()) else {
        return;
    };

    // The ground has to want this object here. Zero on every shipped object, so in practice
    // this only rejects a theme with no weight at all.
    if !(object.theme_blend_threshold < blend as f32 / 255.0) {
        return;
    }

    // 2. Its scale, `(Scale + (2·rand − 1)·ScaleRandomElement) * 0.01`.
    let scale = (object.scale + (2.0 * draw() - 1.0) * object.scale_random_element)
        * MESH_UNITS_PER_WORLD_UNIT;

    // Map-local, not world — see `place_map`. The engine adds `WorldPosX`/`WorldPosY` here
    // because its terrain carries the same offset; ours does not.
    let position = (cell.0 as f32 + element.0, cell.1 as f32 + element.1);
    let normal = interpolated_map_normal(map, position.0, position.1);

    // 3. The slope rejection. `SlopeFadeStart >= SlopeFadeEnd` disables it; otherwise the
    // object survives with probability `clamp((n.z − start) / (end − start))`, so grass at
    // 0.80..0.90 thins out and then stops as the ground steepens.
    let survives = if object.slope_fade_start < object.slope_fade_end {
        ((normal[2] - object.slope_fade_start)
            / (object.slope_fade_end - object.slope_fade_start))
            .clamp(0.0, 1.0)
    } else {
        1.0
    };
    if survives <= draw() {
        return;
    }

    // 4. Its rotation about Z.
    let angle = draw();
    let (sin, cos) = fast_sin_cos(angle);

    let height = interpolated_height(map, position.0, position.1);
    let transform = object_matrix(scale, sin, cos, object, normal, [
        position.0, position.1, height,
    ]);

    out.push(PlacedObject {
        generator: generator_index,
        object_type: object.object_type as u16,
        transform,
        scale,
        normal,
        angle,
    });
}

/// The rotation the engine builds, optionally tilted onto the ground.
///
/// Untilted it is a Z rotation scaled uniformly, written as `CMatrix3x4`'s rows — which are
/// the columns of a column-major matrix (AGENTS.md §3.11):
///
/// ```text
/// row 0 = ( s·cos,  s·sin, 0)      row 2 = (0, 0, s)
/// row 1 = (−s·sin,  s·cos, 0)      row 3 = position
/// ```
///
/// `TiltToSlope` post-multiplies by the basis `(u, w, n)` built from the landscape normal —
/// `u = normalize(Y × n)`, `w = normalize(n × u)` — which sends the object's +Z onto the
/// ground normal and leaves its scale alone.
fn object_matrix(
    scale: f32,
    sin: f32,
    cos: f32,
    object: &LayerObject,
    normal: [f32; 3],
    position: [f32; 3],
) -> [[f32; 4]; 4] {
    let rows = [
        [scale * cos, scale * sin, 0.0],
        [-scale * sin, scale * cos, 0.0],
        [0.0, 0.0, scale],
    ];

    let rows = if object.tilt_to_slope {
        let n = normal;
        // `(n.z, 0, −n.x)` is `Y × n`, degenerate only where the ground faces exactly along
        // Y, which a landscape normal never does — its Z component is dominant.
        let u = normalise([n[2], 0.0, -n[0]]);
        let w = normalise(cross(n, u));
        rows.map(|r| {
            [
                r[0] * u[0] + r[1] * w[0] + r[2] * n[0],
                r[0] * u[1] + r[1] * w[1] + r[2] * n[1],
                r[0] * u[2] + r[1] * w[2] + r[2] * n[2],
            ]
        })
    } else {
        rows
    };

    [
        [rows[0][0], rows[0][1], rows[0][2], 0.0],
        [rows[1][0], rows[1][1], rows[1][2], 0.0],
        [rows[2][0], rows[2][1], rows[2][2], 0.0],
        [position[0], position[1], position[2], 1.0],
    ]
}

/// `GFastCosTable` sampled at `angle * 1024`, linearly interpolated, with sine 256 entries
/// earlier.
///
/// The table is real: the engine indexes `GFastCosTable[i]` and `GFastCosTable[i + 1]` and
/// lerps between them (`engine_local_detail_cache.cpp:3423-3437`), which is a hair coarser
/// than `sin`/`cos` and is the value the object was actually placed with.
pub fn fast_sin_cos(angle: f32) -> (f32, f32) {
    let scaled = angle * COS_TABLE_SIZE as f32;
    let index = scaled.floor();
    let fraction = scaled - index;
    let index = index as i32;

    let cos_at = |i: i32| {
        let entry = |k: i32| {
            let k = k.rem_euclid(COS_TABLE_SIZE);
            (k as f32 * std::f32::consts::TAU / COS_TABLE_SIZE as f32).cos()
        };
        entry(i) * (1.0 - fraction) + entry(i + 1) * fraction
    };

    (cos_at(index - COS_TABLE_SIZE / 4), cos_at(index))
}

/// `CEngineMap::PeekInterpolatedMapNormal` (`fableengine/engine_world_map.cpp:1688`) —
/// bilinear over the four surrounding vertex normals, then normalised.
pub fn interpolated_map_normal(map: &LandscapeMap, x: f32, y: f32) -> [f32; 3] {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (x0, y0) = (x0 as i32, y0 as i32);

    let n00 = vertex_normal(map, x0, y0);
    let n10 = vertex_normal(map, x0 + 1, y0);
    let n01 = vertex_normal(map, x0, y0 + 1);
    let n11 = vertex_normal(map, x0 + 1, y0 + 1);

    let mut out = [0.0f32; 3];
    for axis in 0..3 {
        out[axis] = bilinear(n00[axis], n10[axis], n01[axis], n11[axis], fx, fy);
    }
    normalise(out)
}

/// The ground height under a point, bilinear over the four surrounding vertices.
///
/// The corners come from `PeekLandscapeHeight`, quantised to 1/128 before the interpolation —
/// so this lands on exactly the surface the landscape mesh draws.
pub fn interpolated_height(map: &LandscapeMap, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (x0, y0) = (x0 as i32, y0 as i32);

    bilinear(
        map.height_at(x0, y0),
        map.height_at(x0 + 1, y0),
        map.height_at(x0, y0 + 1),
        map.height_at(x0 + 1, y0 + 1),
        fx,
        fy,
    )
}

fn bilinear(v00: f32, v10: f32, v01: f32, v11: f32, fx: f32, fy: f32) -> f32 {
    v00 * (1.0 - fx) * (1.0 - fy) + v10 * fx * (1.0 - fy) + v01 * (1.0 - fx) * fy + v11 * fx * fy
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length > 0.0 {
        [v[0] / length, v[1] / length, v[2] / length]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table has to agree with the trigonometry it was built from, or every object is
    /// rotated wrongly by a little.
    #[test]
    fn the_cosine_table_is_cosine() {
        for step in 0..64 {
            let angle = step as f32 / 64.0;
            let (sin, cos) = fast_sin_cos(angle);
            let radians = angle * std::f32::consts::TAU;
            assert!((cos - radians.cos()).abs() < 1e-4, "cos at {angle}: {cos}");
            assert!((sin - radians.sin()).abs() < 1e-4, "sin at {angle}: {sin}");
        }
    }

    /// The rotation must be a rotation: orthonormal after the scale is divided out, and
    /// right-handed. A sign error here would mirror every object.
    #[test]
    fn the_object_matrix_is_a_scaled_rotation() {
        let object = LayerObject {
            scale: 1.0,
            scale_random_element: 0.0,
            slope_fade_start: 0.0,
            slope_fade_end: 0.0,
            probability: 1.0,
            theme_blend_threshold: 0.0,
            tilt_to_slope: false,
            object_type: 0,
        };

        for step in 0..16 {
            let (sin, cos) = fast_sin_cos(step as f32 / 16.0);
            let scale = 0.37;
            let m = object_matrix(scale, sin, cos, &object, [0.0, 0.0, 1.0], [1.0, 2.0, 3.0]);

            let column = |i: usize| [m[i][0], m[i][1], m[i][2]];
            for i in 0..3 {
                let c = column(i);
                let length = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
                assert!((length - scale).abs() < 1e-5, "column {i} length {length}");
            }
            let handedness = cross(column(0), column(1));
            let z = column(2);
            for axis in 0..3 {
                assert!(
                    (handedness[axis] - z[axis] * scale).abs() < 1e-4,
                    "x cross y should be z: {handedness:?} vs {z:?}",
                );
            }
            assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0]);
        }
    }

    /// Tilting sends the object's +Z onto the ground normal, and does not change its size.
    #[test]
    fn tilting_stands_the_object_on_the_slope() {
        let object = LayerObject {
            scale: 1.0,
            scale_random_element: 0.0,
            slope_fade_start: 0.0,
            slope_fade_end: 0.0,
            probability: 1.0,
            theme_blend_threshold: 0.0,
            tilt_to_slope: true,
            object_type: 0,
        };

        let normal = normalise([0.3, -0.2, 0.9]);
        let (sin, cos) = fast_sin_cos(0.125);
        let m = object_matrix(1.0, sin, cos, &object, normal, [0.0; 3]);

        for axis in 0..3 {
            assert!(
                (m[2][axis] - normal[axis]).abs() < 1e-5,
                "object +Z should be the ground normal: {:?} vs {normal:?}",
                m[2],
            );
        }
        for i in 0..3 {
            let c = [m[i][0], m[i][1], m[i][2]];
            let length = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            assert!((length - 1.0).abs() < 1e-5, "column {i} length {length}");
        }
    }

    /// Bilinear interpolation must reproduce its corners exactly, or objects at cell corners
    /// float above or sink into the ground the landscape pass draws.
    #[test]
    fn bilinear_reproduces_its_corners() {
        assert_eq!(bilinear(1.0, 2.0, 3.0, 4.0, 0.0, 0.0), 1.0);
        assert_eq!(bilinear(1.0, 2.0, 3.0, 4.0, 1.0, 0.0), 2.0);
        assert_eq!(bilinear(1.0, 2.0, 3.0, 4.0, 0.0, 1.0), 3.0);
        assert_eq!(bilinear(1.0, 2.0, 3.0, 4.0, 1.0, 1.0), 4.0);
        assert_eq!(bilinear(1.0, 2.0, 3.0, 4.0, 0.5, 0.5), 2.5);
    }
}
