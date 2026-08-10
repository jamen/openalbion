//! The engine's random number generator, and the displacement table local detail draws from.
//!
//! Every random number in the local detail generator comes from one five-instruction PRNG.
//! It is worth being exact about it: placement, object choice, scale, orientation and the
//! slope rejection are all functions of this sequence, so reproducing the game's foliage
//! *exactly* reduces to reproducing this file exactly.

/// `GFROR13` — rotate right by 13 (`bbblibrary/lib_global_tools.cpp:1133`):
///
/// ```c
/// long __fastcall GFROR13(ulong param_1)
/// {
///   return param_1 >> 0xd | param_1 << 0x13;
/// }
/// ```
pub fn ror13(x: u32) -> u32 {
    x.rotate_right(13)
}

/// One step of the engine's global PRNG: `seed = ror13(seed * 0x24a1 + 0x24df)`.
///
/// `GFRandom` (`lib_global_tools.cpp:1145`) is this step followed by `% range`, and every
/// other caller in the library open-codes the same two constants. Ghidra spells the additive
/// constant three different ways depending on which symbol happens to live at that address —
/// `s_other_000024dc + 3`, `&DAT_000024df`, `s_const_reference_000024da + 5` — but they are
/// one number, `0x24df`, and the local detail generator uses all three spellings.
pub fn next_seed(seed: u32) -> u32 {
    ror13(seed.wrapping_mul(0x24a1).wrapping_add(0x24df))
}

/// A seed being stepped through the sequence above.
///
/// Starts at zero. Both callers in this module — [`DisplacementTable::build`] and the
/// placement grid's dart throwing — begin from a zeroed local, which is what makes the whole
/// subsystem reproducible from nothing but the defs.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rng {
    seed: u32,
}

impl Rng {
    pub fn new() -> Rng {
        Rng { seed: 0 }
    }

    /// Advance and return the raw 32-bit word.
    pub fn next_u32(&mut self) -> u32 {
        self.seed = next_seed(self.seed);
        self.seed
    }

    /// Advance and return `fmod(word, modulus) / 65536`, the shape both call sites use.
    ///
    /// The word is converted to `float` **before** the `fmod`, exactly as the decomp does
    /// (it stores to a `float` local and then calls `_CIfmod`), so words above 2²⁴ lose their
    /// low bits and the result is coarsely quantised. That is faithful, and it is why the
    /// generator can get away with a table of only 32³ values.
    ///
    /// The conversion is **unsigned**. Ghidra types `GFROR13`'s result as `long` and shows a
    /// signed conversion, which would put this in (−1, 1) — but `AddObjectsFromLayerElement`
    /// indexes `ObjectSelectionTable[round(rand * 32 − 0.5)]` with no lower bound check, so a
    /// negative draw would read up to 128 bytes before the array on every level in the game.
    /// `GFRandom` also takes the result straight into an unsigned `%`. Unsigned is the only
    /// reading that is not a shipped out-of-bounds read.
    fn next_scaled(&mut self, modulus: f32) -> f32 {
        let word = self.next_u32() as f32;
        (word % modulus) * (1.0 / 65536.0)
    }

    /// A value in `[0, 1)` — `fmod(word, 65536) / 65536`.
    pub fn next_unit(&mut self) -> f32 {
        self.next_scaled(65536.0)
    }

    /// A value in `[0, 32)` — `fmod(word, 32 * 65536) / 65536`.
    ///
    /// The modulus is not visible in the decomp (`_CIfmod`'s arguments are passed on the FPU
    /// stack), but it is forced: `BuildElementGrid` uses the result to index a 32 × 32 grid
    /// through `element_grid[(int)x][(int)y]`, and the `1 / 65536` scale is shared with
    /// [`Rng::next_unit`]. Any other modulus puts every element in one cell.
    pub fn next_grid_coord(&mut self) -> f32 {
        self.next_scaled(32.0 * 65536.0)
    }
}

/// Cells along each axis of the displacement table, and the modulus every index takes.
pub const DISPLACEMENT_SIZE: usize = 32;

/// `CEngineLocalDetailGenerator::CDisplacementTable` — `float Table[32][32][32]`.
///
/// This is the generator's whole source of per-object randomness.
/// `GetRandomDisplacement(x, y, z)` is a bare
/// `Table[x & 0x1f][y & 0x1f][z & 0x1f]` (`engine_local_detail_generator.cpp:2190`), so the
/// objects placed on a cell depend only on that cell's **world** coordinates and a draw
/// counter. Two consequences worth knowing before looking at a screenshot:
///
/// - the entire pattern of species, scales and orientations repeats every 32 world cells;
/// - a cell's objects do not depend on its neighbours, which is what lets the engine
///   regenerate one patch without regenerating the map.
///
/// The table is only allocated when the generator is constructed with dynamic update enabled
/// (`:84`); retail streams a finished cache out of the static map instead, exactly as it
/// streams landscape patches. We build it, for the same reason we ported
/// `CEngineLandscapeMeshBuilder`.
pub struct DisplacementTable {
    values: Box<[f32]>,
}

impl DisplacementTable {
    /// Fill the table from a zeroed seed, in the constructor's own iteration order.
    ///
    /// The loop nest at `engine_local_detail_generator.cpp:186-210` walks byte offsets, and
    /// the strides name the axes: the innermost loop steps 4 bytes (the last index), the
    /// middle loop steps `0x1000` = 4096 bytes (the *first* index, one 32×32 plane), and the
    /// outer loop steps `0x80` = 128 bytes (the middle index, one row). So the fill order is
    ///
    /// ```text
    /// for y in 0..32 { for x in 0..32 { for z in 0..32 { Table[x][y][z] = next(); } } }
    /// ```
    ///
    /// — middle axis outermost. Getting this wrong permutes the table without changing its
    /// contents, which is exactly the kind of error a screenshot cannot catch.
    pub fn build() -> DisplacementTable {
        const N: usize = DISPLACEMENT_SIZE;
        let mut values = vec![0.0f32; N * N * N].into_boxed_slice();
        let mut rng = Rng::new();

        for y in 0..N {
            for x in 0..N {
                for z in 0..N {
                    values[(x * N + y) * N + z] = rng.next_unit();
                }
            }
        }

        DisplacementTable { values }
    }

    /// `CEngineLocalDetailGenerator::GetRandomDisplacement` — `Table[x & 31][y & 31][z & 31]`.
    pub fn get(&self, x: i32, y: i32, z: i32) -> f32 {
        const N: i32 = DISPLACEMENT_SIZE as i32;
        let x = (x & (N - 1)) as usize;
        let y = (y & (N - 1)) as usize;
        let z = (z & (N - 1)) as usize;
        self.values[(x * DISPLACEMENT_SIZE + y) * DISPLACEMENT_SIZE + z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rotate is the whole of `GFROR13`; a shift would be a different generator.
    #[test]
    fn ror13_is_a_rotate() {
        assert_eq!(ror13(0x0000_2000), 1);
        assert_eq!(ror13(1), 1 << 19);
        assert_eq!(ror13(0x1234_5678), (0x1234_5678u32 >> 13) | (0x1234_5678u32 << 19));
    }

    /// The sequence is chained through the seed, not restarted, and the first value from a
    /// zeroed seed is fixed by the additive constant alone.
    #[test]
    fn sequence_starts_at_the_additive_constant() {
        assert_eq!(next_seed(0), ror13(0x24df));

        let mut rng = Rng::new();
        let first = rng.next_u32();
        assert_eq!(first, ror13(0x24df));
        assert_eq!(rng.next_u32(), next_seed(first));
    }

    /// Every draw must be usable as a probability. If the conversion were signed, half of
    /// them would be negative and `ObjectSelectionTable[round(rand * 32 - 0.5)]` would index
    /// backwards out of the array.
    #[test]
    fn unit_draws_are_in_range() {
        let mut rng = Rng::new();
        for _ in 0..100_000 {
            let v = rng.next_unit();
            assert!((0.0..1.0).contains(&v), "{v} out of [0, 1)");
        }
    }

    /// The grid coordinate must reach every one of the 32 columns, or the placement grid
    /// collapses into a single cell.
    #[test]
    fn grid_coords_cover_the_whole_tile() {
        let mut rng = Rng::new();
        let mut seen = [false; 32];
        for _ in 0..100_000 {
            let v = rng.next_grid_coord();
            assert!((0.0..32.0).contains(&v), "{v} out of [0, 32)");
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "some columns are never drawn: {seen:?}");
    }

    /// The table is a permutation of one contiguous run of the sequence: 32768 draws, in the
    /// order the constructor's byte strides imply.
    #[test]
    fn displacement_table_matches_the_fill_order() {
        let table = DisplacementTable::build();
        let mut rng = Rng::new();

        // The first draw lands at [0][0][0], the second at [0][0][1] (innermost = last index).
        assert_eq!(table.get(0, 0, 0), rng.next_unit());
        assert_eq!(table.get(0, 0, 1), rng.next_unit());

        // After the innermost loop finishes, the middle loop steps a whole plane: [1][0][0].
        for _ in 2..32 {
            rng.next_unit();
        }
        assert_eq!(table.get(1, 0, 0), rng.next_unit());
    }

    /// Indices wrap rather than panicking — the accessor is `& 0x1f` on all three axes, and
    /// world coordinates are routinely larger than 32 and occasionally negative.
    #[test]
    fn displacement_indices_wrap() {
        let table = DisplacementTable::build();
        assert_eq!(table.get(33, 0, 0), table.get(1, 0, 0));
        assert_eq!(table.get(-1, 0, 0), table.get(31, 0, 0));
        assert_eq!(table.get(0, 0, 64), table.get(0, 0, 0));
    }

    /// A degenerate table would make every cell identical. This is a smoke test on the
    /// generator's quality, not on its exactness: 32768 draws should not collapse.
    #[test]
    fn displacement_table_is_not_degenerate() {
        let table = DisplacementTable::build();
        let mut buckets = [0usize; 10];
        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let v = table.get(x, y, z);
                    buckets[(v * 10.0) as usize % 10] += 1;
                }
            }
        }
        // 32768 draws over ten buckets: expect ~3277 each. Allow a wide margin; we are
        // ruling out collapse, not testing uniformity.
        for (i, &n) in buckets.iter().enumerate() {
            assert!(n > 1000, "bucket {i} has only {n} of 32768 draws: {buckets:?}");
        }
    }
}
