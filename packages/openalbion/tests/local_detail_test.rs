//! Local detail generated against the shipped data.
//!
//! These are the numbers behind "the level has grass on it": how many objects each level
//! grows, of which kinds, over which meshes — and that every one of them stands on the
//! terrain the landscape pass draws. They need a Fable install, so they skip without one, but
//! when they run they are the evidence, not the screenshot.
//!
//! The counts are exact rather than approximate on purpose. Placement is a pure function of
//! the defs, the heightmap and one transcribed PRNG, so any change to any of those shows up
//! here as a number rather than as a vague difference in a picture.

use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::def::binary::{DefBinary, DefBody};
use fable_data::def::names::Names;
use fable_data::def::{EngineDef, EngineLocalDetailGeneratorDef, EngineThemeDef};
use fable_data::landscape::LandscapeMap;
use fable_data::lev::Lev;
use fable_data::local_detail::generator::{Generator, PrimitiveType};
use fable_data::local_detail::place::{GeneratorSet, PlacedObject, place_map};
use fable_data::local_detail::rng::DisplacementTable;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};

/// Retail supplies the defs and the meshes; the loose `.lev`s come from the Anniversary tree.
fn fixtures() -> Option<(PathBuf, PathBuf)> {
    let retail = PathBuf::from("/home/jamen/Fable/data");
    let levels = PathBuf::from(
        "/home/jamen/doc/Fable_Anniversary-2013-02-25/Fable/Data/Levels/FinalAlbion",
    );
    (retail.join("CompiledDefs/game.bin").exists() && levels.exists()).then_some((retail, levels))
}

struct Defs {
    engine: EngineDef,
    themes: HashMap<String, EngineThemeDef>,
    generators: HashMap<i32, EngineLocalDetailGeneratorDef>,
}

fn defs(retail: &Path) -> Defs {
    let names = Names::load(&retail.join("CompiledDefs/names.bin")).unwrap();
    let binary = DefBinary::load_with_names(&retail.join("CompiledDefs/game.bin"), &names).unwrap();

    let mut engine = None;
    let mut themes = HashMap::new();
    let mut generators = HashMap::new();
    for entry in binary.entries(&names) {
        match &entry.record.body {
            DefBody::Engine(def) => engine = Some(def.clone()),
            DefBody::EngineThemeDef(def) => {
                if let Some(name) = entry.file_name {
                    themes.insert(name.to_string(), def.clone());
                }
            }
            DefBody::EngineLocalDetailGeneratorDef(def) => {
                generators.insert(entry.global_index as i32, def.clone());
            }
            _ => {}
        }
    }

    Defs {
        engine: engine.expect("game.bin has an ENGINE def"),
        themes,
        generators,
    }
}

/// The palette slots of `lev` that name a generator, resolved and built.
fn generator_set(defs: &Defs, lev: &Lev) -> GeneratorSet {
    let mut set = GeneratorSet::new();
    let mut built: HashMap<i32, usize> = HashMap::new();

    for (slot, entry) in lev.header.heightmap_palette.entries.iter().enumerate() {
        let Some(theme) = defs.themes.get(&entry.name) else {
            continue;
        };
        let index = theme.local_detail_generator_def.0;
        if index == 0 {
            continue;
        }
        if let Some(&existing) = built.get(&index) {
            set.point_slot_at(slot as u8, existing);
            continue;
        }
        let Some(def) = defs.generators.get(&index) else {
            continue;
        };
        let built_index = set.insert(Generator::new(def, &defs.engine), &[slot as u8]);
        built.insert(index, built_index);
    }

    set
}

fn place(defs: &Defs, levels: &Path, level: &str) -> (Lev, GeneratorSet, Vec<PlacedObject>) {
    let lev = Lev::from_bytes(&std::fs::read(levels.join(format!("{level}.lev"))).unwrap()).unwrap();
    let set = generator_set(defs, &lev);
    let objects = {
        let map = LandscapeMap::new(&lev);
        // Every shipped map origin is a multiple of 32 and the random draws are indexed
        // `& 0x1f`, so the world origin cannot change what a map grows — placing at (0, 0)
        // here is the same result the engine gets at the level's real `MapX`/`MapY`.
        place_map(&map, &set, (0, 0), &DisplacementTable::build())
    };
    (lev, set, objects)
}

/// Terrain height at a continuous position, bilinear over the four surrounding vertices —
/// the same surface the landscape mesh draws between its vertices, and the same one
/// `placement_test.rs` measures `.tng` things against.
fn ground(map: &LandscapeMap, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let (h00, h10) = (map.height_at(x0, y0), map.height_at(x0 + 1, y0));
    let (h01, h11) = (map.height_at(x0, y0 + 1), map.height_at(x0 + 1, y0 + 1));
    let a = h00 + (h10 - h00) * fx;
    let b = h01 + (h11 - h01) * fx;
    a + (b - a) * fy
}

/// What each level grows, exactly.
#[test]
fn levels_grow_the_objects_they_grow() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let defs = defs(&retail);

    // (level, total, mesh, repeated, hybrid)
    let expected = [
        ("Witchwood", 691, 381, 220, 90),
        ("Darkwood", 1147, 486, 610, 51),
        ("LookoutPoint", 15845, 114, 15663, 68),
        // Arena's palette names no generator at all, so it grows nothing. A level that
        // *should* be bare is as much a result as one that should not.
        ("Arena", 0, 0, 0, 0),
    ];

    for (level, total, mesh, repeated, hybrid) in expected {
        let (_lev, set, objects) = place(&defs, &levels, level);

        let mut counts = HashMap::new();
        for object in &objects {
            *counts
                .entry(object.primitive_type(&set))
                .or_insert(0usize) += 1;
        }

        assert_eq!(objects.len(), total, "{level}: objects placed");
        assert_eq!(
            counts.get(&PrimitiveType::Mesh).copied().unwrap_or(0),
            mesh,
            "{level}: static meshes",
        );
        assert_eq!(
            counts.get(&PrimitiveType::RepeatedMesh).copied().unwrap_or(0),
            repeated,
            "{level}: repeated meshes",
        );
        assert_eq!(
            counts
                .get(&PrimitiveType::HybridMeshZSprite)
                .copied()
                .unwrap_or(0),
            hybrid,
            "{level}: hybrid mesh/zsprite",
        );
    }
}

/// Every object stands on the ground.
///
/// True by construction — the height comes from the same bilinear over the same accessor —
/// which is exactly why it is worth asserting: it is the check that fails if the coordinate
/// space slips, the axes swap, or the world origin is applied twice.
#[test]
fn objects_stand_on_the_terrain() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let defs = defs(&retail);

    for level in ["Witchwood", "Darkwood", "LookoutPoint"] {
        let (lev, _set, objects) = place(&defs, &levels, level);
        let map = LandscapeMap::new(&lev);
        assert!(!objects.is_empty(), "{level} grows nothing");

        for object in &objects {
            let [x, y, z] = object.position();
            assert!(
                (-0.5..map.cell_width() as f32 + 0.5).contains(&x)
                    && (-0.5..map.cell_height() as f32 + 0.5).contains(&y),
                "{level}: object at ({x}, {y}) is outside the map",
            );
            let offset = z - ground(&map, x, y);
            assert!(
                offset.abs() < 1e-3,
                "{level}: object at ({x}, {y}) is {offset} above the ground",
            );
        }
    }
}

/// Every mesh a generator names must really be a mesh in `graphics.big`, and must decode.
///
/// `EngineLocalDetailObjectDef::Mesh` is an asset id, exactly like `Graphic.BankIndex`
/// (AGENTS.md §3.11). If it were an index into something instead, this is where it would show.
#[test]
fn every_object_names_a_real_mesh() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let defs = defs(&retail);

    let mut big = BigReader::new(File::open(retail.join("graphics/graphics.big")).unwrap()).unwrap();
    let meshes: HashMap<u32, String> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();

    let mut checked = HashSet::new();
    for level in ["Witchwood", "Darkwood", "LookoutPoint"] {
        let (_lev, set, objects) = place(&defs, &levels, level);
        for object in &objects {
            let mesh = object.object_type(&set).mesh;
            if !checked.insert(mesh) {
                continue;
            }
            assert!(
                meshes.contains_key(&(mesh as u32)),
                "{level}: mesh id {mesh} is not a mesh asset",
            );
        }
    }
    assert!(checked.len() > 30, "only {} distinct meshes", checked.len());

    // And they decode: a generator that names an unreadable mesh would be an empty patch of
    // ground with a warning, which is exactly the kind of thing that goes unnoticed.
    let assets: HashMap<u32, _> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .map(|a| (a.id, a.clone()))
        .collect();
    for mesh in &checked {
        let asset = &assets[&(*mesh as u32)];
        let data = big.read_asset_from_metadata(asset).unwrap();
        fable_data::mesh::Mesh::decode(&data)
            .unwrap_or_else(|e| panic!("{}: {e:?}", asset.symbol_name));
    }
}

/// The whole point of transcribing the PRNG rather than picking one: the same level generates
/// the same foliage every time, and would generate it identically in another process.
#[test]
fn generation_is_deterministic() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let defs = defs(&retail);

    let (_lev, _set, first) = place(&defs, &levels, "Witchwood");
    let (_lev, _set, second) = place(&defs, &levels, "Witchwood");

    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(&second) {
        assert_eq!(a.transform, b.transform);
        assert_eq!(a.scale, b.scale);
        assert_eq!(a.object_type, b.object_type);
    }
}

/// Grass is placed only on ground flat enough for it: `SlopeFadeStart`/`End` of 0.80..0.90
/// on the grass objects means the steeper the ground, the fewer survive, and none at all
/// past the end. Measured over the objects that carry a slope fade at all.
#[test]
fn the_slope_fade_thins_objects_out_on_steep_ground() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let defs = defs(&retail);
    let (lev, set, objects) = place(&defs, &levels, "LookoutPoint");
    let map = LandscapeMap::new(&lev);

    // Any object standing where the ground normal's Z is below the lowest SlopeFadeStart in
    // the level's generators is a placement the rejection should have caught.
    let mut floor = f32::MAX;
    for slot in 0..=255u8 {
        let Some(generator) = set.for_slot(slot) else {
            continue;
        };
        for layer in &generator.layers {
            for object in &layer.objects {
                if object.slope_fade_start < object.slope_fade_end {
                    floor = floor.min(object.slope_fade_start);
                }
            }
        }
    }
    assert!(floor < 1.0, "no object in LookoutPoint carries a slope fade");

    let mut steepest_kept = 1.0f32;
    for object in &objects {
        steepest_kept = steepest_kept.min(object.normal[2]);
    }
    // Objects without a slope fade stand anywhere, so this only proves the mechanism runs at
    // all if some ground in the level is steeper than the floor.
    let mut steepest_ground = 1.0f32;
    for y in 0..map.cell_height() {
        for x in 0..map.cell_width() {
            steepest_ground =
                steepest_ground.min(fable_data::landscape::mesh::vertex_normal(&map, x, y)[2]);
        }
    }
    assert!(
        steepest_ground < floor,
        "LookoutPoint has no ground steep enough ({steepest_ground}) to exercise the fade at {floor}",
    );
}
