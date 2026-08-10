//! Validates the `.tng` / `.wld` parsers against real game data.
//!
//! Reads every `.tng` out of `FinalAlbion.wad` and parses it. Two independent
//! signals, because "it parsed" is weak on its own:
//!
//! 1. **Nothing errors.** Every file in the wad parses.
//! 2. **Nothing is dropped.** The number of things the parser produces equals
//!    the number of `NewThing` lines in the file, counted by a scan that shares
//!    no code with the parser. That is what catches a block silently swallowing
//!    its siblings — the failure mode a "no errors" test cannot see.
//!
//! Skips gracefully if the game data isn't present. Set `FABLE_DATA` to override
//! the data dir (defaults to `~/Fable/data`).

use fable_data::tng::{CameraPoint, Tng};
use fable_data::wad::WadReader;
use fable_data::wld::Wld;
use std::collections::BTreeMap;
use std::{fs::File, io::BufReader, path::PathBuf};

fn fable_data_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("FABLE_DATA") {
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").ok()?;
    let dir = PathBuf::from(home).join("Fable/data");
    dir.is_dir().then_some(dir)
}

/// Every `.tng` in the wad, as `(file name, text)`.
fn final_albion_tngs(data_dir: &std::path::Path) -> Option<Vec<(String, String)>> {
    let wad_path = data_dir.join("Levels/FinalAlbion.wad");
    if !wad_path.is_file() {
        eprintln!("skipping: {wad_path:?} not found");
        return None;
    }

    let file = BufReader::new(File::open(&wad_path).expect("open wad"));
    let mut reader = WadReader::new(file).expect("read wad");

    let assets: Vec<_> = reader
        .asset_iter()
        .filter(|a| a.path.to_lowercase().ends_with(".tng"))
        .cloned()
        .collect();
    assert!(!assets.is_empty(), "no .tng files found in wad");

    Some(
        assets
            .iter()
            .map(|asset| {
                let name = asset
                    .path
                    .rsplit('\\')
                    .next()
                    .unwrap_or(&asset.path)
                    .to_string();
                let bytes = reader.read_content(asset).expect("read tng");
                (name, String::from_utf8_lossy(&bytes).into_owned())
            })
            .collect(),
    )
}

/// `NewThing` lines, counted without the parser. The `.tng` writer emits one per
/// thing at the start of a line (`fablelib/thing_manager.cpp:5638`), so this is
/// an honest second opinion on how many there should be.
fn count_new_thing_lines(text: &str) -> usize {
    text.lines()
        .filter(|line| line.trim_end_matches(['\r', ' ']).starts_with("NewThing "))
        .count()
}

#[test]
fn parse_all_final_albion_tngs() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let Some(files) = final_albion_tngs(&data_dir) else {
        return;
    };

    let mut failures = Vec::new();
    let mut miscounts = Vec::new();
    let mut total_things = 0usize;
    let mut thing_types: BTreeMap<String, usize> = BTreeMap::new();

    for (name, text) in &files {
        match Tng::parse(text) {
            Err(error) => failures.push(format!("{name}: {error}")),
            Ok(tng) => {
                let things: Vec<_> = tng.things().collect();
                let expected = count_new_thing_lines(text);
                if things.len() != expected {
                    miscounts.push(format!(
                        "{name}: parsed {} things, file has {expected} NewThing lines",
                        things.len()
                    ));
                }
                total_things += things.len();
                for thing in things {
                    *thing_types
                        .entry(thing.type_name().to_string())
                        .or_default() += 1;
                }
            }
        }
    }

    eprintln!(
        "parsed {} .tng files, {total_things} things; types: {thing_types:?}",
        files.len()
    );

    assert!(
        failures.is_empty(),
        "parse failures:\n  {}",
        failures.join("\n  ")
    );
    assert!(
        miscounts.is_empty(),
        "things dropped or duplicated:\n  {}",
        miscounts.join("\n  ")
    );

    // `Holy Site` is the only type name with a space in it, and it is the one
    // the previous parser could not read (`CThing::GetTypeName()` is a display
    // name, not an identifier). Its presence proves the whole corpus was reached.
    assert!(
        thing_types.contains_key("Holy Site"),
        "no `Holy Site` things parsed — types seen: {thing_types:?}"
    );
}

/// The placement fields every renderable thing needs, checked over the corpus.
///
/// `CTCPhysicsStandard`'s right-handed set is written already orthonormal, which
/// is why the object matrix can be built from it directly with no Gram–Schmidt
/// (`CalcObjectMatrix`, `fableengine/engine_primitive_manager_mesh_base.cpp:48`).
/// If that ever stops holding, this test says so before the renderer does.
#[test]
fn physics_placement_is_orthonormal() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let Some(files) = final_albion_tngs(&data_dir) else {
        return;
    };

    let mut checked = 0usize;
    let mut worst_dot: f32 = 0.0;
    let mut worst_len: f32 = 0.0;

    for (name, text) in &files {
        let tng = Tng::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        for thing in tng.things() {
            let Some(placement) = thing.placement() else {
                continue;
            };
            let Some(rhset) = placement.orientation else {
                continue; // CTCPhysicsLight writes a position and no orientation
            };
            let (f, u) = (rhset.forward, rhset.up);
            let length = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            if length == 0.0 {
                continue; // written with no orientation at all
            }
            checked += 1;
            worst_dot = worst_dot.max((f[0] * u[0] + f[1] * u[1] + f[2] * u[2]).abs());
            worst_len = worst_len.max((length - 1.0).abs());
        }
    }

    eprintln!(
        "checked {checked} right-handed sets: max |F.U| = {worst_dot}, max ||F|-1| = {worst_len}"
    );
    assert!(checked > 10_000, "only {checked} things had placement");
    assert!(
        worst_dot < 1e-4,
        "forward and up are not perpendicular: {worst_dot}"
    );
    assert!(worst_len < 1e-3, "forward is not unit length: {worst_len}");
}

/// Every component class in the game is modelled, and every one is reachable.
///
/// The first half is already proved by `parse_all_final_albion_tngs`: an
/// unmodelled class or field is a hard error, so a clean parse *is* full
/// coverage. This adds the other direction — that no modelled class is dead
/// weight — and prints the inventory, which is the thing to eyeball when the
/// format is extended.
#[test]
fn every_component_class_is_used() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let Some(files) = final_albion_tngs(&data_dir) else {
        return;
    };

    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut total = 0usize;
    for (name, text) in &files {
        let tng = Tng::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        for thing in tng.things() {
            for component in thing.components().iter() {
                *classes.entry(component.class()).or_default() += 1;
                total += 1;
            }
        }
    }

    eprintln!("{total} components across {} classes", classes.len());
    for (class, count) in &classes {
        eprintln!("  {class:<44} {count}");
    }
    assert_eq!(classes.len(), 59, "expected all 59 classes to appear");
}

/// Storing components in named slots loses nothing.
///
/// `Components` is a struct of slots rather than a list, which is only sound
/// because a thing never repeats a class and the order the game writes them in
/// is a total order over the classes. This checks the second half directly:
/// `Components::iter()` must reproduce each thing's blocks in file order, for
/// every thing in the game. Scanning the text for `StartCTC…` shares no code
/// with the parser.
#[test]
fn slots_reproduce_the_file_order() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let Some(files) = final_albion_tngs(&data_dir) else {
        return;
    };

    let mut checked = 0usize;
    for (name, text) in &files {
        let tng = Tng::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));

        // Every `StartCTC…;` in the file, grouped per thing, in written order.
        let mut written: Vec<Vec<&str>> = Vec::new();
        for line in text.lines().map(|l| l.trim().trim_end_matches(';')) {
            if line.starts_with("NewThing ") {
                written.push(Vec::new());
            } else if let Some(class) = line.strip_prefix("Start")
                && class.starts_with("CTC")
                && let Some(current) = written.last_mut()
            {
                current.push(class);
            }
        }

        let parsed: Vec<Vec<&str>> = tng
            .things()
            .map(|t| t.components().iter().map(|c| c.class()).collect())
            .collect();

        assert_eq!(parsed.len(), written.len(), "{name}: thing count");
        for (i, (parsed, written)) in parsed.iter().zip(&written).enumerate() {
            assert_eq!(parsed, written, "{name}: thing {i} component order");
            checked += 1;
        }
    }
    eprintln!("checked component order on {checked} things");
    assert!(checked > 20_000, "only {checked} things checked");
}

/// The array lengths the format writes alongside its arrays are redundant.
///
/// `NumKeyCameras`, `NumShapes` and `Shape[i].size()` are read into
/// `Redundant` and thrown away, which is only safe while they agree with the
/// arrays. Re-count them here from the raw text, so the model's one lossy
/// simplification is the one thing this test watches.
#[test]
fn counts_match_their_arrays() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let Some(files) = final_albion_tngs(&data_dir) else {
        return;
    };

    /// `NumShapes 3;` → `Some(3)`.
    fn written_count<'a>(text: &'a str, field: &str) -> impl Iterator<Item = usize> + 'a {
        let field = format!("{field} ");
        text.lines()
            .map(|l| l.trim().trim_end_matches(';').to_string())
            .filter_map(move |l| l.strip_prefix(&field)?.trim().parse().ok())
            .collect::<Vec<_>>()
            .into_iter()
    }

    let mut checked = 0usize;
    for (name, text) in &files {
        let tng = Tng::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));

        let mut splines = Vec::new();
        let mut managers = Vec::new();
        for thing in tng.things() {
            let components = thing.components();
            if let Some(CameraPoint::ScriptedSpline(s)) = &components.camera_point {
                splines.push(s.key_cameras.len());
            }
            if let Some(s) = &components.shape_manager {
                managers.push(s);
            }
        }

        let declared: Vec<usize> = written_count(text, "NumKeyCameras").collect();
        assert_eq!(declared, splines, "{name}: NumKeyCameras vs KeyCameras[]");
        checked += declared.len();

        let declared: Vec<usize> = written_count(text, "NumShapes").collect();
        let actual: Vec<usize> = managers.iter().map(|m| m.shapes.len()).collect();
        assert_eq!(declared, actual, "{name}: NumShapes vs Shape[]");
        checked += declared.len();

        // `Shape[i].size()` is written once per shape, in the order the shape
        // managers appear, so flattening both sides compares like for like.
        let declared: Vec<usize> = text
            .lines()
            .map(|l| l.trim().trim_end_matches(';'))
            .filter_map(|l| l.split(".size() ").nth(1)?.trim().parse().ok())
            .collect();
        let actual: Vec<usize> = managers
            .iter()
            .flat_map(|m| m.shapes.iter().map(|s| s.positions.len()))
            .collect();
        assert_eq!(declared, actual, "{name}: Shape[i].size() vs pos[]");
        checked += declared.len();
    }
    eprintln!("checked {checked} written array counts against their arrays");
    assert!(checked > 1_000, "only {checked} counts checked");
}

#[test]
fn parse_final_albion_wld() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let path = data_dir.join("Levels/FinalAlbion.wld");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("skipping: {path:?} not found");
        return;
    };

    let text = String::from_utf8_lossy(&bytes);
    let wld = Wld::parse(&text).expect("parse FinalAlbion.wld");

    eprintln!("{} maps, {} regions", wld.maps.len(), wld.regions.len());

    // Counted from the file by the same independent-scan argument as above.
    let expected_maps = text.lines().filter(|l| l.starts_with("NewMap ")).count();
    let expected_regions = text.lines().filter(|l| l.starts_with("NewRegion ")).count();
    assert_eq!(wld.maps.len(), expected_maps);
    assert_eq!(wld.regions.len(), expected_regions);

    assert!(wld.maps.iter().all(|m| !m.level_name.is_empty()));
    assert!(wld.regions.iter().all(|r| !r.region_name.is_empty()));

    // Every `.lev` a region names should be a map the world places, which only
    // holds if the repeated `ContainsMap` / `SeesMap` fields all survived.
    let placed: std::collections::HashSet<String> = wld
        .maps
        .iter()
        .map(|m| m.level_name.to_lowercase())
        .collect();
    let mut unplaced = Vec::new();
    for region in &wld.regions {
        for level in region.contains_maps.iter().chain(&region.sees_maps) {
            if !placed.contains(&level.to_lowercase()) {
                unplaced.push(format!("{}: {level}", region.region_name));
            }
        }
    }
    assert!(
        unplaced.is_empty(),
        "regions name levels the world does not place:\n  {}",
        unplaced.join("\n  ")
    );
}

/// [`Wld::maps_for_region_of`] against the shipped world — the region a level needs to load
/// alongside itself, not just the region parser it is built on.
#[test]
fn resolves_lookoutpoint_s_region_from_the_real_world() {
    let Some(data_dir) = fable_data_dir() else {
        eprintln!("skipping: no Fable data dir (set FABLE_DATA or symlink ~/Fable)");
        return;
    };
    let path = data_dir.join("Levels/FinalAlbion.wld");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("skipping: {path:?} not found");
        return;
    };

    let text = String::from_utf8_lossy(&bytes);
    let wld = Wld::parse(&text).expect("parse FinalAlbion.wld");

    let maps = wld.maps_for_region_of("LookoutPoint");

    // Region 1 "LookoutPoint": 3 `ContainsMap` (BowerstoneBridge, LookoutPoint,
    // GuildExterior), 11 `SeesMap` fillers — read straight off the shipped `.wld`.
    let populated: Vec<&str> = maps
        .iter()
        .filter(|m| m.populated)
        .map(|m| m.level_name.as_str())
        .collect();
    let fillers = maps.iter().filter(|m| !m.populated).count();
    assert_eq!(populated.len(), 3, "populated maps: {populated:?}");
    assert_eq!(fillers, 11);
    assert!(populated.contains(&"BowerstoneBridge"));
    assert!(populated.contains(&"LookoutPoint"));
    assert!(populated.contains(&"GuildExterior"));

    // The requested level keeps its own `.wld` origin, not (0, 0) — the whole point of
    // resolving the region is to place every map correctly relative to the others.
    let lookout = maps.iter().find(|m| m.level_name == "LookoutPoint").unwrap();
    assert_eq!(lookout.origin, (3232, 3488));

    // A filler is present and marked unpopulated, never confused with a `ContainsMap`.
    let filler = maps
        .iter()
        .find(|m| m.level_name == "LookoutPoint_Filler_01")
        .expect("LookoutPoint_Filler_01 should be a SeesMap filler");
    assert!(!filler.populated);

    // Asking from any populated member of the region returns the same set (§ the region is
    // a symmetric relation, not "whichever level you happened to ask about").
    let mut from_lookout = maps.clone();
    let mut from_bridge = wld.maps_for_region_of("BowerstoneBridge");
    from_lookout.sort_by(|a, b| a.level_name.cmp(&b.level_name));
    from_bridge.sort_by(|a, b| a.level_name.cmp(&b.level_name));
    assert_eq!(from_lookout, from_bridge);
}
