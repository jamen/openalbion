//! The bind pose and the animated pose, checked against the shipped data.
//!
//! These are the measurements that closed §5 item 6.7a. Two of them are the load-bearing ones:
//! the bind skeleton has to *be a standing person*, and animation has to move bones without
//! stretching them.

use fable_data::anim::Animation;
use fable_data::big::{BigReader, ExtraMetadata};
use fable_data::mesh::Mesh;
use fable_data::skeleton::{
    mesh_bone_names, multiply, object_to_bone, pose_object, retarget_coverage, skinning_matrices,
};
use std::fs::File;
use std::path::PathBuf;

fn archive() -> Option<BigReader<File>> {
    let path = PathBuf::from("/home/jamen/Fable/data/graphics/graphics.big");
    path.exists()
        .then(|| BigReader::new(File::open(&path).unwrap()).unwrap())
}

fn read_mesh(big: &mut BigReader<File>, name: &str) -> Mesh {
    let asset = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .find(|a| a.symbol_name == name && matches!(&a.extras, Some(ExtraMetadata::Mesh(_))))
        .cloned()
        .unwrap_or_else(|| panic!("{name} is not in the archive"));
    Mesh::decode(&big.read_asset_from_metadata(&asset).unwrap()).unwrap()
}

fn first_animation_on(big: &mut BigReader<File>, skeleton: &str) -> (String, Animation) {
    let ids: Vec<(u32, String)> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Animation(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();
    for (id, name) in ids {
        let asset = big
            .bank_iter()
            .flat_map(|b| b.asset_iter())
            .find(|a| a.id == id)
            .unwrap()
            .clone();
        let Ok(d) = big.read_asset_from_metadata(&asset) else {
            continue;
        };
        let Ok(anim) = Animation::decode(&d) else { continue };
        if anim.objects.first().is_some_and(|o| o.name == skeleton) {
            return (name, anim);
        }
    }
    panic!("no animation on skeleton {skeleton}");
}

fn affine_inverse(m: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let a = [
        [m[0][0], m[0][1], m[0][2]],
        [m[1][0], m[1][1], m[1][2]],
        [m[2][0], m[2][1], m[2][2]],
    ];
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    let id = 1.0 / det;
    let inv = [
        [
            (a[1][1] * a[2][2] - a[1][2] * a[2][1]) * id,
            (a[0][2] * a[2][1] - a[0][1] * a[2][2]) * id,
            (a[0][1] * a[1][2] - a[0][2] * a[1][1]) * id,
        ],
        [
            (a[1][2] * a[2][0] - a[1][0] * a[2][2]) * id,
            (a[0][0] * a[2][2] - a[0][2] * a[2][0]) * id,
            (a[0][2] * a[1][0] - a[0][0] * a[1][2]) * id,
        ],
        [
            (a[1][0] * a[2][1] - a[1][1] * a[2][0]) * id,
            (a[0][1] * a[2][0] - a[0][0] * a[2][1]) * id,
            (a[0][0] * a[1][1] - a[0][1] * a[1][0]) * id,
        ],
    ];
    let t = [m[3][0], m[3][1], m[3][2]];
    [
        [inv[0][0], inv[0][1], inv[0][2], 0.0],
        [inv[1][0], inv[1][1], inv[1][2], 0.0],
        [inv[2][0], inv[2][1], inv[2][2], 0.0],
        [
            -(t[0] * inv[0][0] + t[1] * inv[1][0] + t[2] * inv[2][0]),
            -(t[0] * inv[0][1] + t[1] * inv[1][1] + t[2] * inv[2][1]),
            -(t[0] * inv[0][2] + t[1] * inv[1][2] + t[2] * inv[2][2]),
            1.0,
        ],
    ]
}

/// **The test that settles the convention.** Inverting the object→bone matrix must give a
/// skeleton that is a standing person inside the mesh — feet low, pelvis mid, head high,
/// hands out to the side. No wrong reading of these bytes produces that.
#[test]
fn the_bind_skeleton_is_a_standing_person() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");
    let names = mesh_bone_names(&mesh);

    let height = mesh.bounding_box.max[2] - mesh.bounding_box.min[2];
    assert!((height - 194.0).abs() < 2.0, "mesh is {height} tall");

    let bind_position = |bone: &str| -> [f32; 3] {
        let i = names
            .iter()
            .position(|n| n == bone)
            .unwrap_or_else(|| panic!("no bone {bone}"));
        let inv = affine_inverse(&object_to_bone(&mesh.bone_transforms[i]));
        [inv[3][0], inv[3][1], inv[3][2]]
    };

    let pelvis = bind_position("Bip01 Pelvis");
    let head = bind_position("Bip01 Head");
    let foot = bind_position("Bip01 L Foot");
    let hand = bind_position("Bip01 R Hand");

    // Heights, as fractions of the mesh: feet near the ground, head near the top.
    assert!((10.0..40.0).contains(&foot[2]), "foot at Z {}", foot[2]);
    assert!((80.0..115.0).contains(&pelvis[2]), "pelvis at Z {}", pelvis[2]);
    assert!((150.0..185.0).contains(&head[2]), "head at Z {}", head[2]);
    assert!(head[2] > pelvis[2] && pelvis[2] > foot[2], "the skeleton is upside down");

    // A hand is off to one side, and the body is centred on X.
    assert!(hand[0].abs() > 30.0, "hand at X {}", hand[0]);
    assert!(pelvis[0].abs() < 5.0 && head[0].abs() < 5.0, "the spine is off-centre");

    // Every *skeletal* bone lands inside the mesh's own bounding volume, with a little slack.
    // Attachment dummies are deliberately excluded: `WEAPON_FOCUS_ARROW_IN_BACK` marks where a
    // protruding arrow goes and sits 80 units behind a body only 40 deep, which is correct.
    for (i, name) in names.iter().enumerate() {
        if i >= mesh.bone_transforms.len() {
            break;
        }
        if !name.starts_with("Bip01") {
            continue;
        }
        let inv = affine_inverse(&object_to_bone(&mesh.bone_transforms[i]));
        for axis in 0..3 {
            let v = inv[3][axis];
            let (lo, hi) = (mesh.bounding_box.min[axis], mesh.bounding_box.max[axis]);
            let slack = (hi - lo) * 0.35 + 5.0;
            assert!(
                v >= lo - slack && v <= hi + slack,
                "bone {name} axis {axis} at {v}, outside {lo}..{hi}"
            );
        }
    }
}

/// **The test that settles the composition order.** Animation rotates bones; it never
/// stretches them. So a bone's distance to its parent must equal the bind skeleton's, at every
/// frame. `child · parent` satisfies this; `parent · child` misses by a factor of several.
#[test]
fn animation_moves_bones_without_stretching_them() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");
    let names = mesh_bone_names(&mesh);
    let (anim_name, anim) = first_animation_on(&mut big, "male_villager_complete");
    let object = &anim.objects[0];

    let bind = |bone: &str| -> Option<[f32; 3]> {
        let i = names.iter().position(|n| n == bone)?;
        if i >= mesh.bone_transforms.len() {
            return None;
        }
        let inv = affine_inverse(&object_to_bone(&mesh.bone_transforms[i]));
        Some([inv[3][0], inv[3][1], inv[3][2]])
    };

    let mut checked = 0usize;
    let mut within_2_percent = 0usize;
    let mut worst = 0.0f32;

    for frame in [0usize, 1, 5] {
        let world = pose_object(object, frame);
        for (i, sequence) in object.sequences.iter().enumerate() {
            let parent = sequence.parent_index;
            if parent < 0 || parent as usize >= i {
                continue;
            }
            let parent_name = &object.sequences[parent as usize].bone_name;
            let (Some(b), Some(bp)) = (bind(&sequence.bone_name), bind(parent_name)) else {
                continue;
            };
            let bind_len = distance(b, bp);
            if bind_len < 1.0 {
                continue;
            }
            let a = world[i];
            let ap = world[parent as usize];
            let anim_len = distance(
                [a[3][0], a[3][1], a[3][2]],
                [ap[3][0], ap[3][1], ap[3][2]],
            );
            let error = (anim_len - bind_len).abs() / bind_len;
            checked += 1;
            if error < 0.02 {
                within_2_percent += 1;
            }
            worst = worst.max(error);
        }
    }

    eprintln!("{anim_name}: {within_2_percent} of {checked} bone lengths within 2%, worst {worst:.4}");
    assert!(checked > 100, "only {checked} bone pairs checked");
    assert!(
        within_2_percent * 100 >= checked * 90,
        "only {within_2_percent} of {checked} bone lengths survive the pose - the composition \
         order is wrong"
    );
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Retargeting by name has to actually hit. A villager clip drives most of the villager mesh.
#[test]
fn animations_retarget_onto_the_mesh_by_name() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");
    let (_, anim) = first_animation_on(&mut big, "male_villager_complete");
    let (matched, total) = retarget_coverage(&mesh, &anim.objects[0]);
    eprintln!("retargeted {matched} of {total} sequences onto {} bones", mesh.bones.len());
    assert!(
        matched * 100 >= total * 85,
        "only {matched} of {total} sequences found a bone"
    );
}

/// A bone the animation does not drive keeps the identity, so it stays in bind pose rather
/// than collapsing to the origin — a partially-retargeted mesh should look unanimated, not
/// exploded.
#[test]
fn undriven_bones_keep_the_identity() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");
    let (_, anim) = first_animation_on(&mut big, "male_villager_complete");
    let matrices = skinning_matrices(&mesh, &anim.objects[0], 0);
    assert_eq!(matrices.len(), mesh.bones.len());

    let names = mesh_bone_names(&mesh);
    let driven: Vec<bool> = names
        .iter()
        .map(|n| anim.objects[0].sequences.iter().any(|s| &s.bone_name == n))
        .collect();

    for (i, &is_driven) in driven.iter().enumerate() {
        if !is_driven && i < matrices.len() {
            assert_eq!(
                matrices[i],
                fable_data::skeleton::IDENTITY,
                "undriven bone {} was not left alone",
                names[i]
            );
        }
    }
}

/// The skinning matrix must be the identity where the animated pose equals the bind pose.
/// Feeding the bind pose back in is the one case where the answer is known exactly, so it is
/// the strongest check on `object_to_bone` that does not depend on any clip.
#[test]
fn the_bind_pose_skins_to_the_identity() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");

    let mut worst = 0.0f32;
    for i in 0..mesh.bones.len().min(mesh.bone_transforms.len()) {
        let otb = object_to_bone(&mesh.bone_transforms[i]);
        // The bind world matrix is, by definition, the inverse of object->bone.
        let bind_world = affine_inverse(&otb);
        let skin = multiply(&otb, &bind_world);
        for r in 0..4 {
            for c in 0..4 {
                let want = if r == c { 1.0 } else { 0.0 };
                worst = worst.max((skin[r][c] - want).abs());
            }
        }
    }
    assert!(worst < 1e-3, "bind pose does not skin to the identity: worst {worst}");
}

/// **The test that settles the rotation convention**, and the only one that can: a transposed
/// rotation is still a rotation, so every rigid-motion check (bone lengths included) survives
/// it intact. What does not survive is the *silhouette*.
///
/// Skinning a walking villager must reproduce the mesh's own height with the feet on the
/// ground. Conjugated, `ANIM_VILLAGER_FEAR_WALK_02` spans Z −0.2..193.6 against the mesh's
/// −0.7..193.6. Unconjugated it is 71 units tall and 204 wide — a person turned inside out.
#[test]
fn skinning_reproduces_the_meshs_own_silhouette() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let mesh = read_mesh(&mut big, "MESH_BS_MALE_MIDDLE_UNCLOTHED_01");

    let asset = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .find(|a| a.symbol_name == "ANIM_VILLAGER_FEAR_WALK_02")
        .cloned()
        .expect("the walk clip is in the archive");
    let anim = Animation::decode(&big.read_asset_from_metadata(&asset).unwrap()).unwrap();
    let matrices = skinning_matrices(&mesh, &anim.objects[0], 0);

    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    let mut skinned = 0usize;
    for primitive in &mesh.primitives {
        if primitive.blends.is_empty() {
            continue;
        }
        let Some(palette) = primitive.animated_blocks.first().map(|b| &b.groups) else {
            continue;
        };
        for (vertex, blend) in primitive.vertices.iter().zip(&primitive.blends) {
            let (slots, weights) = (blend.slots(), blend.weights());
            let mut acc = [0.0f32; 3];
            for k in 0..3 {
                if weights[k] <= 0.0 {
                    continue;
                }
                let Some(&bone) = palette.get(slots[k] as usize) else { continue };
                let Some(m) = matrices.get(bone as usize) else { continue };
                let p = [
                    vertex.pos[0] * m[0][0] + vertex.pos[1] * m[1][0] + vertex.pos[2] * m[2][0] + m[3][0],
                    vertex.pos[0] * m[0][1] + vertex.pos[1] * m[1][1] + vertex.pos[2] * m[2][1] + m[3][1],
                    vertex.pos[0] * m[0][2] + vertex.pos[1] * m[1][2] + vertex.pos[2] * m[2][2] + m[3][2],
                ];
                for c in 0..3 {
                    acc[c] += p[c] * weights[k];
                }
            }
            skinned += 1;
            for c in 0..3 {
                lo[c] = lo[c].min(acc[c]);
                hi[c] = hi[c].max(acc[c]);
            }
        }
    }

    eprintln!(
        "skinned {skinned} vertices: X {:.1}..{:.1} Y {:.1}..{:.1} Z {:.1}..{:.1}",
        lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]
    );

    let height = hi[2] - lo[2];
    let mesh_height = mesh.bounding_box.max[2] - mesh.bounding_box.min[2];
    assert!(skinned > 2000, "only {skinned} vertices skinned");
    assert!(
        (height - mesh_height).abs() < 8.0,
        "skinned height {height:.1} should match the mesh's {mesh_height:.1}"
    );
    assert!(lo[2].abs() < 5.0, "the feet should be on the ground, not at Z {:.1}", lo[2]);
    assert!(
        hi[0] - lo[0] < 175.0,
        "the figure is {:.0} wide - inflated, which is the transposed-rotation signature",
        hi[0] - lo[0]
    );
}
