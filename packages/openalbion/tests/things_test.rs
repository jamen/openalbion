//! Resolving a level's `.tng` things to mesh placements, checked against the shipped data.
//!
//! These are the numbers behind the claim that a level is populated: how many things a level
//! has, how many resolve to a static mesh, and that every one of those names a real mesh
//! asset. They need a Fable install, so they skip when there is not one — but when they do
//! run they are the evidence, not the screenshot.

use fable_data::appearance::BodyPartSets;
use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::def::binary::{DefBinary, DefBody};
use fable_data::def::names::Names;
use fable_data::def::EngineGraphic;
use fable_data::tng::Tng;
use renderer::ModelKind;
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

/// The body-part mesh lists, joined from each `CREATURE`'s `CCreatureDef` sub-def — the same
/// join `Files::read_creature_body_parts` does, repeated here because `openalbion` is a bin
/// crate with no library to import.
fn creature_body_parts(retail: &Path) -> HashMap<String, BodyPartSets> {
    let names = Names::load(&retail.join("CompiledDefs/names.bin")).unwrap();
    let defs = DefBinary::load_with_names(&retail.join("CompiledDefs/game.bin"), &names).unwrap();

    let mut by_index: HashMap<u32, BodyPartSets> = HashMap::new();
    for entry in defs.entries(&names) {
        let DefBody::CreatureDef(def) = &entry.record.body else {
            continue;
        };
        let morph = &def.random_appearance_morph;
        let sets = BodyPartSets {
            parts: [
                morph.body_parts0.meshes.iter().map(|m| m.mesh_id).collect(),
                morph.body_parts1.meshes.iter().map(|m| m.mesh_id).collect(),
                morph.body_parts2.meshes.iter().map(|m| m.mesh_id).collect(),
            ],
        };
        if !sets.is_empty() {
            by_index.insert(entry.global_index as u32, sets);
        }
    }

    let mut by_name = HashMap::new();
    for entry in defs.entries(&names) {
        if !matches!(&entry.record.body, DefBody::ThingCreatureDef(_)) {
            continue;
        }
        let (Some(name), Some(sub_defs)) = (entry.file_name, entry.record.sub_defs.as_ref()) else {
            continue;
        };
        if let Some(sets) = sub_defs.iter().find_map(|s| by_index.get(&s.def_index)) {
            by_name.insert(name.to_string(), sets.clone());
        }
    }
    by_name
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

/// Every placement must name an asset that really is a mesh in `graphics.big`. If
/// `Graphic.BankIndex` were an index into something rather than an asset id, this is where it
/// would show — and it covers `ENGINE_GRAPHIC_ANIMATING_MESH` on the same terms as static
/// meshes, which is the claim §5 step 6.7 rests on.
#[test]
fn every_placement_resolves_to_a_mesh_asset() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let graphics = thing_graphics(&retail);
    let body_parts = creature_body_parts(&retail);
    let big = BigReader::new(File::open(retail.join("graphics/graphics.big")).unwrap()).unwrap();
    let mesh_ids: HashMap<u32, String> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();

    // (level, things in file, placements, distinct meshes, of which animating: placements,
    // meshes). Both mesh kinds are placed — the animating ones in bind pose (AGENTS.md
    // §5 step 6.7). Arena is where the second column moves most: 91 of its 148 placements are
    // its animating audience.
    let expected = [
        // (level, things, placements, meshes, animating placements, animating meshes,
        //  creatures assembled from body parts)
        ("Witchwood", 64, 44, 26, 6, 6, 1),
        ("LookoutPoint", 288, 213, 63, 21, 19, 6),
        ("Arena", 355, 148, 12, 91, 8, 0),
    ];

    for (level, total, placements, meshes, anim_placements, anim_meshes, part_creatures) in expected
    {
        let text = std::fs::read_to_string(levels.join(format!("{level}.tng"))).unwrap();
        let tng = Tng::parse(&text).unwrap();
        assert_eq!(tng.things().count(), total, "{level}: things in file");

        let resolved = things::resolve_things(&tng, &graphics, &body_parts, (0, 0));
        assert_eq!(
            resolved.placement_count(),
            placements,
            "{level}: mesh placements"
        );
        assert_eq!(resolved.by_mesh.len(), meshes, "{level}: distinct meshes");
        assert_eq!(
            resolved.placement_count_of(ModelKind::Animated),
            anim_placements,
            "{level}: animating placements"
        );
        assert_eq!(
            resolved.mesh_count_of(ModelKind::Animated),
            anim_meshes,
            "{level}: distinct animating meshes"
        );
        assert_eq!(
            resolved.skipped.kind_conflict, 0,
            "{level}: a mesh asset served two kinds"
        );

        // Each creature with body parts contributes three meshes in place of its base body,
        // so the placement count must move by exactly twice the creature count. Arena is the
        // control: its 91 animating things are `OBJECT`s, not `CREATURE`s, and it has none.
        assert_eq!(
            resolved.body_parts.creatures, part_creatures,
            "{level}: creatures assembled from body parts"
        );
        assert_eq!(
            resolved.body_parts.placements,
            part_creatures * 3,
            "{level}: every body-part creature should yield head, torso and legs"
        );

        for id in resolved.by_mesh.keys() {
            assert!(
                mesh_ids.contains_key(id),
                "{level}: bank index {id} is not a mesh asset"
            );
        }

        // Nothing is lost silently: every thing is either placed or counted as skipped.
        // A body-part creature is *one* thing that yields three placements, so those extras
        // are subtracted before the count is compared — otherwise the invariant would drift
        // upward every time a creature gained a part rather than catching a lost thing.
        let extra = resolved.body_parts.placements - resolved.body_parts.creatures;
        assert_eq!(
            resolved.placement_count() - extra + resolved.skipped.total(),
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
    let body_parts = creature_body_parts(&retail);
    let text = std::fs::read_to_string(levels.join("LookoutPoint.tng")).unwrap();
    let resolved = things::resolve_things(&Tng::parse(&text).unwrap(), &graphics, &body_parts, (0, 0));

    let most = resolved
        .by_mesh
        .values()
        .map(things::MeshPlacements::len)
        .max()
        .unwrap();
    assert_eq!(most, 50, "MESH_SMALL_WALL_CURVED_POST_01 is placed 50 times");
    assert!(
        resolved.by_mesh.len() < resolved.placement_count(),
        "grouping should cut uploads: {} meshes for {} placements",
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
    let body_parts = creature_body_parts(&retail);

    for (level, tolerance) in [("Witchwood", 1.0), ("LookoutPoint", 2.0)] {
        let lev =
            fable_data::lev::Lev::from_bytes(&std::fs::read(levels.join(format!("{level}.lev")))
                .unwrap())
            .unwrap();
        let map = fable_data::landscape::LandscapeMap::new(&lev);

        let text = std::fs::read_to_string(levels.join(format!("{level}.tng"))).unwrap();
        let resolved = things::resolve_things(&Tng::parse(&text).unwrap(), &graphics, &body_parts, (0, 0));

        let mut deltas: Vec<f32> = Vec::new();
        for placement in resolved.by_mesh.values().flat_map(|g| &g.placements) {
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

/// A map's `.wld` origin reaches every placement's translation, and nothing else — the
/// mechanism multiple loaded maps depend on to land in one world rather than stacking at
/// (0, 0). Rotation and scale (the matrix's other three columns) must be untouched.
#[test]
fn origin_translates_every_placement_and_nothing_else() {
    let Some((retail, levels)) = fixtures() else {
        eprintln!("skipping: no Fable install");
        return;
    };

    let graphics = thing_graphics(&retail);
    let body_parts = creature_body_parts(&retail);
    let text = std::fs::read_to_string(levels.join("LookoutPoint.tng")).unwrap();
    let tng = Tng::parse(&text).unwrap();

    let at_origin = things::resolve_things(&tng, &graphics, &body_parts, (0, 0));
    let in_world = things::resolve_things(&tng, &graphics, &body_parts, (3232, 3488));

    let mut checked = 0usize;
    for (mesh_id, group) in &at_origin.by_mesh {
        let shifted = &in_world.by_mesh[mesh_id];
        assert_eq!(group.len(), shifted.len(), "mesh {mesh_id}: placement count changed");
        assert_eq!(group.kind, shifted.kind, "mesh {mesh_id}: kind changed with the origin");
        for (a, b) in group.placements.iter().zip(&shifted.placements) {
            for row in 0..4 {
                let expected = match row {
                    3 => [a.transform[3][0] + 3232.0, a.transform[3][1] + 3488.0, a.transform[3][2], a.transform[3][3]],
                    _ => a.transform[row],
                };
                assert_eq!(b.transform[row], expected, "mesh {mesh_id} row {row}");
            }
            checked += 1;
        }
    }
    assert!(checked > 100, "only checked {checked} placements");
}
