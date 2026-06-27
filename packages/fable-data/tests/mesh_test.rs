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
