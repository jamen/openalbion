//! `LOCAL_DETAIL_GENERATOR` defs → the runtime shape the placement loop reads.
//!
//! A ground theme names one generator (`CEngineThemeDef::LocalDetailGeneratorDef`,
//! `fablelib/defs/engine_theme_def.hpp:900`), a generator is layers, a layer is objects, and
//! an object is a mesh plus the rules for where it may stand. This module is the three
//! engine types that sit between the def and the loop —
//! `CLocalDetailGeneratorTheme` / `CLocalDetailLayer` / `CLocalDetailObjectCollectionType`
//! (`fableengine/engine_local_detail_theme.cpp`) — plus the placement grids they share.

use super::grid::PlacementGrid;
use crate::def::{EngineDef, EngineLocalDetailGeneratorDef, EngineLocalDetailObjectDef};

/// Which renderer an object goes to — `LOCAL_DETAIL_PRIMITIVE_TYPE`
/// (`fableengine/engine_local_detail_primitives.hpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    /// A plain static mesh. `CLocalDetailPrimitiveMesh::AddObjectsToPrimitiveRenderer`
    /// (`engine_local_detail_primitives.cpp:484`) calls `AddStaticMesh` — the same call a
    /// `.tng` thing makes, through the same shaders.
    Mesh,
    /// Batched through `SHADERS_REPEATED_MESH`: 16 instances per draw, a Z rotation and a Z
    /// scale only, and one flat colour per instance computed from the landscape normal.
    RepeatedMesh,
    /// A mesh near the camera and a generated billboard impostor beyond
    /// `ZSpriteFadeStart`.
    HybridMeshZSprite,
}

/// One `CLocalDetailObjectCollectionType`: everything about an object that does not vary
/// between its placements.
#[derive(Debug, Clone)]
pub struct ObjectType {
    /// `graphics.big` asset id. Measured: all 227 shipped objects resolve directly, exactly
    /// like `Graphic.BankIndex` (AGENTS.md §3.11).
    pub mesh: i32,
    /// The shadow proxy, defaulted to [`ObjectType::mesh`] when the def leaves it at zero and
    /// the object casts shadows.
    pub shadow_mesh: i32,
    /// The impostor source, defaulted to [`ObjectType::mesh`] when the def leaves it at zero.
    pub zsprite_mesh: i32,
    pub primitive_type: PrimitiveType,
    pub fade_start: f32,
    pub fade_end: f32,
    pub zsprite_fade_start: f32,
    pub zsprite_fade_end: f32,
    /// Alpha test reference, 0..255, resolved against the `ENGINE` def's defaults.
    pub alpha_ref: i32,
    pub alpha_mip_bias: f32,
    pub alpha_is_boolean: bool,
    /// Lit once per object from the ground normal rather than per vertex. Forced on for
    /// [`PrimitiveType::RepeatedMesh`], whose vertex shader never sees a normal.
    pub landscape_normal_lighting: bool,
    pub receives_shadows: bool,
    pub casts_shadows: bool,
    pub has_wind_skew: bool,
    pub wind_skew_constant_factor: f32,
    pub wind_skew_random_factor: f32,
    pub wind_skew_speed_factor: f32,
}

impl ObjectType {
    /// Port of `CLocalDetailObjectCollectionType::CLocalDetailObjectCollectionType`
    /// (`engine_local_detail_theme.cpp:2650`).
    ///
    /// The primitive type is decided here, and the order of the tests matters: an object with
    /// a `ZSpriteFadeEnd` is a hybrid *even if* `IsRepeatedMesh` is set.
    fn new(def: &EngineLocalDetailObjectDef, engine: &EngineDef) -> ObjectType {
        let primitive_type = if def.z_sprite_fade_end > 0.0 {
            PrimitiveType::HybridMeshZSprite
        } else if def.is_repeated_mesh {
            PrimitiveType::RepeatedMesh
        } else {
            PrimitiveType::Mesh
        };

        // `if (AlphaRef < 0) AlphaRef = AlphaIsBoolean ? GPEngineDef[0x64] : GPEngineDef[0x68]`
        // — `LocalDetailBooleanAlphaDefaultAlphaRef` and `DefaultPrimitiveAlphaRef`
        // (`fablelib/defs/engine_def.hpp:938,940`). 255 and above clamp to 255, but only on
        // the path where the def supplied a value.
        let alpha_ref = if def.alpha_ref < 0 {
            if def.alpha_is_boolean {
                engine.local_detail_boolean_alpha_default_alpha_ref
            } else {
                engine.default_primitive_alpha_ref
            }
        } else {
            def.alpha_ref.min(255)
        };

        ObjectType {
            mesh: def.mesh,
            // `if (ShadowMesh == 0) ShadowMesh = Mesh`, taken only when CastShadows is set.
            shadow_mesh: if def.shadow_mesh == 0 && def.cast_shadows {
                def.mesh
            } else {
                def.shadow_mesh
            },
            // `if (ZSpriteMesh == 0) ZSpriteMesh = Mesh` — unconditional.
            zsprite_mesh: if def.z_sprite_mesh == 0 {
                def.mesh
            } else {
                def.z_sprite_mesh
            },
            primitive_type,
            fade_start: def.fade_start,
            fade_end: def.fade_end,
            zsprite_fade_start: def.z_sprite_fade_start,
            zsprite_fade_end: def.z_sprite_fade_end,
            alpha_ref,
            alpha_mip_bias: def.alpha_mip_bias,
            alpha_is_boolean: def.alpha_is_boolean,
            // The constructor sets this from the def and then *overwrites* it with true on
            // the repeated-mesh branch.
            landscape_normal_lighting: def.has_landscape_normal_lighting
                || primitive_type == PrimitiveType::RepeatedMesh,
            receives_shadows: def.receive_shadows,
            casts_shadows: def.cast_shadows,
            has_wind_skew: def.has_wind_skew,
            wind_skew_constant_factor: def.wind_skew_constant_factor,
            wind_skew_random_factor: def.wind_skew_random_factor,
            wind_skew_speed_factor: def.wind_skew_speed_factor,
        }
    }
}

/// One `CLocalDetailLayer::CObject`: how a placement of an [`ObjectType`] varies.
#[derive(Debug, Clone)]
pub struct LayerObject {
    pub scale: f32,
    pub scale_random_element: f32,
    pub slope_fade_start: f32,
    pub slope_fade_end: f32,
    /// Relative weight within the layer — see [`Layer::selection`].
    pub probability: f32,
    /// The ground blend this object needs before it will stand here.
    ///
    /// **Zero on all 227 shipped objects**, so in practice this only rejects a theme with no
    /// weight at all. Transcribed because the comparison is in the loop, not because it does
    /// anything.
    pub theme_blend_threshold: f32,
    pub tilt_to_slope: bool,
    /// Index into [`Generator::object_types`].
    pub object_type: usize,
}

/// One `CLocalDetailLayer`: a set of objects sharing a placement grid.
#[derive(Debug, Clone)]
pub struct Layer {
    pub objects: Vec<LayerObject>,
    /// `ObjectSelectionTable[32]` — see [`Layer::build_selection_table`].
    selection: [i32; 32],
}

impl Layer {
    /// `CLocalDetailLayer::BuildObjectSelectionTable` (`engine_local_detail_theme.cpp:2395`).
    ///
    /// 32 slots covering the objects in proportion to `Probability / Σ Probability` — so
    /// `Probability` is a **relative weight inside the layer, never a chance of nothing**.
    /// That is worth stating because 11 of the 101 shipped layers have probabilities summing
    /// to something other than 1 (0.30 up to 1.10), and under this reading they are simply
    /// renormalised.
    ///
    /// The advance is `if`, not `while`: at most one object per slot, so a layer with more
    /// than 32 objects would lose the tail. None has more than five.
    fn build_selection_table(objects: &[LayerObject]) -> [i32; 32] {
        if objects.is_empty() {
            // The only path that yields a negative entry, and the reason
            // `AddObjectsFromLayerElement` checks for one.
            return [-1; 32];
        }

        let total: f32 = objects.iter().map(|o| o.probability).sum();
        let mut cumulative = objects[0].probability / total;
        let mut object = 0usize;
        let mut table = [0i32; 32];

        for (slot, entry) in table.iter_mut().enumerate() {
            // The original's bound is `object < objects.len()`, which would let `object`
            // reach the end; it is unreachable because the cumulative is 1.0 by then and
            // `1.0 * 32 < 31` is false.
            if cumulative * 32.0 < slot as f32 && object + 1 < objects.len() {
                object += 1;
                cumulative += objects[object].probability / total;
            }
            *entry = object as i32;
        }

        table
    }

    /// Which object a `[0, 1)` draw selects, or `None` for an empty layer —
    /// `ObjectSelectionTable[floor(rand * 32)]`.
    ///
    /// The index is a floor, not a round: the decomp's `(int)ROUND(v - 0.5)` with a `+1`
    /// fixup at exact integers is MSVC's inlined `floorf` under round-to-nearest-even.
    pub fn select(&self, random: f32) -> Option<&LayerObject> {
        let slot = (random * 32.0).floor().clamp(0.0, 31.0) as usize;
        let index = self.selection[slot];
        (index >= 0).then(|| &self.objects[index as usize])
    }
}

/// One `LOCAL_DETAIL_GENERATOR` def, ready to place.
///
/// Deliberately neither `Clone` nor `Debug`: the placement grids inside it can run to tens of
/// thousands of elements, and a generator is meant to be built once per level and borrowed.
pub struct Generator {
    pub object_types: Vec<ObjectType>,
    pub layers: Vec<Layer>,
    /// One per layer, in the same order — `CLocalDetailLayer::PlacementGrid`.
    pub grids: Vec<PlacementGrid>,
}

impl Generator {
    /// Build a generator from its def, including its placement grids.
    ///
    /// Grid building is the expensive half — a 0.12-spacing layer throws ~200,000 darts — so
    /// a generator should be built once per level and shared, which is what
    /// `CEngineLocalDetailGenerator::BuildThemes` (`engine_local_detail_generator.cpp:1683`)
    /// does with its one instance per def.
    pub fn new(def: &EngineLocalDetailGeneratorDef, engine: &EngineDef) -> Generator {
        let mut object_types: Vec<ObjectType> = Vec::new();
        let mut layers = Vec::with_capacity(def.layers.len());

        for layer_def in &def.layers {
            let objects = layer_def
                .objects
                .iter()
                .map(|object_def| {
                    let object_type = ObjectType::new(object_def, engine);
                    // `RegisterObjectCollectionType` (`:2205`) dedupes equivalent types so
                    // that objects shared between generators batch together. We group by mesh
                    // in the scene layer instead, so this only avoids duplicates within one
                    // generator.
                    let index = object_types
                        .iter()
                        .position(|existing| equivalent(existing, &object_type))
                        .unwrap_or_else(|| {
                            object_types.push(object_type);
                            object_types.len() - 1
                        });

                    LayerObject {
                        scale: object_def.scale,
                        scale_random_element: object_def.scale_random_element,
                        slope_fade_start: object_def.slope_fade_start,
                        slope_fade_end: object_def.slope_fade_end,
                        probability: object_def.probability,
                        theme_blend_threshold: object_def.theme_blend_threshold,
                        tilt_to_slope: object_def.tilt_to_slope,
                        object_type: index,
                    }
                })
                .collect::<Vec<_>>();

            layers.push(Layer {
                selection: Layer::build_selection_table(&objects),
                objects,
            });
        }

        let spacings: Vec<Vec<f32>> = def
            .layers
            .iter()
            .enumerate()
            .map(|(layer, layer_def)| {
                (0..=layer)
                    .map(|from| layer_def.spacing_from_layer.get(from).copied().unwrap_or(0.0))
                    .collect()
            })
            .collect();

        Generator {
            object_types,
            layers,
            grids: PlacementGrid::build_layers(&spacings),
        }
    }
}

/// `CLocalDetailObjectCollectionType::IsEquivalent` (`engine_local_detail_theme.cpp:244`) —
/// same meshes and same plain data means one type.
fn equivalent(a: &ObjectType, b: &ObjectType) -> bool {
    a.mesh == b.mesh
        && a.shadow_mesh == b.shadow_mesh
        && a.zsprite_mesh == b.zsprite_mesh
        && a.primitive_type == b.primitive_type
        && a.fade_start == b.fade_start
        && a.fade_end == b.fade_end
        && a.zsprite_fade_start == b.zsprite_fade_start
        && a.zsprite_fade_end == b.zsprite_fade_end
        && a.alpha_ref == b.alpha_ref
        && a.alpha_is_boolean == b.alpha_is_boolean
        && a.landscape_normal_lighting == b.landscape_normal_lighting
}

#[cfg(test)]
mod tests {
    use super::*;

    fn objects(probabilities: &[f32]) -> Vec<LayerObject> {
        probabilities
            .iter()
            .map(|&probability| LayerObject {
                scale: 1.0,
                scale_random_element: 0.0,
                slope_fade_start: 0.0,
                slope_fade_end: 0.0,
                probability,
                theme_blend_threshold: 0.0,
                tilt_to_slope: false,
                object_type: 0,
            })
            .collect()
    }

    fn layer(probabilities: &[f32]) -> Layer {
        let objects = objects(probabilities);
        Layer {
            selection: Layer::build_selection_table(&objects),
            objects,
        }
    }

    /// A layer with no objects is the only case that yields a negative slot, and
    /// `AddObjectsFromLayerElement` returns early on one.
    #[test]
    fn an_empty_layer_selects_nothing() {
        let layer = layer(&[]);
        for i in 0..32 {
            assert!(layer.select(i as f32 / 32.0).is_none());
        }
    }

    /// Slots are handed out in proportion to the probabilities, and every object gets some.
    #[test]
    fn slots_follow_the_probabilities() {
        // MESH_SILVER_BIRCH_01/02/03's 0.3 / 0.3 / 0.4.
        let layer = layer(&[0.3, 0.3, 0.4]);
        let mut counts = [0usize; 3];
        for slot in 0..32 {
            let object = layer.select(slot as f32 / 32.0).unwrap();
            let index = layer
                .objects
                .iter()
                .position(|o| std::ptr::eq(o, object))
                .unwrap();
            counts[index] += 1;
        }
        assert_eq!(counts.iter().sum::<usize>(), 32);
        for (i, (&count, &expected)) in counts.iter().zip([9.6, 9.6, 12.8].iter()).enumerate() {
            assert!(
                (count as f32 - expected).abs() <= 1.0,
                "object {i} got {count} slots, expected about {expected}",
            );
        }
    }

    /// The 11 shipped layers whose probabilities do not sum to 1 are renormalised, not
    /// treated as a chance of placing nothing. `LOCAL_DETAIL_GREATFIELDS_GORSE`'s two 0.4s
    /// are a 50/50, not an 80% chance of a bush.
    ///
    /// The split is 17/15 rather than 16/16, and that is the transcription being right rather
    /// than wrong: the advance fires on `cumulative * 32 < slot`, so an even two-way weight
    /// hands slot 16 to the first object and only moves on at slot 17. Every layer carries
    /// the same half-slot bias toward its first object.
    #[test]
    fn probabilities_are_relative_weights_not_absolute_chances() {
        let layer = layer(&[0.4, 0.4]);
        let mut counts = [0usize; 2];
        for slot in 0..32 {
            let object = layer.select(slot as f32 / 32.0).unwrap();
            counts[layer
                .objects
                .iter()
                .position(|o| std::ptr::eq(o, object))
                .unwrap()] += 1;
        }
        assert_eq!(counts, [17, 15]);
    }

    /// A single object owns every slot regardless of its probability.
    #[test]
    fn one_object_takes_every_slot() {
        for probability in [0.2f32, 0.5, 1.0] {
            let layer = layer(&[probability]);
            for slot in 0..32 {
                assert!(layer.select(slot as f32 / 32.0).is_some());
            }
        }
    }

    /// The draw is `floor(rand * 32)` over `[0, 1)`, so the ends must not fall off the table.
    #[test]
    fn selection_covers_the_open_unit_interval() {
        let layer = layer(&[1.0, 1.0]);
        assert!(layer.select(0.0).is_some());
        assert!(layer.select(0.999_999).is_some());
    }
}
