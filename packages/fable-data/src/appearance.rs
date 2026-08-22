//! `CRandomAppearanceMorph` — which meshes a creature's body is actually built from.
//!
//! A villager's `Graphic.BankIndex` names a **base body** — `MESH_BS_MALE_MIDDLE_UNCLOTHED_01`
//! and friends — and drawing that alone is why NPCs stand around in their undergarments. The
//! clothed creature is assembled from three *body-part* meshes chosen at random from the
//! creature def, and this module is the port of that choice.
//!
//! ## The parts replace the base body; they do not layer over it
//!
//! Measured from the meshes' own bounding boxes, which needs no oracle
//! (`MESH_BS_MALE_MIDDLE_*`, Z extents in mesh units):
//!
//! ```text
//! UNCLOTHED_01   Z  -0.7 .. 193.6   64 bones   materials: face hair torso legs mouth
//! HEAD_01        Z 156.8 .. 193.6   28 bones   materials: face hair mouth
//! TORSO_02       Z  85.8 .. 168.8   40 bones   materials: skin shirt
//! LEGS_02        Z   0.0 .. 114.7   16 bones   materials: Legs02
//! ```
//!
//! The three parts span the base body's full height between them and carry *clothed*
//! materials (`shirt`, `Legs02`) where the base carries bare ones (`torso`, `legs`). They are
//! a decomposition of the same body, not accessories — so a creature with body parts draws
//! its three parts **instead of** its base mesh, and drawing both would z-fight over the whole
//! silhouette. `CTCRandomAppearanceMorph::OnAppearanceDraw` returning `bool` fits that
//! reading (the component handles the draw and suppresses the default graphic), but its body
//! is too mangled to transcribe, so the geometry above is what this rests on.
//!
//! ## The choice is exact, not approximated
//!
//! `CRandomAppearanceMorph::GetRandomBodyParts` (`fablelib/random_appearance_morph.cpp:838`)
//! draws from **the same PRNG as local detail** (§3.13) — `seed = GFROR13(seed * 0x24a1 +
//! 0x24df)`, already ported in [`crate::local_detail::rng`] — and the seed is authored
//! per placement in the `.tng`:
//!
//! ```text
//! StartCTCRandomAppearanceMorph;
//! Seed -1949563995;
//! EndCTCRandomAppearanceMorph;
//! ```
//!
//! 604 of them across the shipped levels. So every villager's appearance is reproducible
//! exactly, from data we already parse.

use crate::local_detail::rng::next_seed;

/// `CRandomAppearanceMorph::BodyParts` is a three-element vector, and the draw loop is a
/// literal `do { ... } while (iVar8 != 3)`. Every `CCreatureDef` in retail agrees:
/// `num_body_parts` is 3 on all 231 of them.
pub const BODY_PART_COUNT: usize = 3;

/// Body part indices, in the order `GetRandomBodyParts` walks them — which is also the order
/// the text defs list them (`BODY_PART_HEAD`, `BODY_PART_TORSO`, `BODY_PART_LEGS`).
pub const HEAD: usize = 0;
pub const TORSO: usize = 1;
pub const LEGS: usize = 2;

/// The candidate meshes for each body part of one creature def.
///
/// `graphics.big` asset ids, exactly like `Graphic.BankIndex` (AGENTS.md §3.11) — 980 of 980
/// resolve to mesh-typed assets across retail's 127 creature defs that have any.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BodyPartSets {
    pub parts: [Vec<i32>; BODY_PART_COUNT],
}

impl BodyPartSets {
    pub fn is_empty(&self) -> bool {
        self.parts.iter().all(Vec::is_empty)
    }

    /// The mesh ids this creature is built from, for one placement's authored seed.
    ///
    /// Transcribed from `GetRandomBodyParts` (`random_appearance_morph.cpp:838`):
    ///
    /// ```c
    /// for (part = 0; part != 3; part++) {
    ///     if (BodyParts[part] is non-empty) {
    ///         count = (Mylast - Myfirst) / 0x1c;      // 28 = sizeof(CBodyPartMesh)
    ///         *seed = GFROR13(*seed * 0x24a1 + 0x24df);
    ///         index = (count != 0) ? *seed % count : 0;
    ///         mesh  = BodyParts[part][index].MeshIndex;
    ///     }
    /// }
    /// ```
    ///
    /// The advance sits **inside** the non-empty guard, so an empty part does not consume a
    /// draw — which is why this returns a `Vec` rather than a fixed array, and why the parts
    /// must be walked in order.
    ///
    /// `CBodyPartMesh::Priority` is read by nothing here: the pick is uniform over the list.
    /// It is `1` on all 980 shipped entries anyway.
    ///
    /// > **One draw is deliberately not made here, and it is logged in AGENTS.md §9.**
    /// > The engine precedes this loop with
    /// > `if (MaxTextureGroupID != 0) { advance; texture_group = seed % MaxTextureGroupID + 1; }`.
    /// > `MaxTextureGroupID` is a *derived* field — the running maximum of the group ids
    /// > passed to `AddTextureToMesh` — and those group ids reach us only through
    /// > `texture_morphs`, which `fable-defs` currently misparses (§3.16). Its **value** only
    /// > picks a texture group, which we do not implement; but whether it is zero decides
    /// > whether the sequence is shifted by one draw. In the shipped text defs the group id is
    /// > `0` on 650 of 784 calls and on every villager def, so no pre-draw is the right
    /// > reading for the common case; roughly twenty defs (hobbes, bandit officers, prophets,
    /// > guild apprentices, traders) use groups 1–3 and will pick a different variant than the
    /// > original until the parse is fixed. It changes *which* clothed body a creature gets,
    /// > never whether it gets one.
    pub fn choose(&self, seed: u32) -> Vec<i32> {
        let mut seed = seed;
        let mut chosen = Vec::with_capacity(BODY_PART_COUNT);
        for part in &self.parts {
            if part.is_empty() {
                continue;
            }
            seed = next_seed(seed);
            chosen.push(part[seed as usize % part.len()]);
        }
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sets(head: &[i32], torso: &[i32], legs: &[i32]) -> BodyPartSets {
        BodyPartSets {
            parts: [head.to_vec(), torso.to_vec(), legs.to_vec()],
        }
    }

    /// One part per non-empty list, in head/torso/legs order.
    #[test]
    fn one_mesh_per_populated_part() {
        let s = sets(&[1, 2, 3, 4], &[10, 11], &[20, 21, 22]);
        let chosen = s.choose(12345);
        assert_eq!(chosen.len(), 3);
        assert!([1, 2, 3, 4].contains(&chosen[0]));
        assert!([10, 11].contains(&chosen[1]));
        assert!([20, 21, 22].contains(&chosen[2]));
    }

    /// The seed is the whole of it: the same seed must give the same body, every run and
    /// every process. This is what makes a level's crowd reproducible rather than merely
    /// plausible.
    #[test]
    fn the_same_seed_gives_the_same_body() {
        let s = sets(&[1, 2, 3, 4], &[10, 11], &[20, 21, 22]);
        assert_eq!(s.choose(0xdead_beef), s.choose(0xdead_beef));
        assert_ne!(s.choose(1), s.choose(2), "different seeds should differ here");
    }

    /// An empty part consumes no draw, because the advance is inside the non-empty guard.
    /// If it did consume one, every part after it would shift and the whole crowd would be
    /// wrong in a way no screenshot would reveal.
    #[test]
    fn an_empty_part_consumes_no_draw() {
        let with_head = sets(&[1, 2, 3, 4], &[10, 11], &[20, 21, 22]);
        let without_head = sets(&[], &[10, 11], &[20, 21, 22]);

        let a = with_head.choose(7);
        let b = without_head.choose(7);
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 2);

        // Had the empty head still advanced the seed, `b` would have started one draw later
        // and could not match `a`'s torso and legs.
        let mut seed = next_seed(7); // the head draw `a` made and `b` did not
        seed = next_seed(seed);
        assert_eq!(b[0], [10, 11][next_seed(7) as usize % 2], "torso used the first draw");
        assert_eq!(a[1], [10, 11][seed as usize % 2], "and `a`'s torso used the second");
    }

    /// The seed arrives from the `.tng` as a signed integer and is routinely negative
    /// (`Seed -1949563995;`). The engine's is `unsigned long`, so the cast must be a
    /// reinterpretation — a saturating or absolute conversion would pick a different body.
    #[test]
    fn negative_tng_seeds_reinterpret_rather_than_clamp() {
        let s = sets(&[1, 2, 3, 4], &[10, 11], &[20, 21, 22]);
        let seed = -1_949_563_995i32;
        assert_eq!(s.choose(seed as u32), s.choose(2_345_403_301));
    }

    /// Every part is reachable: a list of four must not always yield the same mesh.
    #[test]
    fn the_draw_covers_every_candidate() {
        let s = sets(&[1, 2, 3, 4], &[], &[]);
        let mut seen = [false; 4];
        for seed in 0..2000u32 {
            let chosen = s.choose(seed);
            seen[(chosen[0] - 1) as usize] = true;
        }
        assert!(seen.iter().all(|&x| x), "some head variants are never drawn: {seen:?}");
    }
}
