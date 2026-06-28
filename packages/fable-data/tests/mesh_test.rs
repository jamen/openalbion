//! Validates the mesh parser against real game data: decode every mesh asset in `graphics.big`.
//!
//! Skips gracefully if the game data isn't present. Set `FABLE_DATA` to override the data dir
//! (defaults to `~/Fable/data`).

use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::mesh::Mesh;
use std::{fs::File, path::PathBuf};

fn fable_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FABLE_DATA") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join("Fable/data");
    dir.is_dir().then_some(dir)
}

#[test]
fn decode_all_graphics_meshes() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let big_path = data_dir.join("graphics/graphics.big");
    if !big_path.is_file() {
        eprintln!("skipping: {big_path:?} not found");
        return;
    }

    let mut reader = BigReader::new(File::open(&big_path).expect("open big")).expect("read big");

    // Collect the (bank, symbol) of every mesh asset (extras tagged as Mesh).
    let targets: Vec<(String, String)> = reader
        .bank_iter()
        .flat_map(|bank| {
            let bank_name = bank.metadata().name.to_string();
            bank.asset_iter().filter_map(move |asset| {
                matches!(asset.extras, Some(ExtraMetadata::Mesh(_)))
                    .then(|| (bank_name.clone(), asset.symbol_name.to_string()))
            })
        })
        .collect();

    assert!(!targets.is_empty(), "no mesh assets found in graphics.big");

    let mut decoded = 0usize;
    let mut total_primitives = 0u64;
    let mut total_vertices = 0u64;
    let mut total_indices = 0u64;
    let mut with_geometry = 0usize;
    let mut failures = Vec::new();

    for (bank, symbol) in &targets {
        let (_meta, bytes) = reader.read_asset(bank, symbol).expect("read asset");
        match Mesh::decode(&bytes) {
            Ok(mesh) => {
                decoded += 1;
                total_primitives += mesh.primitives.len() as u64;
                let verts: usize = mesh.primitives.iter().map(|p| p.vertices.len()).sum();
                let idxs: usize = mesh.primitives.iter().map(|p| p.indices.len()).sum();
                total_vertices += verts as u64;
                total_indices += idxs as u64;
                if verts > 0 && idxs > 0 {
                    with_geometry += 1;
                }
                // Referential integrity: every triangle index must point at a real vertex, and
                // each primitive's material index must be in range. A misaligned vertex/index
                // decode would blow these bounds.
                for prim in &mesh.primitives {
                    if !mesh.materials.is_empty()
                        && prim.material_index as usize >= mesh.materials.len()
                    {
                        failures.push(format!(
                            "{bank}/{symbol}: material_index {} >= {}",
                            prim.material_index,
                            mesh.materials.len()
                        ));
                        break;
                    }
                    if let Some(&bad) =
                        prim.indices.iter().find(|&&v| v as usize >= prim.vertices.len())
                    {
                        failures.push(format!(
                            "{bank}/{symbol}: index {bad} >= vertices {}",
                            prim.vertices.len()
                        ));
                        break;
                    }
                    // Sub-meshes must tile the index buffer exactly (contiguous, no gaps/overlap)
                    // and reference valid materials — the renderer draws each as one call.
                    let mut cursor = 0u32;
                    for sub in &prim.sub_meshes {
                        if sub.index_start != cursor
                            || sub.index_start as usize + sub.index_count as usize
                                > prim.indices.len()
                        {
                            failures.push(format!(
                                "{bank}/{symbol}: sub-mesh range {}..+{} not contiguous in {} indices",
                                sub.index_start,
                                sub.index_count,
                                prim.indices.len()
                            ));
                            break;
                        }
                        if !mesh.materials.is_empty()
                            && sub.material_index as usize >= mesh.materials.len()
                        {
                            failures.push(format!(
                                "{bank}/{symbol}: sub-mesh material {} >= {}",
                                sub.material_index,
                                mesh.materials.len()
                            ));
                            break;
                        }
                        cursor += sub.index_count;
                    }
                    if cursor as usize != prim.indices.len() {
                        failures.push(format!(
                            "{bank}/{symbol}: sub-meshes cover {cursor} of {} indices",
                            prim.indices.len()
                        ));
                        break;
                    }
                }

                // Vertex-decode sanity: decoded positions must sit inside the stored bounding box
                // *and* actually fill it. A broken position decode either over-ranges (outside the
                // box) or collapses every vertex to a point (inside the box but no extent), so we
                // check both. Compared against the union of all primitives' vertices.
                let (bmin, bmax) = (mesh.bounding_box.min, mesh.bounding_box.max);
                let mut dmin = [f32::INFINITY; 3];
                let mut dmax = [f32::NEG_INFINITY; 3];
                for prim in &mesh.primitives {
                    for v in &prim.vertices {
                        for a in 0..3 {
                            dmin[a] = dmin[a].min(v.pos[a]);
                            dmax[a] = dmax[a].max(v.pos[a]);
                        }
                    }
                }
                if verts > 0 && idxs > 0 {
                    for a in 0..3 {
                        let extent = (bmax[a] - bmin[a]).max(0.0);
                        // Loose bound: catches gross over-range corruption (positions far outside
                        // the box) while tolerating quantization rounding, which scales with
                        // pos_scale and so can be much larger than the box extent for tiny models.
                        let tol = (extent * 0.25).max(1.0);
                        if dmin[a] < bmin[a] - tol || dmax[a] > bmax[a] + tol {
                            failures.push(format!(
                                "{bank}/{symbol}: axis {a} decoded [{:.3}, {:.3}] far outside box [{:.3}, {:.3}]",
                                dmin[a], dmax[a], bmin[a], bmax[a]
                            ));
                            break;
                        }
                        // Strict: catches the collapse bug (every vertex decoded to ~one point).
                        if extent > 1.0 && (dmax[a] - dmin[a]) < extent * 0.5 {
                            failures.push(format!(
                                "{bank}/{symbol}: axis {a} decoded extent {:.3} << box extent {:.3} (collapsed?)",
                                dmax[a] - dmin[a], extent
                            ));
                            break;
                        }
                    }
                }
            }
            Err(e) => failures.push(format!("{bank}/{symbol}: {e}")),
        }
    }

    eprintln!(
        "decoded {}/{} meshes; {with_geometry} with geometry; {total_primitives} primitives, {total_vertices} vertices, {total_indices} indices",
        decoded,
        targets.len(),
    );
    for f in failures.iter().take(15) {
        eprintln!("  FAIL {f}");
    }

    assert!(
        failures.is_empty(),
        "{} of {} meshes failed to decode",
        failures.len(),
        targets.len()
    );
    // Sanity: most meshes should actually contain renderable geometry.
    assert!(
        with_geometry * 100 / decoded.max(1) > 80,
        "only {with_geometry}/{decoded} meshes had geometry"
    );
}
