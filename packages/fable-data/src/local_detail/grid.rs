//! `CLocalDetailPlacementGrid` — where local detail objects may stand.
//!
//! One grid per `(generator, layer)`. It holds a **32 × 32 cell tile of scattered points**
//! that the world repeats in both axes, generated once by dart throwing with a minimum
//! spacing. `AddObjectsFromBlendedThemes` walks a map cell's points through
//! [`PlacementGrid::element_count`] / [`PlacementGrid::element`] and turns each into at most
//! one object.
//!
//! Two properties of the tile are worth stating before anyone looks at a screenshot, because
//! both are the original's behaviour and neither is a bug to fix:
//!
//! - **It repeats every 32 world cells.** `PeekElementCount` and `PeekElement` mask their
//!   coordinates with `& 0x1f` (`engine_local_detail_theme.cpp:1650,1683`), so a map is
//!   tiled with copies of this one grid. What breaks up the repetition on screen is the
//!   theme map, the per-cell random draws and the terrain — not the point pattern.
//! - **It is toroidal.** `MapPosToElementSpace` (`:786`) wraps differences at ±16 with a
//!   period of 32, so points near one edge keep their spacing from points near the opposite
//!   edge and the tile seams are invisible.

use super::rng::Rng;

/// Cells along each axis of the tile — `ElementIndexGrid[32][32]`.
pub const GRID_SIZE: i32 = 32;

const CELLS: usize = (GRID_SIZE * GRID_SIZE) as usize;

/// Consecutive rejected candidates that end the dart throwing.
///
/// `BuildElementGrid` seeds a counter with `0x100`, decrements it on every attempt, gives up
/// when a *rejected* attempt sees zero, and resets it to `0x100` on every accepted one
/// (`engine_local_detail_theme.cpp:1357`, `:1593`). So it is 256 failures in a row since the
/// last success, not a budget for the whole grid.
const GIVE_UP_AFTER: u32 = 0x100;

/// A tile of scattered points, indexed by cell.
///
/// Storage is the original's: a per-cell prefix offset into one flat array of packed
/// elements, so a cell's points are contiguous and the whole grid is two allocations.
pub struct PlacementGrid {
    /// `ElementIndexGrid`, plus a terminator.
    ///
    /// The engine keeps 1024 entries and reconstructs the end of the last cell from
    /// `Elements.size()`, which is why `PeekElementCount` has three branches: the successor
    /// of `[x][y]` is `[x + 1][y]`, the successor of `[31][y]` is `[0][y + 1]`, and the
    /// successor of `[31][31]` is the end of the array. That is a prefix sum over cells
    /// ordered with **x fastest** — confirmed by the fill loop, whose inner step advances the
    /// index pointer by one whole row and whose outer step advances it by one entry
    /// (`:1487-1560`). Storing the terminator explicitly makes the three branches one
    /// subtraction without changing any of them.
    starts: Box<[u32]>,
    /// Packed `(x, y)` offsets, one `u16` each — see [`PlacementGrid::element`].
    elements: Box<[u16]>,
}

/// Cell index for a tile coordinate, with the tile's wrap applied.
fn cell_of(v: i32) -> usize {
    (v & (GRID_SIZE - 1)) as usize
}

fn flat(x: i32, y: i32) -> usize {
    cell_of(y) * GRID_SIZE as usize + cell_of(x)
}

/// Shortest signed difference on a tile of period 32, matching `MapPosToElementSpace`
/// (`engine_local_detail_theme.cpp:786`): `> 16` folds down, `< -16` folds up, and `16`
/// itself is left alone.
fn wrap_delta(d: f32) -> f32 {
    if d > 16.0 {
        d - 32.0
    } else if d < -16.0 {
        d + 32.0
    } else {
        d
    }
}

impl PlacementGrid {
    /// How many points cell `(x, y)` carries — `PeekElementCount` (`:1683`).
    pub fn element_count(&self, x: i32, y: i32) -> usize {
        let k = flat(x, y);
        (self.starts[k + 1] - self.starts[k]) as usize
    }

    /// The `i`-th point of cell `(x, y)`, as an offset **from the cell** in `[-0.5, 0.5]`.
    ///
    /// `PeekElement` (`:1650`) decodes the packed `u16` as
    /// `(lo / 255 - 0.5, hi / 255 - 0.5)`, and `AddObjectsFromLayerElement` adds the cell's
    /// world coordinates to it — so a point may sit just inside a neighbouring cell.
    pub fn element(&self, x: i32, y: i32, i: usize) -> (f32, f32) {
        let packed = self.elements[self.starts[flat(x, y)] as usize + i];
        (
            (packed & 0xff) as f32 / 255.0 - 0.5,
            (packed >> 8) as f32 / 255.0 - 0.5,
        )
    }

    /// Points in the whole tile — `GetTotalElementCount` (`:2601`).
    pub fn total_elements(&self) -> usize {
        self.elements.len()
    }

    /// Build one grid per layer, in order, sharing each layer's points with the next.
    ///
    /// `spacings[l]` is layer `l`'s `SpacingFromLayer`, so `spacings[l][j]` is how far layer
    /// `l`'s points must stay from layer `j`'s — including `spacings[l][l]`, its own. The
    /// engine expresses this as a chain: a layer's `CLocalDetailPlacementGrid` holds
    /// `LayerSpacing = SpacingFromLayer[0..=l]`, its `BaseLayer` is layer `l - 1`'s grid, and
    /// `BuildElementGrid` walks that chain testing each ancestor's points against
    /// `LayerSpacing[l]`, `LayerSpacing[l - 1]`, … — always **this** layer's numbers, never
    /// the ancestor's (`:1102` for the chain, `:1440` for the walk).
    ///
    /// Building them together rather than one at a time is what makes that expressible; it
    /// is also what `SetupPlacementGrids` (`engine_local_detail_generator.cpp:1360`) is doing
    /// when it shares one grid between generators whose spacings agree.
    pub fn build_layers(spacings: &[Vec<f32>]) -> Vec<PlacementGrid> {
        let mut grids = Vec::with_capacity(spacings.len());
        let mut previous: Vec<PointSet> = Vec::with_capacity(spacings.len());

        for layer_spacings in spacings {
            let points = throw_darts(layer_spacings, &previous);
            grids.push(pack(&points));
            previous.push(points);
        }

        grids
    }
}

/// Points of one layer, bucketed per cell so a proximity query touches a handful of them.
///
/// The engine uses a quadtree per cell (`NLocalDetailPlacementGrid::CQuadTree`), whose leaf
/// test is `dx*dx + dy*dy < spacing*spacing` against the *candidate's* spacing
/// (`engine_local_detail_theme.cpp:885`) — the radius a point was inserted with only chooses
/// how deep it sits, and every node is tested on the way down, so no point can be missed.
/// A uniform bucket grid answers the same question with the same predicate.
struct PointSet {
    buckets: Vec<Vec<(f32, f32)>>,
    len: usize,
}

impl PointSet {
    fn new() -> PointSet {
        PointSet {
            buckets: vec![Vec::new(); CELLS],
            len: 0,
        }
    }

    fn push(&mut self, x: f32, y: f32) {
        self.buckets[flat(x as i32, y as i32)].push((x, y));
        self.len += 1;
    }

    /// Is `(x, y)` within `spacing` of any point? — `CQuadTree::ClipObject` returns *false*
    /// when it is, and dart throwing rejects the candidate.
    fn occupied(&self, x: f32, y: f32, spacing: f32) -> bool {
        let reach = spacing.ceil() as i32 + 1;
        let (cx, cy) = (x as i32, y as i32);
        let limit = spacing * spacing;

        for dy in -reach..=reach {
            for dx in -reach..=reach {
                for &(px, py) in &self.buckets[flat(cx + dx, cy + dy)] {
                    let ddx = wrap_delta(x - px);
                    let ddy = wrap_delta(y - py);
                    if ddx * ddx + ddy * ddy < limit {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// `CLocalDetailPlacementGrid::BuildElementGrid` (`engine_local_detail_theme.cpp:1357`).
///
/// ```text
/// seed = 0
/// loop {
///     x, y = two draws in [0, 32)
///     reject if within spacings[l] of layer l's points, for l = own, own-1, … 0
///     accept: record the point
///     stop after 256 consecutive rejections
/// }
/// ```
///
/// The two draws happen before any test, so the rejection order does not affect the sequence
/// and cannot affect the result.
fn throw_darts(spacings: &[f32], previous: &[PointSet]) -> PointSet {
    let own = spacings.len() - 1;
    let mut rng = Rng::new();
    let mut points = PointSet::new();
    let mut budget = GIVE_UP_AFTER;

    loop {
        let x = rng.next_grid_coord();
        let y = rng.next_grid_coord();
        budget -= 1;

        let clipped = points.occupied(x, y, spacings[own])
            || previous
                .iter()
                .enumerate()
                .any(|(layer, set)| set.occupied(x, y, spacings[layer]));

        if clipped {
            if budget == 0 {
                return points;
            }
            continue;
        }

        points.push(x, y);
        budget = GIVE_UP_AFTER;
    }
}

/// Flatten a layer's points into the cell-ordered prefix layout, packing each as two bytes.
///
/// A point belongs to the cell it is *nearest* to, not the one it is inside:
/// `BuildElementGrid`'s accept path takes the cell with a truncation and then subtracts it
/// with `MapPosToElementSpace`'s ±16 wrap (`:1593-1615`). The FPU arguments are invisible
/// there, but the storage settles it — the encode is `floor((offset + 0.5) * 255)` clamped to
/// a byte and the decode is `byte / 255 - 0.5`, so the offset has to land in `[-0.5, 0.5]`.
/// Under `cell = floor(x)` half of every grid would clamp to the cell's far edge and the ±16
/// wrap would never fire; under `cell = floor(x + 0.5)` the round trip is exact and the wrap
/// is exactly what carries `x` near 31.5 back to cell 0 with an offset of −0.5.
fn pack(points: &PointSet) -> PlacementGrid {
    let mut by_cell: Vec<Vec<u16>> = vec![Vec::new(); CELLS];

    for bucket in &points.buckets {
        for &(x, y) in bucket {
            let cell_x = (x + 0.5) as i32;
            let cell_y = (y + 0.5) as i32;
            let offset_x = wrap_delta(x - cell_of(cell_x) as f32);
            let offset_y = wrap_delta(y - cell_of(cell_y) as f32);
            by_cell[flat(cell_x, cell_y)].push(pack_offset(offset_x, offset_y));
        }
    }

    let mut starts = Vec::with_capacity(CELLS + 1);
    let mut elements = Vec::with_capacity(points.len);
    for cell in &by_cell {
        starts.push(elements.len() as u32);
        elements.extend_from_slice(cell);
    }
    starts.push(elements.len() as u32);

    debug_assert!(
        elements.len() <= u16::MAX as usize,
        "a placement grid of {} elements overflows the engine's u16 element index",
        elements.len(),
    );

    PlacementGrid {
        starts: starts.into_boxed_slice(),
        elements: elements.into_boxed_slice(),
    }
}

fn pack_offset(x: f32, y: f32) -> u16 {
    let byte = |v: f32| ((v + 0.5) * 255.0).floor().clamp(0.0, 255.0) as u16;
    byte(x) | (byte(y) << 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(spacing: f32) -> PlacementGrid {
        PlacementGrid::build_layers(&[vec![spacing]]).pop().unwrap()
    }

    /// The counts a spacing produces are the subsystem's headline numbers: they set how much
    /// grass a level grows and how much geometry the pass draws. They are a pure function of
    /// the PRNG and the rejection rule, so pinning them here catches any change to either.
    #[test]
    fn element_counts_per_spacing() {
        // (spacing, elements in the 32x32 tile). The extremes are the shipped data's:
        // 0.12 is `LOCAL_DETAIL_BRIGHTWOOD_TALLGRASS`'s grass, 6.5 is
        // `LOCAL_DETAIL_BRIGHTWOOD_MIXEDWOOD`'s oaks.
        for (spacing, expected) in [
            (0.12f32, 37241usize),
            (0.25, 8636),
            (0.5, 2285),
            (1.0, 584),
            (2.5, 88),
            (4.5, 32),
            (6.5, 15),
        ] {
            let g = grid(spacing);
            assert_eq!(
                g.total_elements(),
                expected,
                "spacing {spacing} produced {} elements",
                g.total_elements(),
            );
        }
    }

    /// Every point must decode inside the cell-relative range the storage can represent, or
    /// the packing is lying about where things stand.
    #[test]
    fn offsets_stay_within_half_a_cell() {
        let g = grid(0.5);
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                for i in 0..g.element_count(x, y) {
                    let (ox, oy) = g.element(x, y, i);
                    assert!((-0.5..=0.5).contains(&ox), "({x},{y})[{i}] x offset {ox}");
                    assert!((-0.5..=0.5).contains(&oy), "({x},{y})[{i}] y offset {oy}");
                }
            }
        }
    }

    /// The per-cell counts must partition the element array — the invariant the prefix layout
    /// exists to provide, and the one `PeekElementCount`'s three branches reconstruct.
    #[test]
    fn cell_counts_partition_the_elements() {
        let g = grid(0.25);
        let mut total = 0;
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                total += g.element_count(x, y);
            }
        }
        assert_eq!(total, g.total_elements());
        assert!(total > 0);
    }

    /// Coordinates are masked, not bounds-checked: a map is tiled with copies of one grid.
    #[test]
    fn the_tile_repeats_every_32_cells() {
        let g = grid(1.0);
        for (x, y) in [(0, 0), (5, 7), (31, 31)] {
            assert_eq!(g.element_count(x, y), g.element_count(x + 32, y));
            assert_eq!(g.element_count(x, y), g.element_count(x, y - 32));
            if g.element_count(x, y) > 0 {
                assert_eq!(g.element(x, y, 0), g.element(x + 64, y + 32, 0));
            }
        }
    }

    /// The minimum spacing has to hold across the tile seam as well as inside it, which is
    /// what the ±16 wrap buys. Measured on the decoded world positions, so it also exercises
    /// the cell/offset split.
    #[test]
    fn points_respect_their_spacing_across_the_seam() {
        let spacing = 1.5f32;
        let g = grid(spacing);

        let mut world = Vec::new();
        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                for i in 0..g.element_count(x, y) {
                    let (ox, oy) = g.element(x, y, i);
                    world.push((x as f32 + ox, y as f32 + oy));
                }
            }
        }

        // The byte packing quantises each axis by 1/255, so two points can end up marginally
        // closer than the spacing they were accepted at.
        let tolerance = spacing - 2.0 / 255.0;
        for (i, &(ax, ay)) in world.iter().enumerate() {
            for &(bx, by) in &world[i + 1..] {
                let dx = wrap_delta(ax - bx);
                let dy = wrap_delta(ay - by);
                let d = (dx * dx + dy * dy).sqrt();
                assert!(d >= tolerance, "({ax},{ay}) and ({bx},{by}) are {d} apart");
            }
        }
    }

    /// A layer keeps its distance from the layers below it, at the distance *it* names.
    /// This is the mechanism that puts saplings between trees rather than inside them.
    ///
    /// Note which number is which: `SpacingFromLayer[j]` is the distance from layer `j`, so
    /// layer 1's `[3.0, 4.0]` means 3.0 from layer 0 and 4.0 from its own points. Reading
    /// those the other way round is the obvious mistake, and `BuildElementGrid`'s chain walk
    /// settles it — it indexes `LayerSpacing[LayerID]` for its own tree and counts *down*
    /// through the ancestors.
    #[test]
    fn later_layers_avoid_earlier_ones() {
        // The shape `LOCAL_DETAIL_BRIGHTWOOD_BIRCH_BRACKEN`'s first two layers use.
        let grids = PlacementGrid::build_layers(&[vec![4.5], vec![3.0, 4.0]]);

        let positions = |g: &PlacementGrid| {
            let mut v = Vec::new();
            for y in 0..GRID_SIZE {
                for x in 0..GRID_SIZE {
                    for i in 0..g.element_count(x, y) {
                        let (ox, oy) = g.element(x, y, i);
                        v.push((x as f32 + ox, y as f32 + oy));
                    }
                }
            }
            v
        };

        let base = positions(&grids[0]);
        let upper = positions(&grids[1]);
        assert!(!base.is_empty() && !upper.is_empty());

        for &(ax, ay) in &upper {
            for &(bx, by) in &base {
                let dx = wrap_delta(ax - bx);
                let dy = wrap_delta(ay - by);
                let d = (dx * dx + dy * dy).sqrt();
                assert!(d >= 3.0 - 2.0 / 255.0, "layer 1 point {d} from a layer 0 point");
            }
        }

        for (i, &(ax, ay)) in upper.iter().enumerate() {
            for &(bx, by) in &upper[i + 1..] {
                let dx = wrap_delta(ax - bx);
                let dy = wrap_delta(ay - by);
                let d = (dx * dx + dy * dy).sqrt();
                assert!(d >= 4.0 - 2.0 / 255.0, "two layer 1 points are {d} apart");
            }
        }
    }
}
