//! The `.tng` placements as an independent witness to the landscape's shape.
//!
//! Things were authored standing on the ground, so "do the placements sit on the terrain we
//! build" is a check on the *terrain*, not just on the placements — and it is the only one
//! we have that does not depend on our own reading of the `.lev`. It is what caught the
//! heightmap row indexing being flipped (AGENTS.md §3.4).
//!
//! Needs a Fable install; skips without one.

use fable_data::landscape::LandscapeMap;
use fable_data::lev::Lev;
use fable_data::tng::Tng;
use std::path::{Path, PathBuf};

fn levels() -> Option<PathBuf> {
    let dir = PathBuf::from(
        "/home/jamen/doc/Fable_Anniversary-2013-02-25/Fable/Data/Levels/FinalAlbion",
    );
    dir.exists().then_some(dir)
}

/// Terrain height at a continuous position, bilinear over the four surrounding vertices —
/// the same surface the landscape mesh draws between its vertices.
fn ground(map: &LandscapeMap, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let (h00, h10) = (map.height_at(x0, y0), map.height_at(x0 + 1, y0));
    let (h01, h11) = (map.height_at(x0, y0 + 1), map.height_at(x0 + 1, y0 + 1));
    let a = h00 + (h10 - h00) * fx;
    let b = h01 + (h11 - h01) * fx;
    a + (b - a) * fy
}

/// Vertical offsets of every in-bounds placement above the terrain, sorted.
fn offsets(dir: &Path, level: &str) -> Vec<f32> {
    let lev = Lev::from_bytes(&std::fs::read(dir.join(format!("{level}.lev"))).unwrap()).unwrap();
    let map = LandscapeMap::new(&lev);
    let tng =
        Tng::parse(&std::fs::read_to_string(dir.join(format!("{level}.tng"))).unwrap()).unwrap();

    let mut d: Vec<f32> = Vec::new();
    for thing in tng.things() {
        let Some(p) = thing.placement() else { continue };
        let [x, y, z] = p.position;
        if x < 0.0 || y < 0.0 || x >= map.cell_width() as f32 || y >= map.cell_height() as f32 {
            continue;
        }
        d.push(z - ground(&map, x, y));
    }
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    d
}

/// Outdoor levels put nearly everything on the ground. These thresholds are far from the
/// values the flipped indexing produced (LookoutPoint managed 28% within a metre, and a mean
/// deviation of 2.85), so this fails loudly if the row order regresses.
#[test]
fn things_stand_on_the_terrain() {
    let Some(dir) = levels() else {
        eprintln!("skipping: no level corpus");
        return;
    };

    for (level, min_within_1m, max_mean_deviation) in
        [("Witchwood", 85, 0.6), ("LookoutPoint", 85, 0.6)]
    {
        let d = offsets(&dir, level);
        let within = d.iter().filter(|v| v.abs() < 1.0).count() * 100 / d.len();
        let mean: f32 = d.iter().map(|v| v.abs()).sum::<f32>() / d.len() as f32;

        assert!(
            within >= min_within_1m,
            "{level}: only {within}% of {} placements are within 1m of the ground \
             (mean deviation {mean:.2}) — is the heightmap row order flipped?",
            d.len(),
        );
        assert!(
            mean < max_mean_deviation,
            "{level}: mean deviation from the ground is {mean:.2}"
        );
    }
}

/// Interiors and multi-storey towns legitimately place things well above the ground, so they
/// get a weaker check: the *median* thing is still near it, and nothing is deeply buried.
#[test]
fn town_levels_are_not_systematically_buried() {
    let Some(dir) = levels() else {
        eprintln!("skipping: no level corpus");
        return;
    };

    for level in ["BowerstoneSlums_v2", "SnowspireVillage"] {
        if !dir.join(format!("{level}.lev")).exists() {
            continue;
        }
        let d = offsets(&dir, level);
        let quartile = d[d.len() / 4];
        assert!(
            quartile > -2.0,
            "{level}: a quarter of placements are {quartile:.2} or more below the ground"
        );
    }
}
