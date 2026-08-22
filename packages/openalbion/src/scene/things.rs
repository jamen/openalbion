//! `.tng` things → renderable placements.
//!
//! A `.tng` says *what* stands where by name (`DefinitionType "OBJECT_BARREL_01"`) and
//! *how* it is oriented (`CTCPhysicsStandard`'s position and `RHSet`). What it does not say
//! is which mesh that def draws — that lives in the def's `Graphic`, resolved from
//! `CompiledDefs/game.bin`, whose `BankIndex` is an asset id in `graphics.big`.
//!
//! Four def types carry a `Graphic` and account for every placed thing that draws:
//! `OBJECT`, `BUILDING`, `MARKER` and `CREATURE`. They are read uniformly, so a building is
//! not a special case — it is an `EngineGraphic` like any other.
//!
//! Placements are grouped by mesh here rather than in the renderer, because that is the
//! shape the pass wants: one upload per mesh asset, one instance per placement. LookoutPoint
//! is 201 placements over 44 meshes.
//!
//! Two graphic types are placed: `ENGINE_GRAPHIC_STATIC_MESH` and
//! `ENGINE_GRAPHIC_ANIMATING_MESH`. The second draws in the **bind pose the asset ships in** —
//! no bones, no animation. That is not an approximation: `VSHADER_PALSKIN_DIRLIGHT_FOG` is
//! `VSHADER_STATIC_DIRLIGHT` with a three-bone blend on the front, and an identity palette
//! makes the blend a no-op (see `renderer::model`'s note). The shipped vertices really are in
//! that pose — over all 244 animating mesh assets the defs reference, the decoded vertex bounds
//! equal the mesh header's own `bounding_box` 244 times out of 244, and the meshes come out
//! person-sized under the same `RenderSizeX × ObjectScale × 0.01` as everything else.
//!
//! **A creature is usually not its `Graphic.BankIndex` mesh.** That names a *base body*
//! (`MESH_BS_MALE_MIDDLE_UNCLOTHED_01`), and the clothed creature is three body-part meshes
//! chosen from the def by the placement's own authored seed — see [`fable_data::appearance`].
//! When a creature has body parts, they are placed **instead of** the base mesh, at the same
//! object matrix.

use fable_data::appearance::BodyPartSets;
use fable_data::def::{EngineGraphic, EngineGraphicType};
use fable_data::tng::Tng;
use renderer::ModelKind;
use std::collections::HashMap;

/// Mesh coordinates are a hundred times world coordinates.
///
/// `CTCGraphicAppearance` builds the scale it hands the primitive as
/// (`fablelib/tc_graphic_appearance.cpp:4658`):
///
/// ```c
/// fVar4 = (this->MainGraphic).RenderSizeX * this->Scale * (float)9.999999776482582e-3;
/// ```
///
/// — the same product at seven other sites in that file. `9.999999776482582e-3` is `0.01f`.
/// It is not a fudge: meshes really are authored at that scale. A fence post
/// (`MESH_SMALL_WALL_CURVED_POST_01`) is 176 units tall in the file and 1.76 world units on
/// the ground, where one landscape cell is 1.0.
const MESH_UNITS_PER_WORLD_UNIT: f32 = 0.01;

/// Every placement in a level that resolves to a mesh, grouped by mesh asset id.
#[derive(Default)]
pub struct LevelThings {
    /// `graphics.big` asset id → the object matrices to draw it with.
    pub by_mesh: HashMap<u32, MeshPlacements>,
    pub skipped: Skipped,
    pub body_parts: BodyPartCounts,
}

/// How much of a level's crowd was assembled from body parts rather than drawn as a base
/// body. `placements / creatures` should be 3 wherever every part list is populated.
#[derive(Default, Debug, Clone, Copy)]
pub struct BodyPartCounts {
    /// Creature placements that resolved to body parts.
    pub creatures: usize,
    /// Part meshes those placements produced.
    pub placements: usize,
}

/// Every placement of one mesh asset, and which toggle they answer to.
///
/// The kind belongs to the *group* rather than to each placement because it is a property of
/// the mesh: an asset is authored skinned or it is not. In the shipped data the two sets are
/// disjoint — 2,327 static mesh ids and 244 animating ones, **zero overlap** — so one upload
/// per mesh never has to serve both. [`MeshPlacements::push`] keeps that an invariant rather
/// than an assumption.
pub struct MeshPlacements {
    pub kind: ModelKind,
    pub placements: Vec<Placement>,
}

impl MeshPlacements {
    fn new(kind: ModelKind) -> Self {
        MeshPlacements {
            kind,
            placements: Vec::new(),
        }
    }

    /// Add a placement, returning `false` if it disagrees with the group's kind — which would
    /// mean one mesh asset is referenced as both static and animating. That does not happen in
    /// the shipped data; if it ever does, the placement is dropped and counted rather than
    /// silently drawn under the wrong toggle.
    fn push(&mut self, kind: ModelKind, placement: Placement) -> bool {
        if self.kind != kind {
            return false;
        }
        self.placements.push(placement);
        true
    }

    pub fn len(&self) -> usize {
        self.placements.len()
    }
}

/// Combine several maps' resolved things into one, for a region loaded as a unit. Placements
/// merge by mesh id — global asset ids, so the same mesh placed on two different maps is
/// still one upload — and skip counts sum.
pub fn merge_things(things: Vec<LevelThings>) -> LevelThings {
    let mut merged = LevelThings::default();
    for level in things {
        for (mesh_id, group) in level.by_mesh {
            let entry = merged
                .by_mesh
                .entry(mesh_id)
                .or_insert_with(|| MeshPlacements::new(group.kind));
            for placement in group.placements {
                if !entry.push(group.kind, placement) {
                    merged.skipped.kind_conflict += 1;
                }
            }
        }
        merged.skipped.no_def += level.skipped.no_def;
        merged.skipped.not_drawable += level.skipped.not_drawable;
        merged.skipped.no_placement += level.skipped.no_placement;
        merged.skipped.kind_conflict += level.skipped.kind_conflict;
        merged.body_parts.creatures += level.body_parts.creatures;
        merged.body_parts.placements += level.body_parts.placements;
        for (kind, count) in level.skipped.other_graphic_type {
            *merged.skipped.other_graphic_type.entry(kind).or_default() += count;
        }
    }
    merged
}

/// One placement of one mesh.
pub struct Placement {
    /// Object → world, column-major (`CalcObjectMatrix`).
    pub transform: [[f32; 4]; 4],
    /// The def this came from, for logging.
    pub definition_type: String,
}

/// Why things did not become placements. Counted rather than dropped silently: a level that
/// draws 90% of its things should say so, not look 10% empty for no stated reason.
#[derive(Default, Debug)]
pub struct Skipped {
    /// The `DefinitionType` names no def with a `Graphic` — camera points, regions, gazes.
    pub no_def: usize,
    /// The def draws nothing (`Graphic.BankIndex == 0`).
    pub not_drawable: usize,
    /// A graphic type this pass does not render, counted per type.
    pub other_graphic_type: HashMap<&'static str, usize>,
    /// No physics component, or one with no orientation (`CTCPhysicsLight`).
    pub no_placement: usize,
    /// One mesh asset referenced as both static and animating. Zero in the shipped data; see
    /// [`MeshPlacements`].
    pub kind_conflict: usize,
}

impl Skipped {
    pub fn total(&self) -> usize {
        self.no_def
            + self.not_drawable
            + self.other_graphic_type.values().sum::<usize>()
            + self.no_placement
            + self.kind_conflict
    }
}

/// Resolve every thing in `tng` against `graphics`, keyed by `DefinitionType`.
///
/// [`EngineGraphicType::EngineGraphicStaticMesh`] and
/// [`EngineGraphicType::EngineGraphicAnimatingMesh`] are placed; the second in bind pose (see
/// the module note). Sprites, 3D sprites and generated effects are their own primitive managers
/// and are counted in [`Skipped::other_graphic_type`] rather than approximated.
///
/// `origin` is the map's position in world cells (`MapX`/`MapY`, AGENTS.md §6.12), added to
/// each placement's translation so things from multiple maps land in one world rather than
/// stacking at (0, 0). `.tng` positions are map-local (AGENTS.md §3.11), so this is the only
/// place the offset belongs — `(0, 0)` for a level loaded on its own.
pub fn resolve_things(
    tng: &Tng,
    graphics: &HashMap<String, EngineGraphic>,
    body_parts: &HashMap<String, BodyPartSets>,
    origin: (i32, i32),
) -> LevelThings {
    let mut by_mesh: HashMap<u32, MeshPlacements> = HashMap::new();
    let mut skipped = Skipped::default();
    let mut body_part_placements = 0usize;
    let mut creatures_with_body_parts = 0usize;

    for thing in tng.things() {
        let definition_type = &thing.base().definition_type;

        let Some(graphic) = graphics.get(definition_type.as_str()) else {
            skipped.no_def += 1;
            continue;
        };

        let kind = match graphic.type_ {
            EngineGraphicType::EngineGraphicStaticMesh => ModelKind::Static,
            EngineGraphicType::EngineGraphicAnimatingMesh => ModelKind::Animated,
            other => {
                if graphic.bank_index == 0 {
                    skipped.not_drawable += 1;
                } else {
                    *skipped
                        .other_graphic_type
                        .entry(graphic_type_name(other))
                        .or_default() += 1;
                }
                continue;
            }
        };

        if graphic.bank_index <= 0 {
            skipped.not_drawable += 1;
            continue;
        }

        // `RenderSizeX * Scale * 0.01`, the float `CalcObjectMatrix` takes. `Scale` is
        // `CTCGraphicAppearance::Scale`, which starts at 1.0 (`tc_graphic_appearance.cpp:526`)
        // and is what a thing's `ObjectScale` multiplies.
        let object_scale = thing
            .physical()
            .and_then(|p| p.object_scale)
            .unwrap_or(1.0);
        let scale = graphic.render_size_x * object_scale * MESH_UNITS_PER_WORLD_UNIT;

        let Some(mut transform) = thing
            .placement()
            .and_then(|p| p.object_matrix(scale))
        else {
            skipped.no_placement += 1;
            continue;
        };
        transform[3][0] += origin.0 as f32;
        transform[3][1] += origin.1 as f32;

        // A creature with body parts is drawn as its parts, not as its base body. The parts
        // partition the same silhouette and carry the clothed materials, so this is a
        // replacement rather than an addition — see `fable_data::appearance`.
        //
        // The seed comes from the placement's own `CTCRandomAppearanceMorph`. A creature def
        // that has body parts but a placement that carries no seed component falls back to
        // seed 0, which is a real draw rather than a special case: the engine's `Seed` member
        // is likewise zero-initialised until something sets it.
        let chosen = body_parts.get(definition_type.as_str()).map(|sets| {
            let seed = thing
                .components()
                .random_appearance_morph
                .as_ref()
                .map_or(0, |m| m.seed as u32);
            sets.choose(seed)
        });

        let meshes: Vec<u32> = match &chosen {
            Some(parts) if !parts.is_empty() => {
                creatures_with_body_parts += 1;
                body_part_placements += parts.len();
                parts.iter().map(|&id| id as u32).collect()
            }
            _ => vec![graphic.bank_index as u32],
        };

        for mesh_id in meshes {
            let placed = by_mesh
                .entry(mesh_id)
                .or_insert_with(|| MeshPlacements::new(kind))
                .push(
                    kind,
                    Placement {
                        transform,
                        definition_type: definition_type.clone(),
                    },
                );
            if !placed {
                skipped.kind_conflict += 1;
            }
        }
    }

    LevelThings {
        by_mesh,
        skipped,
        body_parts: BodyPartCounts {
            creatures: creatures_with_body_parts,
            placements: body_part_placements,
        },
    }
}

impl LevelThings {
    /// Total placements across every mesh — what the level will actually draw.
    pub fn placement_count(&self) -> usize {
        self.by_mesh.values().map(MeshPlacements::len).sum()
    }

    /// Placements of one kind — the number behind "N creatures in this level".
    pub fn placement_count_of(&self, kind: ModelKind) -> usize {
        self.by_mesh
            .values()
            .filter(|g| g.kind == kind)
            .map(MeshPlacements::len)
            .sum()
    }

    /// Distinct mesh assets of one kind.
    pub fn mesh_count_of(&self, kind: ModelKind) -> usize {
        self.by_mesh.values().filter(|g| g.kind == kind).count()
    }
}

fn graphic_type_name(kind: EngineGraphicType) -> &'static str {
    match kind {
        EngineGraphicType::EngineGraphicNull => "null",
        EngineGraphicType::EngineGraphicSprite => "sprite",
        EngineGraphicType::EngineGraphic3dsprite => "3d sprite",
        EngineGraphicType::EngineGraphicGeneratedEffect => "generated effect",
        EngineGraphicType::EngineGraphicAnimatingMesh => "animating mesh",
        EngineGraphicType::EngineGraphicStaticMesh => "static mesh",
        EngineGraphicType::MaxNoEngineGraphicTypes => "out of range",
    }
}
