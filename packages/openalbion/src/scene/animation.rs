//! Choosing and playing a creature's animation.
//!
//! A creature's clips are data: `APPEARANCE`'s `Animation` is an `AnimationSet`, and each
//! `AnimationEntry` carries a **`bank_index` that is a `graphics.big` asset id** — the same
//! shape `Graphic.BankIndex` has for meshes (AGENTS.md §3.11). Measured across retail:
//! **23,005 of 23,005 entries resolve** to real animation assets, over 151 of 174 `APPEARANCE`
//! defs.
//!
//! Every set is keyed by a hash, and one key recurs across all of them:
//!
//! ```text
//! key 1098326459 -> ANIMATION_ARENA_GATE_DEFAULT_01
//!                   ANIM_ARENA_AUDIENCE_IDLE_BREATHE_01
//!                   ANIMATION_DUMMY_DEFAULT_01
//! ```
//!
//! — the **default/idle clip**, which is what a creature standing around would play.
//!
//! What is *not* here: any notion of choosing an animation because of what a creature is
//! *doing*. There is no AI and no state machine (§5 step 6.7c). Until there is, a creature
//! draws a clip at random from its def's whole set — `--idle-animations` restricts it to the
//! idle, and `--animation NAME` forces one clip on everything. Random is the default because a
//! level of villagers all breathing in place shows almost none of what the animation data
//! holds.

use fable_data::anim::Animation;
use fable_data::def::binary::{DefBinary, DefBody};
use fable_data::def::names::Names;
use std::collections::HashMap;

/// The `AnimationSet` key that names a creature's default/idle clip.
///
/// Read out of the shipped data rather than guessed: it is the only key shared by the arena
/// gate, the arena audience and the training dummy, and in each case it names that def's
/// `*_DEFAULT_01` or `*_IDLE_*` entry.
pub const DEFAULT_ANIMATION_KEY: u32 = 1_098_326_459;

/// Every clip one thing def can play, and which of them is its idle.
#[derive(Debug, Clone, Default)]
pub struct DefAnimations {
    /// The entry under [`DEFAULT_ANIMATION_KEY`], when the set has one.
    pub default: Option<i32>,
    /// Every clip in the set, in the order the def lists them.
    pub all: Vec<i32>,
}

impl DefAnimations {
    /// The idle clip, or the first the def lists.
    pub fn idle(&self) -> Option<i32> {
        self.default.or_else(|| self.all.first().copied())
    }

    /// One clip chosen by `seed`, through the engine's own PRNG (§3.13) so the choice is
    /// reproducible run to run rather than merely arbitrary.
    pub fn random(&self, seed: u32) -> Option<i32> {
        if self.all.is_empty() {
            return None;
        }
        let draw = fable_data::local_detail::rng::next_seed(seed);
        self.all.get(draw as usize % self.all.len()).copied()
    }
}

/// Which animations each `DefinitionType` can play.
pub type DefaultAnimations = HashMap<String, DefAnimations>;

/// Join every thing def to its `APPEARANCE` sub-def and take the default clip.
///
/// Sub-defs are separate entries in the global index space, reached through the parent's
/// `SubDefRecord::def_index` — the same indirection `Files::read_creature_body_parts` walks.
pub fn read_default_animations(names: &Names, def_binary: &DefBinary) -> DefaultAnimations {
    // Every APPEARANCE sub-def's clips, by the global index its parent names.
    let mut by_index: HashMap<u32, DefAnimations> = HashMap::new();
    for entry in def_binary.entries(names) {
        let DefBody::AppearanceDef(def) = &entry.record.body else {
            continue;
        };
        let set = &def.animation;
        if set.anims.is_empty() {
            continue;
        }
        let all: Vec<i32> = set
            .anims
            .iter()
            .map(|a| a.entry.bank_index)
            .filter(|&b| b > 0)
            .collect();
        if all.is_empty() {
            continue;
        }
        let default = set
            .anims
            .iter()
            .find(|a| a.key == DEFAULT_ANIMATION_KEY)
            .map(|a| a.entry.bank_index)
            .filter(|&b| b > 0);
        by_index.insert(entry.global_index as u32, DefAnimations { default, all });
    }

    let mut by_name = HashMap::new();
    for entry in def_binary.entries(names) {
        if !matches!(
            &entry.record.body,
            DefBody::ThingCreatureDef(_) | DefBody::ThingObjectDef(_)
        ) {
            continue;
        }
        let (Some(name), Some(sub_defs)) = (entry.file_name, entry.record.sub_defs.as_ref()) else {
            continue;
        };
        if let Some(set) = sub_defs.iter().find_map(|s| by_index.get(&s.def_index)) {
            by_name.insert(name.to_string(), set.clone());
        }
    }
    by_name
}

/// A clip bound to one mesh, ready to pose every frame.
///
/// The retargeting map is resolved **once**, at load: matching 62 sequence names against 64
/// bone names for every instance of every creature, sixty times a second, would be pure waste.
pub struct BoundAnimation {
    pub animation: Animation,
    /// Which object of the animation to play — always the first in the shipped data.
    pub object: usize,
    /// `bone -> sequence`, or `None` where the clip does not drive that bone.
    pub bone_to_sequence: Vec<Option<usize>>,
    /// `object_to_bone[bone]`, precomputed in row-vector form.
    pub object_to_bone: Vec<fable_data::skeleton::Mat4>,
}

impl BoundAnimation {
    /// Bind `animation` to `mesh`, resolving the name-based retargeting once.
    pub fn bind(mesh: &fable_data::mesh::Mesh, animation: Animation) -> Option<BoundAnimation> {
        let object = animation.objects.first()?;
        let names = fable_data::skeleton::mesh_bone_names(mesh);

        let bone_to_sequence = names
            .iter()
            .map(|name| object.sequences.iter().position(|s| &s.bone_name == name))
            .collect();
        let object_to_bone = mesh
            .bone_transforms
            .iter()
            .map(fable_data::skeleton::object_to_bone)
            .collect();

        Some(BoundAnimation {
            animation,
            object: 0,
            bone_to_sequence,
            object_to_bone,
        })
    }

    pub fn frame_count(&self) -> u32 {
        self.animation.frame_count().max(1)
    }

    /// Which frame `seconds` lands on, looping.
    pub fn frame_at(&self, seconds: f32) -> usize {
        let rate = self.animation.samples_per_second().max(1.0);
        let frames = self.frame_count();
        let index = (seconds * rate) as i64;
        index.rem_euclid(frames as i64) as usize
    }

    /// The skinning matrices for one instance at `frame`, in mesh-bone order.
    ///
    /// Bones the clip does not drive keep the identity, leaving them in bind pose.
    pub fn matrices(&self, frame: usize) -> Vec<fable_data::skeleton::Mat4> {
        use fable_data::skeleton::{multiply, pose_object, IDENTITY};

        let object = &self.animation.objects[self.object];
        let world = pose_object(object, frame);

        self.bone_to_sequence
            .iter()
            .enumerate()
            .map(|(bone, sequence)| match sequence {
                Some(s) if bone < self.object_to_bone.len() => {
                    multiply(&self.object_to_bone[bone], &world[*s])
                }
                _ => IDENTITY,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fable_data::anim::{AnimObject, Sequence, TrackMode};

    fn clip(frames: u32, rate: f32) -> Animation {
        Animation {
            looping: true,
            duration: frames as f32 / rate,
            objects: vec![AnimObject {
                name: "test".into(),
                sequences: vec![Sequence {
                    bone_name: "Bip01".into(),
                    parent_index: -1,
                    bone_type: 0,
                    enabled: true,
                    samples_per_second: rate,
                    frame_count: frames,
                    rotation_mode: TrackMode::Identity,
                    position_mode: TrackMode::Identity,
                    scaling_mode: TrackMode::Identity,
                    position_factor: 1.0,
                    scaling_factor: 1.0,
                    rotations: vec![],
                    rotation_palette: vec![],
                    positions: vec![],
                    position_palette: vec![],
                }],
            }],
        }
    }

    fn bound(frames: u32, rate: f32) -> BoundAnimation {
        BoundAnimation {
            animation: clip(frames, rate),
            object: 0,
            bone_to_sequence: vec![Some(0)],
            object_to_bone: vec![fable_data::skeleton::IDENTITY],
        }
    }

    /// Time advances at the clip's own rate, and wraps rather than running off the end.
    #[test]
    fn playback_loops_at_the_clips_rate() {
        let b = bound(30, 30.0);
        assert_eq!(b.frame_at(0.0), 0);
        assert_eq!(b.frame_at(0.5), 15);
        assert_eq!(b.frame_at(1.0), 0, "one second of a one-second clip wraps");
        assert_eq!(b.frame_at(1.5), 15);
    }

    /// A negative offset must wrap forwards, not index backwards — instances are spread in time
    /// by subtracting an offset, and a bare `%` would give a negative frame.
    #[test]
    fn negative_times_wrap_forwards() {
        let b = bound(30, 30.0);
        assert_eq!(b.frame_at(-0.5), 15);
        assert_eq!(b.frame_at(-1.0 / 30.0), 29);
    }

    /// A slower clip advances more slowly. 15 fps is the second most common rate in the data.
    #[test]
    fn the_clips_own_rate_is_used() {
        let b = bound(15, 15.0);
        assert_eq!(b.frame_at(0.5), 7);
        assert_eq!(b.frame_at(1.0), 0);
    }

    /// An undriven bone stays in bind pose.
    #[test]
    fn undriven_bones_get_the_identity() {
        let mut b = bound(2, 30.0);
        b.bone_to_sequence = vec![Some(0), None];
        b.object_to_bone = vec![fable_data::skeleton::IDENTITY; 2];
        let m = b.matrices(0);
        assert_eq!(m.len(), 2);
        assert_eq!(m[1], fable_data::skeleton::IDENTITY);
    }
}
