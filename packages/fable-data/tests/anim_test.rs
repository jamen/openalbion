//! The animation parser against the shipped data — the evidence behind §3.17.
//!
//! These are the numbers the format claim rests on: every animation asset in `graphics.big`
//! decompresses, walks its chunk tree, and yields sequences whose frame counts, sample rates
//! and track sizes reconcile with each other. They need a Fable install, so they skip when
//! there is not one.

use fable_data::anim::{Animation, TrackMode};
use fable_data::big::{BigReader, ExtraMetadata};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

fn archive() -> Option<BigReader<File>> {
    let path = PathBuf::from("/home/jamen/Fable/data/graphics/graphics.big");
    path.exists()
        .then(|| BigReader::new(File::open(&path).unwrap()).unwrap())
}

/// Every animation asset parses. A format read wrongly would overrun a chunk long before it
/// got through 3,435 files, so this is the load-bearing test.
#[test]
fn every_animation_parses() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let ids: Vec<(u32, String)> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Animation(_))))
        .map(|a| (a.id, a.symbol_name.to_string()))
        .collect();

    assert_eq!(ids.len(), 3435, "animation assets in graphics.big");

    let mut parsed = 0usize;
    let mut sequences = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut rates: BTreeMap<u32, usize> = BTreeMap::new();
    let mut modes: BTreeMap<&str, usize> = BTreeMap::new();

    for (id, name) in &ids {
        let asset = big
            .bank_iter()
            .flat_map(|b| b.asset_iter())
            .find(|a| a.id == *id)
            .unwrap()
            .clone();
        let data = match big.read_asset_from_metadata(&asset) {
            Ok(d) => d,
            Err(e) => {
                failures.push(format!("{name}: read {e}"));
                continue;
            }
        };
        match Animation::decode(&data) {
            Ok(anim) => {
                parsed += 1;
                for object in &anim.objects {
                    for s in &object.sequences {
                        sequences += 1;
                        *rates.entry(s.samples_per_second as u32).or_default() += 1;
                        for (label, mode) in [
                            ("rotation", s.rotation_mode),
                            ("position", s.position_mode),
                        ] {
                            let key = match (label, mode) {
                                ("rotation", TrackMode::Identity) => "rot identity",
                                ("rotation", TrackMode::Constant) => "rot constant",
                                ("rotation", TrackMode::Normal) => "rot normal",
                                ("rotation", TrackMode::Paletted) => "rot paletted",
                                (_, TrackMode::Identity) => "pos identity",
                                (_, TrackMode::Constant) => "pos constant",
                                (_, TrackMode::Normal) => "pos normal",
                                (_, TrackMode::Paletted) => "pos paletted",
                            };
                            *modes.entry(key).or_default() += 1;
                        }
                    }
                }
            }
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }

    eprintln!("parsed {parsed} of {}, {sequences} sequences", ids.len());
    eprintln!("  sample rates: {rates:?}");
    eprintln!("  track modes:  {modes:?}");
    for f in failures.iter().take(10) {
        eprintln!("  FAIL {f}");
    }

    assert!(failures.is_empty(), "{} animations failed to parse", failures.len());
    assert_eq!(sequences, 210_743, "bone sequences across the archive");
}

/// Internal consistency, which is what catches a field read at the wrong offset even when the
/// chunk sizes happen to work out.
#[test]
fn sequences_are_self_consistent() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let ids: Vec<u32> = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .filter(|a| matches!(&a.extras, Some(ExtraMetadata::Animation(_))))
        .map(|a| a.id)
        .collect();

    let mut checked = 0usize;
    let (mut quats, mut unit_quats, mut worst_quat) = (0usize, 0usize, 0.0f32);
    for id in ids {
        let asset = big
            .bank_iter()
            .flat_map(|b| b.asset_iter())
            .find(|a| a.id == id)
            .unwrap()
            .clone();
        let Ok(data) = big.read_asset_from_metadata(&asset) else {
            continue;
        };
        let Ok(anim) = Animation::decode(&data) else { continue };

        for object in &anim.objects {
            for s in &object.sequences {
                checked += 1;

                // A frame rate the game could plausibly have authored.
                assert!(
                    [5.0, 8.0, 10.0, 15.0, 20.0, 24.0, 25.0, 30.0].contains(&s.samples_per_second),
                    "{}: odd sample rate {}",
                    s.bone_name,
                    s.samples_per_second
                );

                // A paletted track needs one index per frame, and every index must be in range.
                if s.rotation_mode == TrackMode::Paletted {
                    assert_eq!(
                        s.rotation_palette.len() as u32, s.frame_count,
                        "{}: rotation palette length must equal the frame count",
                        s.bone_name
                    );
                    assert!(
                        s.rotation_palette.iter().all(|&i| (i as usize) < s.rotations.len()),
                        "{}: rotation palette index out of range",
                        s.bone_name
                    );
                }
                if s.position_mode == TrackMode::Paletted {
                    assert_eq!(
                        s.position_palette.len() as u32, s.frame_count,
                        "{}: position palette length must equal the frame count",
                        s.bone_name
                    );
                    assert!(
                        s.position_palette.iter().all(|&i| (i as usize) < s.positions.len()),
                        "{}: position palette index out of range",
                        s.bone_name
                    );
                }
                // A per-frame track needs one value per frame.
                if s.rotation_mode == TrackMode::Normal {
                    assert_eq!(
                        s.rotations.len() as u32, s.frame_count,
                        "{}: normal rotation track length must equal the frame count",
                        s.bone_name
                    );
                }
                if s.position_mode == TrackMode::Normal {
                    assert_eq!(
                        s.positions.len() as u32, s.frame_count,
                        "{}: normal position track length must equal the frame count",
                        s.bone_name
                    );
                }
                // A constant track stores exactly one value.
                if s.rotation_mode == TrackMode::Constant {
                    assert_eq!(s.rotations.len(), 1, "{}: constant rotation", s.bone_name);
                }

                // Quaternions must be unit length. This is the strongest single check that
                // the rotation track is read at the right offset: a wrong offset yields
                // arbitrary floats, and arbitrary floats are not unit quaternions. A handful
                // of authored values are a few percent off — never renormalised by the
                // exporter — so this measures the distribution rather than demanding
                // perfection, and the *shape* of that distribution is the evidence.
                for q in &s.rotations {
                    let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
                    quats += 1;
                    if (len - 1.0).abs() < 0.01 {
                        unit_quats += 1;
                    }
                    if (len - 1.0).abs() > worst_quat {
                        worst_quat = (len - 1.0).abs();
                    }
                }
            }
        }
    }
    let unit_fraction = unit_quats as f64 / quats as f64;
    eprintln!("checked {checked} sequences, {quats} quaternions");
    eprintln!("  unit length within 1%: {unit_quats} ({:.4}%), worst deviation {worst_quat:.4}", unit_fraction * 100.0);
    assert!(checked > 200_000);
    assert!(
        unit_fraction > 0.999,
        "only {:.3}% of quaternions are unit length - the rotation track is misaligned",
        unit_fraction * 100.0
    );
}

/// A known clip, pinned end to end. `ANIM_BIPED_GENERIC_MAN_HOMELIFE_GET_OUT_BED` is a
/// villager animation: looping, 10.5 s at 30 fps, on a Biped skeleton.
#[test]
fn a_known_clip_reads_back_exactly() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let asset = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .find(|a| a.symbol_name == "ANIM_BIPED_GENERIC_MAN_HOMELIFE_GET_OUT_BED")
        .cloned()
        .expect("the clip is in the archive");
    let data = big.read_asset_from_metadata(&asset).unwrap();
    let anim = Animation::decode(&data).unwrap();

    assert!(anim.looping);
    assert!((anim.duration - 10.5).abs() < 1e-3, "duration {}", anim.duration);
    assert_eq!(anim.objects.len(), 1);
    assert_eq!(anim.objects[0].name, "male_villager_complete");
    assert_eq!(anim.samples_per_second(), 30.0);

    // duration x rate is the frame count, exactly.
    assert_eq!(anim.frame_count(), 315);
    assert_eq!((anim.duration * 30.0).round() as u32, anim.frame_count());

    let names: Vec<&str> = anim.objects[0]
        .sequences
        .iter()
        .map(|s| s.bone_name.as_str())
        .collect();
    assert_eq!(&names[..5], &["Scene Root", "Movement", "Sub_movement_dummy", "Bip01", "Bip01 Pelvis"]);
    assert!(names.contains(&"Bip01 Spine"));
    assert_eq!(anim.objects[0].sequences[0].parent_index, -1, "the root has no parent");
    assert_eq!(anim.objects[0].sequences[1].parent_index, 0);
}

/// A door: one bone, one frame, a constant 90-degree rotation about Z. This is the smallest
/// end-to-end proof that the quaternion track decodes to a *meaningful* value rather than to
/// plausible-looking floats.
#[test]
fn a_door_rotates_ninety_degrees() {
    let Some(mut big) = archive() else {
        eprintln!("skipping: no Fable install");
        return;
    };
    let asset = big
        .bank_iter()
        .flat_map(|b| b.asset_iter())
        .find(|a| a.symbol_name == "ANIM_CRYPT_PASSAGE_DOOR_DEFAULT_01")
        .cloned()
        .expect("the clip is in the archive");
    let data = big.read_asset_from_metadata(&asset).unwrap();
    let anim = Animation::decode(&data).unwrap();

    let object = &anim.objects[0];
    assert_eq!(object.name, "Crypt_door");
    let door = object
        .sequences
        .iter()
        .find(|s| s.bone_name == "Box01")
        .expect("Box01");

    assert_eq!(door.rotation_mode, TrackMode::Constant);
    let q = door.rotation_at(0).expect("a constant rotation");
    let root2 = std::f32::consts::FRAC_1_SQRT_2;
    assert!((q[0]).abs() < 1e-4 && (q[1]).abs() < 1e-4, "{q:?} should rotate about Z only");
    assert!(
        (q[2].abs() - root2).abs() < 1e-3 && (q[3].abs() - root2).abs() < 1e-3,
        "{q:?} should be a quarter turn (components +/-1/sqrt(2))"
    );
}
