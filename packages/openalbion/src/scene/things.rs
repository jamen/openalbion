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

use fable_data::def::{EngineGraphic, EngineGraphicType};
use fable_data::tng::Tng;
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

/// Every placement in a level that resolves to a static mesh, grouped by mesh asset id.
pub struct LevelThings {
    /// `graphics.big` asset id → the object matrices to draw it with.
    pub by_mesh: HashMap<u32, Vec<Placement>>,
    pub skipped: Skipped,
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
}

impl Skipped {
    pub fn total(&self) -> usize {
        self.no_def
            + self.not_drawable
            + self.other_graphic_type.values().sum::<usize>()
            + self.no_placement
    }
}

/// Resolve every thing in `tng` against `graphics`, keyed by `DefinitionType`.
///
/// Only [`EngineGraphicType::EngineGraphicStaticMesh`] is placed. Animating meshes need
/// bones and the palette-skinning shaders, and sprites and generated effects are their own
/// primitive managers; all three are counted in [`Skipped::other_graphic_type`] rather than
/// approximated.
pub fn resolve_things(tng: &Tng, graphics: &HashMap<String, EngineGraphic>) -> LevelThings {
    let mut by_mesh: HashMap<u32, Vec<Placement>> = HashMap::new();
    let mut skipped = Skipped::default();

    for thing in tng.things() {
        let definition_type = &thing.base().definition_type;

        let Some(graphic) = graphics.get(definition_type.as_str()) else {
            skipped.no_def += 1;
            continue;
        };

        if graphic.type_ != EngineGraphicType::EngineGraphicStaticMesh {
            if graphic.bank_index == 0 {
                skipped.not_drawable += 1;
            } else {
                *skipped
                    .other_graphic_type
                    .entry(graphic_type_name(graphic.type_))
                    .or_default() += 1;
            }
            continue;
        }

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

        let Some(transform) = thing
            .placement()
            .and_then(|p| p.object_matrix(scale))
        else {
            skipped.no_placement += 1;
            continue;
        };

        by_mesh
            .entry(graphic.bank_index as u32)
            .or_default()
            .push(Placement {
                transform,
                definition_type: definition_type.clone(),
            });
    }

    LevelThings { by_mesh, skipped }
}

impl LevelThings {
    /// Total placements across every mesh — what the level will actually draw.
    pub fn placement_count(&self) -> usize {
        self.by_mesh.values().map(Vec::len).sum()
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
