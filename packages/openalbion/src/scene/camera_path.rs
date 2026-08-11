//! `CTCCameraPointScriptedSpline` → a flyable camera path.
//!
//! The game ships pre-authored cutscene camera rigs — the same data the original director
//! used to frame a scene. Rather than hand-authoring flythrough waypoints, this reads those
//! rigs straight out of a level's `.tng` and turns the best one into something a camera can
//! be driven along over time.
//!
//! `KeyCameras[i].Position`/`.LookDirection` are **local to the spline's own frame**, not
//! world space: `KeyCameras[i].Position` is single digits, an offset rather than an
//! absolute point.
//!
//! `CoordBase` is confusingly *not* that frame's world-space origin directly — checked
//! against the owning thing's own `CTCPhysicsStandard` across several splines in
//! `LookoutPoint.tng`, `CoordBase` is exactly `-(map_origin + thing.Position)` (every
//! spline in the game sets `IsCoordBaseRelativeToParent TRUE`, and the name is apt: it is
//! the parent thing's world position, stored negated). So the frame's actual translation is
//! `-CoordBase`, already absolute — a level's own placements bake in its `MapX`/`MapY`
//! offset at authoring time, unlike `CTCPhysicsStandard.Position` on ordinary things, which
//! stays map-local so a filler map can be repositioned. No separate origin add here.
//!
//! `CoordAxisFwd`/`CoordAxisUp` is a `CRightHandedSet` — the same pair
//! `CTCPhysicsStandard` uses — but it is the *rig's own* orientation, independent of the
//! parent thing's facing (confirmed: they differ in every sample checked). Combined with
//! `-CoordBase` as the translation, it's exactly a [`Placement`], so
//! [`Placement::object_matrix`] — already ported from `CalcObjectMatrix` for
//! [`resolve_things`](super::things::resolve_things) — is the local→world transform here
//! too: points through it as points, `LookDirection` through it as a vector (no
//! translation).

use fable_data::tng::{CameraPoint, KeyCamera, Placement, Tng};
use glam::{Mat4, Vec3};
use std::f32::consts::TAU;

/// One control point of a flyable path, already in world space.
#[derive(Clone, Copy, Debug)]
pub struct CameraKey {
    pub position: Vec3,
    /// Unit vector.
    pub look_dir: Vec3,
    /// Horizontal FOV, radians. **Hypothesis, not verified**: the raw def value (`0.111111`,
    /// `0.2` in samples) reads as a fraction of 360°, not degrees or radians directly —
    /// `0.2 * 360 = 72°`, close to `camera_mode.def`'s 70° template default; `0.111 * 360 =
    /// 40°`, a plausible tighter dramatic shot. If that turns out wrong, only this
    /// conversion needs to change.
    pub fov_h: f32,
    pub duration: f32,
    pub pause_time: f32,
    pub roll_angle: f32,
}

/// A flyable path built from one `CTCCameraPointScriptedSpline`.
#[derive(Clone, Debug)]
pub struct CameraPath {
    pub keys: Vec<CameraKey>,
    pub tension: f32,
    /// Playback duration, seconds. **Modeling assumption**: the data alone can't
    /// disambiguate which segment a `Duration` belongs to (an 8-key spline has 8 durations
    /// but only 7 segments, all equal in every sample seen). Chosen model: segment *i* →
    /// *i+1* takes `keys[i].duration` seconds, preceded by a `keys[i].pause_time` hold at
    /// key *i*; the last key's `duration` is unused (nothing to travel to) but its
    /// `pause_time` still holds, which matters for looped playback.
    pub total_duration: f32,
}

/// Splines shorter than this aren't a flythrough — they're a two-key establishing cut.
const MIN_KEYS: usize = 3;

impl CameraPath {
    fn from_key_cameras(keys: Vec<CameraKey>, tension: f32) -> Self {
        let travel: f32 = keys[..keys.len() - 1].iter().map(|k| k.duration).sum();
        let holds: f32 = keys.iter().map(|k| k.pause_time).sum();
        CameraPath {
            keys,
            tension,
            total_duration: travel + holds,
        }
    }

    /// Total straight-line length through the world-space control points — used only to
    /// rank candidate splines against each other, not for playback.
    pub fn path_length(&self) -> f32 {
        self.keys
            .windows(2)
            .map(|w| (w[1].position - w[0].position).length())
            .sum()
    }

    /// Sample the path at `t` seconds, wrapping into `0..total_duration`. Returns
    /// `(position, look_dir, fov_h, roll_angle)`. `look_dir` is renormalized after
    /// interpolation.
    pub fn sample(&self, t: f32) -> (Vec3, Vec3, f32, f32) {
        let n = self.keys.len();
        let t = t.rem_euclid(self.total_duration.max(f32::EPSILON));

        let mut elapsed = 0.0_f32;
        for i in 0..n - 1 {
            let key = &self.keys[i];
            let hold_end = elapsed + key.pause_time;
            let travel_end = hold_end + key.duration;

            if t < hold_end {
                return (key.position, key.look_dir, key.fov_h, key.roll_angle);
            }
            // `t < hold_end` was false above, so `t >= hold_end` here — this can only be
            // reached with `key.duration == 0.0` (making `travel_end == hold_end`) when
            // `t >= travel_end` too, which fails the check below and falls through to the
            // next segment. No division by zero.
            if t < travel_end {
                let u = ((t - hold_end) / key.duration).clamp(0.0, 1.0);
                return self.sample_segment(i, u);
            }
            elapsed = travel_end;
        }

        let last = &self.keys[n - 1];
        (last.position, last.look_dir, last.fov_h, last.roll_angle)
    }

    /// Cardinal-spline (tension-parameterized Catmull-Rom) sample within segment `i` → `i+1`,
    /// `u` in `0..=1`. Position and look direction each get their own spline; the direction
    /// is renormalized afterwards since a Hermite blend of two unit vectors isn't itself
    /// unit length. FOV and roll lerp linearly — a cardinal spline is overkill for a scalar
    /// that only meaningfully has two neighbours' worth of data anyway.
    fn sample_segment(&self, i: usize, u: f32) -> (Vec3, Vec3, f32, f32) {
        let n = self.keys.len();
        let key_at =
            |idx: isize| -> &CameraKey { &self.keys[idx.clamp(0, n as isize - 1) as usize] };

        let p0 = key_at(i as isize - 1).position;
        let p1 = key_at(i as isize).position;
        let p2 = key_at(i as isize + 1).position;
        let p3 = key_at(i as isize + 2).position;
        let position = hermite(
            p1,
            p2,
            tangent(p0, p2, self.tension),
            tangent(p1, p3, self.tension),
            u,
        );

        let d0 = key_at(i as isize - 1).look_dir;
        let d1 = key_at(i as isize).look_dir;
        let d2 = key_at(i as isize + 1).look_dir;
        let d3 = key_at(i as isize + 2).look_dir;
        let look_dir = hermite(
            d1,
            d2,
            tangent(d0, d2, self.tension),
            tangent(d1, d3, self.tension),
            u,
        )
        .normalize_or(d1);

        let a = key_at(i as isize);
        let b = key_at(i as isize + 1);
        let fov_h = a.fov_h + (b.fov_h - a.fov_h) * u;
        let roll_angle = a.roll_angle + (b.roll_angle - a.roll_angle) * u;

        (position, look_dir, fov_h, roll_angle)
    }
}

/// Cardinal-spline tangent at the point between `prev` and `next`: `(1 - tension) / 2 *
/// (next - prev)`. `tension = 0` is a standard Catmull-Rom; `tension = 1` flattens to
/// straight segments through each key.
fn tangent(prev: Vec3, next: Vec3, tension: f32) -> Vec3 {
    (next - prev) * ((1.0 - tension) * 0.5)
}

/// Cubic Hermite blend of `p0`/`p1` with tangents `m0`/`m1`, `u` in `0..=1`.
fn hermite(p0: Vec3, p1: Vec3, m0: Vec3, m1: Vec3, u: f32) -> Vec3 {
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    p0 * h00 + m0 * h10 + p1 * h01 + m1 * h11
}

/// Every `CTCCameraPointScriptedSpline` in `tng` with at least [`MIN_KEYS`] key cameras,
/// world-transformed and ranked richest-first: more keys first, longer path breaking ties.
///
/// No map origin to add here — unlike ordinary things' `CTCPhysicsStandard.Position`, a
/// camera rig's `CoordBase` already bakes in the level's world placement (module docs).
pub fn find_camera_paths(tng: &Tng) -> Vec<CameraPath> {
    let mut paths: Vec<CameraPath> = tng
        .things()
        .filter_map(|thing| match &thing.components().camera_point {
            Some(CameraPoint::ScriptedSpline(spline)) => Some(spline),
            _ => None,
        })
        .filter(|spline| spline.key_cameras.len() >= MIN_KEYS)
        .map(|spline| {
            let world_anchor = -Vec3::from(spline.base.coord_base);
            let local_to_world = Mat4::from_cols_array_2d(
                &Placement {
                    position: world_anchor.into(),
                    orientation: Some(spline.base.coord_axis),
                }
                .object_matrix(1.0)
                .expect("orientation is always Some above"),
            );

            let keys = spline
                .key_cameras
                .iter()
                .map(|k: &KeyCamera| CameraKey {
                    position: local_to_world.transform_point3(Vec3::from(k.position)),
                    look_dir: local_to_world
                        .transform_vector3(Vec3::from(k.look_direction))
                        .normalize_or(Vec3::X),
                    fov_h: k.fov * TAU,
                    duration: k.duration,
                    pause_time: k.pause_time,
                    roll_angle: k.roll_angle,
                })
                .collect();

            CameraPath::from_key_cameras(keys, spline.tension)
        })
        .collect();

    paths.sort_by(rank_camera_paths);
    paths
}

/// Ranks richer paths first: more keys, then a longer path breaking ties. Exposed so a
/// caller merging candidates from several maps in a region (each already ranked by
/// [`find_camera_paths`] on its own) can re-rank the merged set the same way.
pub fn rank_camera_paths(a: &CameraPath, b: &CameraPath) -> std::cmp::Ordering {
    b.keys.len().cmp(&a.keys.len()).then_with(|| {
        b.path_length()
            .partial_cmp(&a.path_length())
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fable_data::tng::Tng;

    /// A 4-key spline with an axis-aligned `CoordAxis` and a `CoordBase` of
    /// `-(100, 200, 10)` — so the world anchor (`-CoordBase`) is `(100, 200, 10)`, a round
    /// number to check the rest of the pipeline by hand against.
    fn fixture() -> CameraPath {
        let text = "NewThing Thing;\n\
             StartCTCCameraPointScriptedSpline;\n\
             CoordBase C3DCoordF(-100.0,-200.0,-10.0);\n\
             CoordAxisUp C3DCoordF(0.0,0.0,1.0);\n\
             CoordAxisFwd C3DCoordF(1.0,0.0,0.0);\n\
             Tension 0.0;\n\
             NumKeyCameras 4;\n\
             KeyCameras[0].Position C3DCoordF(0.0,0.0,0.0);\n\
             KeyCameras[0].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[0].FOV 0.2;\n\
             KeyCameras[0].Duration 2.0;\n\
             KeyCameras[0].PauseTime 0.0;\n\
             KeyCameras[0].RollAngle 0.0;\n\
             KeyCameras[1].Position C3DCoordF(10.0,0.0,0.0);\n\
             KeyCameras[1].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].FOV 0.2;\n\
             KeyCameras[1].Duration 2.0;\n\
             KeyCameras[1].PauseTime 0.0;\n\
             KeyCameras[1].RollAngle 0.0;\n\
             KeyCameras[2].Position C3DCoordF(20.0,0.0,0.0);\n\
             KeyCameras[2].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[2].FOV 0.2;\n\
             KeyCameras[2].Duration 2.0;\n\
             KeyCameras[2].PauseTime 0.0;\n\
             KeyCameras[2].RollAngle 0.0;\n\
             KeyCameras[3].Position C3DCoordF(30.0,0.0,0.0);\n\
             KeyCameras[3].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[3].FOV 0.2;\n\
             KeyCameras[3].Duration 2.0;\n\
             KeyCameras[3].PauseTime 0.0;\n\
             KeyCameras[3].RollAngle 0.0;\n\
             EndCTCCameraPointScriptedSpline;\n\
             EndThing;\n";
        let tng = Tng::parse(text).expect("valid fixture");
        find_camera_paths(&tng)
            .into_iter()
            .next()
            .expect("one spline")
    }

    #[test]
    fn world_transform_negates_coord_base_and_skips_no_origin() {
        let path = fixture();
        // `CalcObjectMatrix`'s basis (`Placement::object_matrix`, mod.rs:228) maps object
        // +X to `Up x Forward`, not to `Forward` — with `Forward = +X`, `Up = +Z` here, that
        // cross product is `+Y`, so a local-X offset of 10 lands at world `-CoordBase +
        // (0, 10, 0)`.
        assert_eq!(path.keys[0].position, Vec3::new(100.0, 200.0, 10.0));
        assert_eq!(path.keys[1].position, Vec3::new(100.0, 210.0, 10.0));
        // Local look direction (1,0,0) rotates through the same basis to world +Y.
        assert_eq!(path.keys[0].look_dir, Vec3::Y);
    }

    #[test]
    fn sample_at_the_ends_matches_the_first_and_last_key() {
        let path = fixture();
        let (p0, d0, fov0, _) = path.sample(0.0);
        assert_eq!(p0, path.keys[0].position);
        assert_eq!(d0, path.keys[0].look_dir);
        assert_eq!(fov0, path.keys[0].fov_h);

        // `total_duration` itself wraps back to the start (that's what makes looped
        // playback seamless — see `sample_wraps_for_looped_playback`), so check just short
        // of it instead.
        let (p_end, _, _, _) = path.sample(path.total_duration - 0.001);
        assert!((p_end - path.keys[3].position).length() < 1e-2);
    }

    #[test]
    fn sample_wraps_for_looped_playback() {
        let path = fixture();
        let (looped, _, _, _) = path.sample(path.total_duration + 0.001);
        let (start, _, _, _) = path.sample(0.001);
        assert!((looped - start).length() < 1e-3);
    }

    #[test]
    fn short_splines_are_filtered_out() {
        let text = "NewThing Thing;\n\
             StartCTCCameraPointScriptedSpline;\n\
             CoordBase C3DCoordF(0.0,0.0,0.0);\n\
             CoordAxisUp C3DCoordF(0.0,0.0,1.0);\n\
             CoordAxisFwd C3DCoordF(1.0,0.0,0.0);\n\
             NumKeyCameras 2;\n\
             KeyCameras[0].Position C3DCoordF(0.0,0.0,0.0);\n\
             KeyCameras[0].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].Position C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             EndCTCCameraPointScriptedSpline;\n\
             EndThing;\n";
        let tng = Tng::parse(text).expect("valid fixture");
        assert!(find_camera_paths(&tng).is_empty());
    }

    #[test]
    fn richer_splines_rank_first() {
        let text = "NewThing Thing;\n\
             StartCTCCameraPointScriptedSpline;\n\
             CoordBase C3DCoordF(0.0,0.0,0.0);\n\
             CoordAxisUp C3DCoordF(0.0,0.0,1.0);\n\
             CoordAxisFwd C3DCoordF(1.0,0.0,0.0);\n\
             NumKeyCameras 3;\n\
             KeyCameras[0].Position C3DCoordF(0.0,0.0,0.0);\n\
             KeyCameras[0].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].Position C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[2].Position C3DCoordF(2.0,0.0,0.0);\n\
             KeyCameras[2].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             EndCTCCameraPointScriptedSpline;\n\
             EndThing;\n\
             NewThing Thing;\n\
             StartCTCCameraPointScriptedSpline;\n\
             CoordBase C3DCoordF(0.0,0.0,0.0);\n\
             CoordAxisUp C3DCoordF(0.0,0.0,1.0);\n\
             CoordAxisFwd C3DCoordF(1.0,0.0,0.0);\n\
             NumKeyCameras 4;\n\
             KeyCameras[0].Position C3DCoordF(0.0,0.0,0.0);\n\
             KeyCameras[0].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].Position C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[1].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[2].Position C3DCoordF(2.0,0.0,0.0);\n\
             KeyCameras[2].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             KeyCameras[3].Position C3DCoordF(3.0,0.0,0.0);\n\
             KeyCameras[3].LookDirection C3DCoordF(1.0,0.0,0.0);\n\
             EndCTCCameraPointScriptedSpline;\n\
             EndThing;\n";
        let tng = Tng::parse(text).expect("valid fixture");
        let paths = find_camera_paths(&tng);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].keys.len(), 4);
    }
}
