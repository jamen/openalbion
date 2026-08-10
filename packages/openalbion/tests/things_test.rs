//! Resolving a level's `.tng` things to mesh placements, checked against the shipped data.
//!
//! These are the numbers behind the claim that a level is populated: how many things a level
//! has, how many resolve to a static mesh, and that every one of those names a real mesh
//! asset. They need a Fable install, so they skip when there is not one — but when they do
//! run they are the evidence, not the screenshot.

use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::def::binary::{DefBinary, DefBody};
use fable_data::def::names::Names;
use fable_data::def::EngineGraphic;
use fable_data::tng::Tng;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

// The module under test, compiled into this test binary — `openalbion` is a bin crate, so
// there is no library to import it from.
#[path = "../src/scene/things.rs"]
#[allow(dead_code)]
mod things;

/// Retail supplies the defs and the meshes; the loose `.tng`s come from the Anniversary tree.
fn fixtures() -> Option<(PathBuf, PathBuf)> {
    let retail = PathBuf::from("/home/jamen/Fable/data");
    let levels = PathBuf::from(
        "/home/jamen/doc/Fable_Anniversary-2013-02-25/Fable/Data/Levels/FinalAlbion",
    );
    (retail.join("CompiledDefs/game.bin").exists() && levels.exists()).then_some((retail, levels))
}

fn thing_graphics(retail: &Path) -> HashMap<String, EngineGraphic> {
    let names = Names::load(&retail.join("CompiledDefs/names.bin")).unwrap();
    let defs = DefBinary::load_with_names(&retail.join("CompiledDefs/game.bin"), &names).unwrap();

    let mut map = HashMap::new();
    for entry in defs.entries(&names) {
        let graphic = match &entry.record.body {
            DefBody::ThingObjectDef(d) => &d.graphic,
            DefBody::ThingBuildingDef(d) => &d.graphic,
            DefBody::ThingMarkerDef(d) => &d.graphic,
            DefBody::ThingCreatureDef(d) => &d.graphic,
            _ => continue,
        };
        if let Some(name) = entry.file_name {
            map.insert(name.to_string(), graphic.clone());
        }
    }
    map
}

/// Every static-mesh placement must name an asset that really is a mesh in `graphics.big`.
/// If `Graphic.BankIndex` were an index into something rather than an asset id, this is
/// where it would show.
#[test]
fn every_placement_resolves_to_a_mesh_asset() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let graphics = thing_graphics(&retail);
    let big = BigReader::new(File::open(retail.join("graphics/graphics.big")).unwrap()).unwrap();
    let mesh_ids: HashMap<u32, String> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();

    // (level, things in file, static-mesh placements, distinct meshes)
    let expected = [
        ("Witchwood", 64, 38, 20),
        ("LookoutPoint", 288, 192, 44),
        ("Arena", 355, 57, 4),
    ];

    for (level, total, placements, meshes) in expected {
        let text = std::fs::read_to_string(levels.join(format!("{level}.tng"))).unwrap();
        let tng = Tng::parse(&text).unwrap();
        assert_eq!(tng.things().count(), total, "{level}: things in file");

        let resolved = things::resolve_things(&tng, &graphics);
        assert_eq!(
            resolved.placement_count(),
            placements,
            "{level}: static-mesh placements"
        );
        assert_eq!(resolved.by_mesh.len(), meshes, "{level}: distinct meshes");

        for id in resolved.by_mesh.keys() {
            assert!(
                mesh_ids.contains_key(id),
                "{level}: bank index {id} is not a mesh asset"
            );
        }

        // Nothing is lost silently: every thing is either placed or counted as skipped.
        assert_eq!(
            resolved.placement_count() + resolved.skipped.total(),
            total,
            "{level}: placements + skipped must account for every thing"
        );
    }
}

/// Instancing is the whole reason placements are grouped by mesh. LookoutPoint's most
/// repeated mesh must stay one upload, not fifty.
#[test]
fn repeated_meshes_group_into_one_entry() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let graphics = thing_graphics(&retail);
    let text = std::fs::read_to_string(levels.join("LookoutPoint.tng")).unwrap();
    let resolved = things::resolve_things(&Tng::parse(&text).unwrap(), &graphics);

    let most = resolved.by_mesh.values().map(Vec::len).max().unwrap();
    assert_eq!(most, 50, "MESH_SMALL_WALL_CURVED_POST_01 is placed 50 times");
    assert!(
        resolved.by_mesh.len() * 4 < resolved.placement_count(),
        "grouping should cut uploads several-fold: {} meshes for {} placements",
        resolved.by_mesh.len(),
        resolved.placement_count(),
    );
}

/// Placements must land on the ground the landscape pass builds. A wrong Y-flip, height
/// scale or coordinate space would show as a large median offset rather than a small one.
#[test]
fn placements_sit_on_the_terrain() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let graphics = thing_graphics(&retail);

    for (level, tolerance) in [("Witchwood", 1.0), ("LookoutPoint", 2.0)] {
        let lev =
            fable_data::lev::Lev::from_bytes(&std::fs::read(levels.join(format!("{level}.lev")))
                .unwrap())
            .unwrap();
        let map = fable_data::landscape::LandscapeMap::new(&lev);

        let text = std::fs::read_to_string(levels.join(format!("{level}.tng"))).unwrap();
        let resolved = things::resolve_things(&Tng::parse(&text).unwrap(), &graphics);

        let mut deltas: Vec<f32> = Vec::new();
        for placement in resolved.by_mesh.values().flatten() {
            // The object matrix's translation column is the world position.
            let [x, y, z] = [
                placement.transform[3][0],
                placement.transform[3][1],
                placement.transform[3][2],
            ];
            assert!(
                x >= 0.0
                    && y >= 0.0
                    && x <= map.cell_width() as f32
                    && y <= map.cell_height() as f32,
                "{level}: placement ({x}, {y}) is outside the map"
            );
            deltas.push(z - map.height_at(x.round() as i32, y.round() as i32));
        }

        deltas.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = deltas[deltas.len() / 2];
        assert!(
            median.abs() < tolerance,
            "{level}: median height above terrain is {median:.2}, expected within {tolerance}"
        );
    }
}
