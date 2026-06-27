//! Validates the Lev parser against real game data.
//!
//! Reads every `.lev` file out of `FinalAlbion.wad` and parses it. The strongest correctness
//! signal is the header's `navigation_offset`: after parsing the header + heightmap + soundmap,
//! the cursor must land exactly on `navigation_offset` (where the navigation graph begins).
//!
//! Skips gracefully if the game data isn't present. Set `FABLE_DATA` to override the data dir
//! (defaults to `~/Fable/data`).

use fable_data::lev::Lev;
use fable_data::wad::WadReader;
use std::{fs::File, io::BufReader, path::PathBuf};

/// Levels known to use a non-standard header variant the parser doesn't handle yet.
///
/// `HauntedHouse_Filler_01.lev` lacks the two 33,792-byte heightmap/sound palettes that every
/// other FinalAlbion level has (theme strings appear ~176 bytes in instead of after ~67 KB),
/// so the standard header parse runs off the end. It's the only such file out of 398.
const KNOWN_UNSUPPORTED: &[&str] = &["HauntedHouse_Filler_01.lev"];

fn fable_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FABLE_DATA") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join("Fable/data");
    dir.is_dir().then_some(dir)
}

#[test]
fn parse_all_final_albion_levs() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };

    let wad_path = data_dir.join("Levels/FinalAlbion.wad");
    if !wad_path.is_file() {
        eprintln!("skipping: {wad_path:?} not found");
        return;
    }

    let file = BufReader::new(File::open(&wad_path).expect("open wad"));
    let mut reader = WadReader::new(file).expect("read wad");

    let lev_assets: Vec<_> = reader
        .asset_iter()
        .filter(|a| a.path.to_lowercase().ends_with(".lev"))
        .cloned()
        .collect();

    assert!(!lev_assets.is_empty(), "no .lev files found in wad");

    let mut parsed = 0usize;
    let mut offset_ok = 0usize;
    let mut failures = Vec::new();

    for asset in &lev_assets {
        let is_known_unsupported = KNOWN_UNSUPPORTED
            .iter()
            .any(|name| asset.path.ends_with(name));

        let bytes = reader.read_content(asset).expect("read lev content");

        let mut cursor = &bytes[..];
        match Lev::parse(&mut cursor) {
            Ok(_lev) => {
                parsed += 1;
                // Oracle: a full parse (header + heightmap + soundmap + navigation) must consume
                // the entire file.
                let consumed = bytes.len() - cursor.len();
                if consumed == bytes.len() {
                    offset_ok += 1;
                } else {
                    failures.push(format!(
                        "{}: consumed {} of {} bytes ({} left over)",
                        asset.path,
                        consumed,
                        bytes.len(),
                        bytes.len() - consumed,
                    ));
                }
            }
            // Known-unsupported variants are expected to fail; everything else is a real failure.
            Err(_) if is_known_unsupported => {}
            Err(e) => failures.push(format!("{}: parse error: {e}", asset.path)),
        }
    }

    eprintln!(
        "parsed {}/{} levs; fully consumed to EOF on {}/{}",
        parsed,
        lev_assets.len(),
        offset_ok,
        parsed
    );

    for f in failures.iter().take(15) {
        eprintln!("  FAIL {f}");
    }

    assert!(
        failures.is_empty(),
        "{} of {} levs failed",
        failures.len(),
        lev_assets.len()
    );
}

/// Stronger-than-EOF check: the quad-tree should be internally consistent — interior children and
/// leaf neighbours reference node ids that actually exist in the same section. (Blocked nodes are
/// the shared sentinel and have no id, so a small number of references may point at the sentinel.)
#[test]
fn navigation_references_resolve() {
    use fable_data::lev::LevNavigationNode as N;
    use std::collections::HashSet;

    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir");
        return;
    };
    let wad_path = data_dir.join("Levels/FinalAlbion.wad");
    if !wad_path.is_file() {
        eprintln!("skipping: {wad_path:?} not found");
        return;
    }

    let mut reader = WadReader::new(BufReader::new(File::open(&wad_path).unwrap())).unwrap();
    let lev_assets: Vec<_> = reader
        .asset_iter()
        .filter(|a| a.path.to_lowercase().ends_with(".lev"))
        .cloned()
        .collect();

    let mut refs = 0u64;
    let mut resolved = 0u64;

    for asset in &lev_assets {
        let bytes = reader.read_content(asset).expect("read lev");
        let Ok(lev) = Lev::from_bytes(&bytes) else { continue };

        for section in &lev.navigation.sections {
            let ids: HashSet<u32> = section
                .nodes
                .iter()
                .filter_map(|n| match n {
                    N::Interior { header, .. }
                    | N::Navigable { header, .. }
                    | N::Switchable { header, .. } => Some(header.id),
                    N::Blocked { .. } => None,
                })
                .collect();

            // Id 0 is the shared blocked sentinel (inserted at index 0 by LoadFromFile), so a
            // reference of 0 means "blocked child/neighbour" and is legitimately resolvable.
            let mut check = |id: u32| {
                refs += 1;
                if id == 0 || ids.contains(&id) {
                    resolved += 1;
                }
            };

            for node in &section.nodes {
                match node {
                    N::Interior { children, .. } => children.iter().for_each(|&c| check(c)),
                    N::Navigable { leaf, .. } | N::Switchable { leaf, .. } => {
                        leaf.neighbours.iter().for_each(|&n| check(n))
                    }
                    N::Blocked { .. } => {}
                }
            }
        }
    }

    let rate = resolved as f64 / refs as f64;
    eprintln!("node references resolved: {resolved}/{refs} ({:.1}%)", rate * 100.0);
    // A correct decode resolves nearly all references; a misaligned one would resolve almost none.
    assert!(rate > 0.95, "only {:.1}% of node references resolved", rate * 100.0);
}
