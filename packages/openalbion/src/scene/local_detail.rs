//! A level's ground themes → the grass, flowers and trees standing on them.
//!
//! The generator itself is `fable_data::local_detail`, a port of
//! `CEngineLocalDetailGenerator` and the placement half of `CLocalDetailCacheMap`. This
//! module is the conversion layer either side of it: resolving the level's theme palette to
//! `LOCAL_DETAIL_GENERATOR` defs on the way in, and grouping the placed objects by mesh asset
//! on the way out.
//!
//! Objects are split by primitive type, which is the engine's own split:
//! `CLocalDetailPrimitiveMesh::AddObjectsToPrimitiveRenderer`
//! (`engine_local_detail_primitives.cpp:484`) calls the same `AddStaticMesh` a `.tng` thing
//! does, so those objects go to the model pass; repeated meshes go to `LocalDetailPass` and
//! its transcription of `SHADERS_REPEATED_MESH`.

use crate::files::Files;
use fable_data::landscape::LandscapeMap;
use fable_data::lev::Lev;
use fable_data::local_detail::generator::{Generator, PrimitiveType};
use fable_data::local_detail::place::{GeneratorSet, PlacedObject, place_map};
use fable_data::local_detail::rng::DisplacementTable;
use renderer::{LocalDetailInstance, ModelInstance};
use std::collections::HashMap;

/// A level's local detail, grouped for upload.
#[derive(Default)]
pub struct LevelLocalDetail {
    /// `graphics.big` asset id → the instances to draw it with, for the objects that go
    /// through the model pass.
    pub by_mesh: HashMap<u32, Vec<ModelInstance>>,
    /// The repeated meshes, one batch per `(mesh asset id, AlphaRef)`.
    ///
    /// Keyed by both because the alpha test belongs to the object type rather than to the
    /// mesh's materials — the engine passes it per `CAddMeshDesc` — so one mesh shared by two
    /// object types with different refs is two batches.
    pub repeated: HashMap<(u32, i32), Vec<LocalDetailInstance>>,
    pub counts: Counts,
}

/// What the level generated, and what reached the model pass.
#[derive(Default, Debug)]
pub struct Counts {
    /// Palette slots that named a generator.
    pub slots: usize,
    /// Distinct generators built.
    pub generators: usize,
    /// Objects placed, of every primitive type.
    pub placed: usize,
    /// `LOCAL_DETAIL_PRIMITIVE_TYPE_MESH` — drawn.
    pub mesh: usize,
    /// `LOCAL_DETAIL_PRIMITIVE_TYPE_HYBRID_MESH_ZSPRITE` — drawn as its mesh half, with no
    /// impostor beyond `ZSpriteFadeStart`.
    pub hybrid: usize,
    /// `LOCAL_DETAIL_PRIMITIVE_TYPE_REPEATED_MESH` — drawn through `LocalDetailPass`.
    pub repeated: usize,
}

impl Counts {
    /// Objects that reach the model pass. The repeated meshes are counted separately because
    /// they go somewhere else.
    pub fn drawn_as_models(&self) -> usize {
        self.mesh + self.hybrid
    }

    fn add(&mut self, other: &Counts) {
        self.slots += other.slots;
        self.generators += other.generators;
        self.placed += other.placed;
        self.mesh += other.mesh;
        self.hybrid += other.hybrid;
        self.repeated += other.repeated;
    }
}

/// Combine several maps' local detail into one, for a region loaded as a unit. Instances
/// merge by mesh id — global asset ids, so foliage sharing a mesh across two maps is still
/// one upload — and counts sum.
pub fn merge_local_detail(details: Vec<LevelLocalDetail>) -> LevelLocalDetail {
    let mut merged = LevelLocalDetail::default();
    for detail in details {
        merged.counts.add(&detail.counts);
        for (mesh_id, mut instances) in detail.by_mesh {
            merged.by_mesh.entry(mesh_id).or_default().append(&mut instances);
        }
        for (key, mut instances) in detail.repeated {
            merged.repeated.entry(key).or_default().append(&mut instances);
        }
    }
    merged
}

/// Generate `lev`'s local detail and group it by mesh.
///
/// `origin` (`MapX`/`MapY`, AGENTS.md §6.12) reaches [`place_map`]'s random draws — which
/// plant matters is a property of world position — but its objects come back **map-local**
/// (`place_map`'s own doc, and pinned by `local_detail_test.rs`'s
/// `the_origin_moves_the_draws_and_not_the_objects`). This function adds `origin` a second
/// time, only to the returned instances' translations, so the result lands in world space
/// without touching that invariant.
pub fn build_local_detail(files: &Files, lev: &Lev, origin: (i32, i32)) -> LevelLocalDetail {
    let map = LandscapeMap::new(lev);
    let (generators, slots) = resolve_generators(files, lev);
    if generators.is_empty() {
        return LevelLocalDetail::default();
    }

    let table = DisplacementTable::build();
    let objects = place_map(&map, &generators, origin, &table);

    let mut detail = LevelLocalDetail {
        counts: Counts {
            slots,
            generators: generators.len(),
            placed: objects.len(),
            ..Default::default()
        },
        ..Default::default()
    };

    for object in &objects {
        let object_type = object.object_type(&generators);
        if object_type.mesh <= 0 {
            continue;
        }

        match object_type.primitive_type {
            PrimitiveType::Mesh => detail.counts.mesh += 1,
            PrimitiveType::HybridMeshZSprite => detail.counts.hybrid += 1,
            PrimitiveType::RepeatedMesh => {
                detail.counts.repeated += 1;
                detail
                    .repeated
                    .entry((object_type.mesh as u32, object_type.alpha_ref))
                    .or_default()
                    .push(repeated_instance(object, origin));
                continue;
            }
        }

        detail
            .by_mesh
            .entry(object_type.mesh as u32)
            .or_default()
            .push(model_instance(object, origin));
    }

    tracing::info!(
        "Local detail: {} objects from {} generators over {} palette slots — \
         {} mesh + {} hybrid over {} meshes, {} repeated over {} batches",
        detail.counts.placed,
        detail.counts.generators,
        detail.counts.slots,
        detail.counts.mesh,
        detail.counts.hybrid,
        detail.by_mesh.len(),
        detail.counts.repeated,
        detail.repeated.len(),
    );

    detail
}

/// One placed object as a model instance.
///
/// The per-object colour (`c0`) stays opaque white, exactly as `.tng` placements do: the
/// engine passes `0xff` for it here too, and the distance fade that would modulate it is its
/// own step.
///
/// `origin` places the object in world space — `object.transform` is map-local, per
/// [`build_local_detail`]'s doc.
fn model_instance(object: &PlacedObject, origin: (i32, i32)) -> ModelInstance {
    let mut transform = object.transform;
    transform[3][0] += origin.0 as f32;
    transform[3][1] += origin.1 as f32;
    ModelInstance {
        transform,
        ..Default::default()
    }
}

/// One placed object as a repeated-mesh instance.
///
/// Built from the angle and scale rather than from the placement matrix, because that is what
/// `BuildFromSourceMeshes` (`engine_local_detail_primitives.cpp:2799`) does: it writes
/// `(cos(Angle) * Scale, sin(Angle) * Scale, 0, 0)` and `(E41, E42, E43, Scale)`. The
/// difference is visible — a repeated mesh whose def sets `TiltToSlope` still stands upright,
/// because the tilt never survives into those four floats.
///
/// `origin` places the object in world space, same as [`model_instance`].
fn repeated_instance(object: &PlacedObject, origin: (i32, i32)) -> LocalDetailInstance {
    let (sin, cos) = fable_data::local_detail::place::fast_sin_cos(object.angle);
    LocalDetailInstance {
        rotation: [cos * object.scale, sin * object.scale, 0.0, 0.0],
        offset: [
            object.transform[3][0] + origin.0 as f32,
            object.transform[3][1] + origin.1 as f32,
            object.transform[3][2],
            object.scale,
        ],
        ground_normal: [object.normal[0], object.normal[1], object.normal[2], 0.0],
    }
}

/// Resolve the level's theme palette to generators.
///
/// Two hops, and they resolve differently on purpose. The palette entry's **name** finds the
/// `ENGINE_THEME` def, because the index stored beside it is stale in retail data
/// (AGENTS.md §3.4). That theme's `LocalDetailGeneratorDef` is then a plain def **index**,
/// which does resolve — measured across all 79 themes in retail `game.bin` that name one.
fn resolve_generators(files: &Files, lev: &Lev) -> (GeneratorSet, usize) {
    let mut set = GeneratorSet::new();
    let Some(engine) = files.engine_def() else {
        tracing::warn!("No ENGINE def, local detail disabled");
        return (set, 0);
    };

    // Several palette slots routinely name the same generator, and building its placement
    // grids again would be the expensive half of the work done twice.
    let mut built: HashMap<i32, usize> = HashMap::new();
    let mut slots = 0usize;
    let mut unresolved: Vec<&str> = Vec::new();

    for (slot, entry) in lev.header.heightmap_palette.entries.iter().enumerate() {
        if entry.name.is_empty() || entry.name == "NO_THEME" {
            continue;
        }
        let Some(theme) = files.engine_theme_by_name(&entry.name) else {
            continue;
        };
        let def_index = theme.local_detail_generator_def.0;
        if def_index == 0 {
            continue;
        }

        if let Some(&index) = built.get(&def_index) {
            set.point_slot_at(slot as u8, index);
            slots += 1;
            continue;
        }

        let Some(def) = files.local_detail_generator(def_index) else {
            unresolved.push(&entry.name);
            continue;
        };

        let index = set.insert(Generator::new(def, engine), &[slot as u8]);
        built.insert(def_index, index);
        slots += 1;
        tracing::debug!(
            "Local detail: palette slot {slot} ({}) ← generator def {def_index}, {} layers",
            entry.name,
            def.layers.len(),
        );
    }

    if !unresolved.is_empty() {
        tracing::warn!(
            "{} themes name a LOCAL_DETAIL_GENERATOR that game.bin does not have: {unresolved:?}",
            unresolved.len(),
        );
    }

    (set, slots)
}
