//! A level's ground themes → the grass, flowers and trees standing on them.
//!
//! The generator itself is `fable_data::local_detail`, a port of
//! `CEngineLocalDetailGenerator` and the placement half of `CLocalDetailCacheMap`. This
//! module is the conversion layer either side of it: resolving the level's theme palette to
//! `LOCAL_DETAIL_GENERATOR` defs on the way in, and grouping the placed objects by mesh asset
//! on the way out.
//!
//! **Only the objects the original draws as plain static meshes are grouped here.** That is
//! not a shortcut: `CLocalDetailPrimitiveMesh::AddObjectsToPrimitiveRenderer`
//! (`engine_local_detail_primitives.cpp:484`) calls the same `AddStaticMesh` a `.tng` thing
//! does, through the same shaders, so those objects belong in the model pass by construction.
//! Repeated meshes go through `SHADERS_REPEATED_MESH` instead and are counted, not
//! approximated.

use crate::files::Files;
use fable_data::landscape::LandscapeMap;
use fable_data::lev::Lev;
use fable_data::local_detail::generator::{Generator, PrimitiveType};
use fable_data::local_detail::place::{GeneratorSet, PlacedObject, place_map};
use fable_data::local_detail::rng::DisplacementTable;
use renderer::ModelInstance;
use std::collections::HashMap;

/// A level's local detail, grouped for upload.
#[derive(Default)]
pub struct LevelLocalDetail {
    /// `graphics.big` asset id → the instances to draw it with, for the objects that go
    /// through the model pass.
    pub by_mesh: HashMap<u32, Vec<ModelInstance>>,
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
    /// `LOCAL_DETAIL_PRIMITIVE_TYPE_REPEATED_MESH` — not drawn yet; needs its own pass.
    pub repeated: usize,
}

impl Counts {
    pub fn drawn(&self) -> usize {
        self.mesh + self.hybrid
    }
}

/// Generate `lev`'s local detail and group it by mesh.
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
        match object_type.primitive_type {
            PrimitiveType::Mesh => detail.counts.mesh += 1,
            PrimitiveType::HybridMeshZSprite => detail.counts.hybrid += 1,
            PrimitiveType::RepeatedMesh => {
                detail.counts.repeated += 1;
                continue;
            }
        }

        if object_type.mesh <= 0 {
            continue;
        }

        detail
            .by_mesh
            .entry(object_type.mesh as u32)
            .or_default()
            .push(instance(object));
    }

    tracing::info!(
        "Local detail: {} objects from {} generators over {} palette slots — \
         {} mesh + {} hybrid drawn over {} meshes, {} repeated meshes deferred",
        detail.counts.placed,
        detail.counts.generators,
        detail.counts.slots,
        detail.counts.mesh,
        detail.counts.hybrid,
        detail.by_mesh.len(),
        detail.counts.repeated,
    );

    detail
}

/// One placed object as a model instance.
///
/// The per-object colour (`c0`) stays opaque white, exactly as `.tng` placements do: the
/// engine passes `0xff` for it here too, and the distance fade that would modulate it is its
/// own step.
fn instance(object: &PlacedObject) -> ModelInstance {
    ModelInstance {
        transform: object.transform,
        ..Default::default()
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
