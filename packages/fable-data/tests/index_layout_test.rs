//! How a primitive's blocks find their indices, checked against the whole archive.
//!
//! This is the regression guard for a bug that shipped for months and only showed on screen:
//! animated blocks were walked sequentially rather than seeking `start_index`, and because
//! `expand_block` consumes one index per triangle of a strip while a strip of N triangles
//! *occupies* N + 2, every block after the first began two indices early. The error compounded
//! — 2, 4, 6, 8 — and rendered as a spray of long thin triangles across a villager's chest.
//!
//! Two independent properties are pinned, because the first alone would not have caught it:
//! the file's layout, and what the expansion actually produces.

use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::mesh::Mesh;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

fn archive() -> Option<BigReader<File>> {
    let path = PathBuf::from("/home/jamen/Fable/data/graphics/graphics.big");
    path.exists()
        .then(|| BigReader::new(File::open(&path).unwrap()).unwrap())
}

fn meshes(big: &mut BigReader<File>) -> Vec<(String, Mesh)> {
    let ids: Vec<(u32, String)> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();
    let mut out = Vec::new();
    for (id, name) in ids {
        let asset = big
            .bank_iter()
            .flat_map(|b| b.asset_iter())
            .find(|a| a.id == id)
            .unwrap()
            .clone();
        let Ok(data) = big.read_asset_from_metadata(&asset) else {
            continue;
        };
        if let Ok(mesh) = Mesh::decode(&data) {
            out.push((name, mesh));
        }
    }
    out
}

/// **A strip of N triangles occupies N + 2 indices**, and `start_index` is the running total.
///
/// This is the property the old sequential walk violated. It is stated as arithmetic on the
/// declared fields, so it holds whatever the expansion does with them.
#[test]
fn block_start_indices_are_the_cumulative_layout() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let mut blocks = 0usize;
    let mut agree = 0usize;
    let mut multi_block_primitives = 0usize;

    for (name, mesh) in meshes(&mut big) {
        for primitive in &mesh.primitives {
            if primitive.animated_blocks.len() > 1 {
                multi_block_primitives += 1;
            }
            let mut expected = 0u32;
            for block in &primitive.animated_blocks {
                blocks += 1;
                if block.base.start_index == expected {
                    agree += 1;
                } else if blocks < 4000 {
                    eprintln!(
                        "{name}: block start_index {} != cumulative {expected}",
                        block.base.start_index
                    );
                }
                expected += if block.base.is_strip {
                    block.base.primitive_count + 2
                } else {
                    block.base.primitive_count * 3
                };
            }
        }
    }

    eprintln!("{blocks} animated blocks, {multi_block_primitives} multi-block primitives");
    assert!(blocks > 1000, "expected the archive's animated blocks, got {blocks}");
    assert!(
        multi_block_primitives > 100,
        "only {multi_block_primitives} multi-block primitives — the case this guards"
    );
    assert_eq!(agree, blocks, "a block's start_index must be the cumulative index count");
}

/// **Consecutive triangles of a strip share an edge.** That is what makes it a strip, and it is
/// the property that fails when a block reads from the wrong offset — the indices are still
/// valid numbers, so nothing else complains.
///
/// This checks the *expanded* output rather than the declared fields, so it catches a wrong
/// offset even if the arithmetic above is somehow satisfied.
#[test]
fn consecutive_strip_triangles_share_an_edge() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let mut pairs = 0usize;
    let mut sharing = 0usize;
    let mut worst: Option<(f64, String)> = None;

    for (name, mesh) in meshes(&mut big) {
        for primitive in &mesh.primitives {
            // Sub-meshes are emitted static blocks first, then animated ones, so index k maps
            // back to its block. Only **strips** are checked: a triangle list's consecutive
            // triangles share nothing by construction, and a tree's foliage is all lists.
            let is_strip = |k: usize| -> bool {
                primitive
                    .static_blocks
                    .get(k)
                    .map(|b| b.base.is_strip)
                    .or_else(|| {
                        primitive
                            .animated_blocks
                            .get(k.wrapping_sub(primitive.static_blocks.len()))
                            .map(|b| b.base.is_strip)
                    })
                    .unwrap_or(false)
            };
            for (k, sub) in primitive.sub_meshes.iter().enumerate() {
                if !is_strip(k) {
                    continue;
                }
                let start = sub.index_start as usize;
                let end = (start + sub.index_count as usize).min(primitive.indices.len());
                let range = &primitive.indices[start..end];
                if range.len() < 6 {
                    continue;
                }
                let mut local_pairs = 0usize;
                let mut local_sharing = 0usize;
                for window in range.chunks_exact(3).collect::<Vec<_>>().windows(2) {
                    // Degenerate triangles are excluded: half of every strip is stitching with
                    // two equal indices, and such a triangle has only two *distinct* vertices,
                    // so it cannot share two with its neighbour however correct the strip is.
                    let distinct = |t: &[u16]| -> Option<HashSet<u16>> {
                        let set: HashSet<u16> = t.iter().copied().collect();
                        (set.len() == 3).then_some(set)
                    };
                    let (Some(a), Some(b)) = (distinct(window[0]), distinct(window[1])) else {
                        continue;
                    };
                    local_pairs += 1;
                    if a.intersection(&b).count() >= 2 {
                        local_sharing += 1;
                    }
                }
                pairs += local_pairs;
                sharing += local_sharing;
                if local_pairs > 20 {
                    let frac = local_sharing as f64 / local_pairs as f64;
                    if worst.as_ref().is_none_or(|(w, _)| frac < *w) {
                        worst = Some((frac, name.clone()));
                    }
                }
            }
        }
    }

    let fraction = sharing as f64 / pairs as f64;
    eprintln!(
        "{sharing} of {pairs} consecutive triangle pairs share an edge ({:.2}%)",
        fraction * 100.0
    );
    if let Some((frac, name)) = &worst {
        eprintln!("  worst block: {:.1}% in {name}", frac * 100.0);
    }

    assert!(pairs > 100_000, "only {pairs} pairs checked");
    assert!(
        fraction > 0.99,
        "only {:.2}% of consecutive triangles share an edge — blocks are reading from the \
         wrong offset",
        fraction * 100.0
    );
}
