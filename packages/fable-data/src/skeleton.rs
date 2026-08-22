//! Posing a skinned mesh — the bind pose, the animated pose, and the matrix that maps between.
//!
//! This is the piece §5 item 6.7a was blocked on for three sessions, and the block was a
//! misread matrix convention rather than missing data.
//!
//! ## `Mesh::bone_transforms` is the inverse bind matrix, stored transposed
//!
//! `C3DMesh2::BoneSpaceTransforms` is a `CMatrix4x4` array whose accessor is
//! **`PeekTransposedObjectToBoneSpaceTransform`** (`bbblibrary/lib_3d_mesh_2.hpp:1105,1140`),
//! and the data agrees: the stored matrix's last **row** is `(0,0,0,1)` on 15,022 of 15,022
//! bones, so it is a *column-vector* affine matrix whose translation sits at indices 3, 7, 11.
//! Transposing it gives the row-vector object→bone matrix this codebase uses everywhere else.
//!
//! Inverting *that* gives the bind pose, and the bind pose is a standing person — which is the
//! check that settles it, because no wrong reading produces a plausible skeleton:
//!
//! ```text
//! MESH_BS_MALE_MIDDLE_UNCLOTHED_01 (mesh is 194 units tall)
//!   Bip01 Pelvis  ( -0.3,  6.0,  96.5)     Bip01 L Foot  ( 16.5, 6.7,  18.8)
//!   Bip01 Spine   ( -0.3,  6.0, 105.7)     Bip01 R Hand  (-50.7, 7.2, 107.4)
//!   Bip01 Head    ( -0.3,  5.6, 164.8)
//! ```
//!
//! ## The animation composes child-first, and bone lengths prove it
//!
//! A `Sequence`'s stored translation is the bone's length along its own local X — the 3ds Max
//! Biped convention — so the pose is `world_child = local_child · world_parent` in row-vector
//! order. Tested against the invariant that animation *rotates* bones and never stretches
//! them: each bone's distance to its parent must equal the bind skeleton's, at any frame.
//! `child · parent` gives a **median error of 0.0000 on 50 of 52 bones**; `parent · child`
//! gives 7.6 and 3 of 52.
//!
//! ## Putting it together
//!
//! ```text
//! skin[b]   = object_to_bone[b] · world[b]        // into bone space, then out to the pose
//! vertex'   = Σ weightₖ · (vertex · skin[palette[slotₖ]])
//! ```

use crate::anim::{AnimObject, Sequence};
use crate::mesh::Mesh;

/// A row-vector 4×4, matching the convention used across this codebase (AGENTS.md §3.11).
pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

pub fn multiply(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut r = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

/// The object→bone (inverse bind) matrix for one bone, in row-vector form.
///
/// The file stores it transposed — see the module note.
pub fn object_to_bone(stored: &[f32; 16]) -> Mat4 {
    [
        [stored[0], stored[4], stored[8], stored[12]],
        [stored[1], stored[5], stored[9], stored[13]],
        [stored[2], stored[6], stored[10], stored[14]],
        [stored[3], stored[7], stored[11], stored[15]],
    ]
}

/// A rotation-and-translation matrix from a quaternion `(x, y, z, w)` and a translation.
///
/// The quaternion is renormalised: 5 of the archive's 4,737,597 stored quaternions are up to
/// 5% off unit length, never having been renormalised by the exporter, and an un-normalised
/// quaternion scales the bone it drives.
pub fn from_rotation_translation(q: [f32; 4], t: [f32; 3]) -> Mat4 {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    let inv = if n > 1e-9 { 1.0 / n } else { 1.0 };
    let (x, y, z, w) = (q[0] * inv, q[1] * inv, q[2] * inv, q[3] * inv);
    let (xx, yy, zz) = (x * x, y * y, z * z);
    let (xy, xz, yz) = (x * y, x * z, y * z);
    let (wx, wy, wz) = (w * x, w * y, w * z);
    [
        [1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0],
        [2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0],
        [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0],
        [t[0], t[1], t[2], 1.0],
    ]
}

/// One sequence's local matrix at a frame. An absent track contributes the identity.
///
/// **The stored quaternion is conjugated on the way in**, which is the same transpose
/// `object_to_bone` applies: the file is column-vector throughout, and this codebase is
/// row-vector (AGENTS.md §3.11). Conjugating a unit quaternion transposes its rotation matrix,
/// so this is one convention change stated once rather than a sign flip buried in the maths.
///
/// It cannot be caught by any rigid-motion check — a transposed rotation is still a rotation,
/// so bone lengths survive it intact. What catches it is the *skinned* silhouette:
/// `ANIM_VILLAGER_FEAR_WALK_02` on the villager body spans Z −0.2..193.6 conjugated, matching
/// the mesh's own −0.7..193.6 with the feet on the ground; unconjugated it is 71 units tall
/// and inflated to 204 wide.
pub fn local_pose(sequence: &Sequence, frame: usize) -> Mat4 {
    let q = sequence.rotation_at(frame).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    from_rotation_translation(
        [-q[0], -q[1], -q[2], q[3]],
        sequence.position_at(frame).unwrap_or([0.0, 0.0, 0.0]),
    )
}

/// Model-space matrices for every sequence at `frame`, composed down the parent chain.
///
/// `world_child = local_child · world_parent`, in sequence order — the file lists parents
/// before children on every shipped animation, so one forward pass suffices.
pub fn pose_object(object: &AnimObject, frame: usize) -> Vec<Mat4> {
    let mut world: Vec<Mat4> = Vec::with_capacity(object.sequences.len());
    for (i, sequence) in object.sequences.iter().enumerate() {
        let local = local_pose(sequence, frame);
        let parent = sequence.parent_index;
        let parent = if parent < 0 || parent as usize >= i {
            IDENTITY
        } else {
            world[parent as usize]
        };
        world.push(multiply(&local, &parent));
    }
    world
}

/// The bone names a mesh carries, in bone order.
///
/// `bone_name_indices` holds the *end* offset of each of the first `bone_count - 1` names
/// within the `bone_names` blob; the last name runs to the end.
pub fn mesh_bone_names(mesh: &Mesh) -> Vec<String> {
    let blob = &mesh.bone_names;
    let mut starts: Vec<usize> = vec![0];
    starts.extend(mesh.bone_name_indices.iter().map(|&i| i as usize));
    starts
        .iter()
        .enumerate()
        .map(|(k, &start)| {
            let end = starts.get(k + 1).copied().unwrap_or(blob.len()).min(blob.len());
            if start <= end {
                String::from_utf8_lossy(&blob[start..end])
                    .trim_end_matches('\0')
                    .to_string()
            } else {
                String::new()
            }
        })
        .collect()
}

/// The skinning matrix per **mesh bone**, ready for a shader's bone palette.
///
/// Bones the animation does not drive keep the identity, which leaves them in bind pose rather
/// than collapsing them to the origin — a missing bone should look unanimated, not broken.
///
/// Retargeting is by name: an animation carries its own skeleton (`male_villager_complete`,
/// `HeroUnclothed`, …) and binds to a mesh through matching Biped bone names.
pub fn skinning_matrices(mesh: &Mesh, object: &AnimObject, frame: usize) -> Vec<Mat4> {
    let names = mesh_bone_names(mesh);
    let world = pose_object(object, frame);

    let mut matrices = vec![IDENTITY; mesh.bones.len()];
    for (i, sequence) in object.sequences.iter().enumerate() {
        let Some(bone) = names.iter().position(|n| n == &sequence.bone_name) else {
            continue;
        };
        if bone >= mesh.bone_transforms.len() || bone >= matrices.len() {
            continue;
        }
        matrices[bone] = multiply(&object_to_bone(&mesh.bone_transforms[bone]), &world[i]);
    }
    matrices
}

/// How many of a mesh's bones an animation actually drives — the retargeting hit rate, which
/// is worth logging rather than assuming.
pub fn retarget_coverage(mesh: &Mesh, object: &AnimObject) -> (usize, usize) {
    let names = mesh_bone_names(mesh);
    let matched = object
        .sequences
        .iter()
        .filter(|s| names.iter().any(|n| n == &s.bone_name))
        .count();
    (matched, object.sequences.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_quaternion_gives_a_pure_translation() {
        let m = from_rotation_translation([0.0, 0.0, 0.0, 1.0], [1.0, 2.0, 3.0]);
        assert_eq!(m[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m[3], [1.0, 2.0, 3.0, 1.0]);
    }

    /// A quarter turn about Z takes +X to +Y in the row-vector convention.
    #[test]
    fn a_quarter_turn_about_z_rotates_x_to_y() {
        let r = std::f32::consts::FRAC_1_SQRT_2;
        let m = from_rotation_translation([0.0, 0.0, r, r], [0.0; 3]);
        // Row 0 is where the +X basis vector lands.
        assert!((m[0][0]).abs() < 1e-5, "{:?}", m[0]);
        assert!((m[0][1] - 1.0).abs() < 1e-5, "{:?}", m[0]);
    }

    /// An un-normalised quaternion must not scale the bone.
    #[test]
    fn quaternions_are_renormalised() {
        let m = from_rotation_translation([0.0, 0.0, 0.0, 2.0], [0.0; 3]);
        assert!((m[0][0] - 1.0).abs() < 1e-5, "scaled: {:?}", m[0]);
    }

    /// `multiply` is row-vector: a point times (child · parent) applies the child first.
    #[test]
    fn composition_applies_the_child_first() {
        let child = from_rotation_translation([0.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let r = std::f32::consts::FRAC_1_SQRT_2;
        let parent = from_rotation_translation([0.0, 0.0, r, r], [0.0; 3]);
        let composed = multiply(&child, &parent);
        // The child's +1 along X is rotated by the parent into +1 along Y.
        assert!((composed[3][0]).abs() < 1e-5, "{:?}", composed[3]);
        assert!((composed[3][1] - 1.0).abs() < 1e-5, "{:?}", composed[3]);
    }

    /// The transpose is not decorative: it moves the translation from the stored column-vector
    /// slots (3, 7, 11) into the row-vector row.
    #[test]
    fn object_to_bone_transposes_the_stored_matrix() {
        let mut stored = [0.0f32; 16];
        stored[0] = 1.0;
        stored[5] = 1.0;
        stored[10] = 1.0;
        stored[15] = 1.0;
        stored[3] = 7.0; // translation X, in the stored column-vector layout
        stored[7] = 8.0;
        stored[11] = 9.0;
        let m = object_to_bone(&stored);
        assert_eq!(m[3], [7.0, 8.0, 9.0, 1.0]);
    }
}
