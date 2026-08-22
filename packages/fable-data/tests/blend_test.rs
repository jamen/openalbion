//! Per-vertex skinning data, checked against the shipped data.
//!
//! `decode_vertex` skipped these eight bytes until now, and their split into palette slots and
//! blend weights is the kind of thing that "looks fine" while being wrong. These are the two
//! invariants that pin it, and both hold on every skinned vertex in the archive.

use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::mesh::Mesh;
use std::fs::File;
use std::path::PathBuf;

fn archive() -> Option<BigReader<File>> {
    let path = PathBuf::from("/home/jamen/Fable/data/graphics/graphics.big");
    path.exists()
        .then(|| BigReader::new(File::open(&path).unwrap()).unwrap())
}

/// The two measurements the layout rests on:
///
/// 1. **weights sum to 255** — if the halves were swapped, or the stride were wrong, they
///    would not;
/// 2. **slots are multiples of 3 and stay inside their block's palette** — the x3 is the
///    shader's three-registers-per-bone stride, and the bound is `Groups[18]`.
#[test]
fn blend_slots_and_weights_are_what_they_claim() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let ids: Vec<u32> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .map(|a| a.id)
        .collect();

    let mut vertices = 0usize;
    let mut weights_sum_255 = 0usize;
    let mut slots_multiple_of_3 = 0usize;
    let mut slots_in_palette = 0usize;
    let mut palette_checked = 0usize;
    let mut max_slot = 0u8;
    let mut max_palette = 0usize;

    for id in ids {
        let asset = big
            .bank_iter()
            .flat_map(|b| b.asset_iter())
            .find(|a| a.id == id)
            .unwrap()
            .clone();
        let Ok(data) = big.read_asset_from_metadata(&asset) else {
            continue;
        };
        let Ok(mesh) = Mesh::decode(&data) else { continue };

        for primitive in &mesh.primitives {
            if primitive.blends.is_empty() {
                continue;
            }
            // Every animated block of a primitive shares its vertex stream, so the widest
            // palette is the bound a slot has to satisfy.
            let palette = primitive
                .animated_blocks
                .iter()
                .map(|b| b.groups.len())
                .max()
                .unwrap_or(0);
            max_palette = max_palette.max(palette);

            for blend in &primitive.blends {
                vertices += 1;
                if blend.raw_weights.iter().map(|&w| w as u32).sum::<u32>() == 255 {
                    weights_sum_255 += 1;
                }
                if blend.scaled_slots.iter().all(|&s| s % 3 == 0) {
                    slots_multiple_of_3 += 1;
                }
                max_slot = max_slot.max(*blend.scaled_slots.iter().max().unwrap());
                if palette > 0 {
                    palette_checked += 1;
                    if blend.slots().iter().all(|&s| (s as usize) < palette) {
                        slots_in_palette += 1;
                    }
                }
            }
        }
    }

    eprintln!("{vertices} skinned vertices across the archive");
    eprintln!("  weights summing to 255      : {weights_sum_255}");
    eprintln!("  slots multiples of 3        : {slots_multiple_of_3}");
    eprintln!("  slots inside their palette  : {slots_in_palette} of {palette_checked}");
    eprintln!("  widest palette {max_palette}, largest stored slot {max_slot} (= slot {})", max_slot / 3);

    assert!(vertices > 400_000, "expected the archive's skinned vertices, got {vertices}");
    assert_eq!(weights_sum_255, vertices, "every vertex's weights must sum to 255");
    assert_eq!(slots_multiple_of_3, vertices, "every stored slot must be a multiple of 3");
    assert_eq!(slots_in_palette, palette_checked, "every slot must index its block's palette");

    // `CAnimatedBlock::Groups[18]` caps the palette, and the data reaches it exactly — which
    // is also the bound a skinning pass has to size its per-draw palette for.
    assert!(max_palette <= 18, "palette of {max_palette} exceeds Groups[18]");
    assert!(max_slot / 3 < 18, "slot {} exceeds the palette", max_slot / 3);
}

/// `weights()` and `slots()` are the accessors a renderer uses; they must agree with the raw
/// bytes rather than quietly renormalising.
#[test]
fn accessors_match_the_stored_bytes() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let asset = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .find(|a| a.symbol_name == "MESH_BS_MALE_MIDDLE_UNCLOTHED_01")
        .cloned()
        .expect("the villager body is in the archive");
    let mesh = Mesh::decode(&big.read_asset_from_metadata(&asset).unwrap()).unwrap();

    let mut checked = 0usize;
    for primitive in &mesh.primitives {
        for blend in &primitive.blends {
            checked += 1;
            let w = blend.weights();
            assert!(
                (w.iter().sum::<f32>() - 1.0).abs() < 1e-3,
                "weights {w:?} do not sum to 1"
            );
            for k in 0..4 {
                assert_eq!(blend.slots()[k], blend.scaled_slots[k] / 3);
            }
        }
    }
    assert!(checked > 1000, "only {checked} vertices");
}
