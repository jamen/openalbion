// Validation (ignored by default): load retail game.bin and check the typed
// environment defs parse.  cargo test -p fable-data --test explore_env -- --ignored --nocapture
use fable_data::def::binary::def_binary::{DefBinary, DefBody};
use fable_data::def::binary::names::Names;
use std::collections::BTreeMap;
use std::path::Path;

#[test]
#[ignore]
fn explore() {
    let fable = Path::new("/home/jamen/Fable/data");
    let names = Names::load(&fable.join("CompiledDefs/names.bin")).expect("names");
    let game =
        DefBinary::load_with_names(&fable.join("CompiledDefs/game.bin"), &names).expect("game");

    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    let mut envs = 0;
    let mut sets = 0;
    for entry in game.entries(&names) {
        match &entry.record.body {
            DefBody::Environment(_) => { envs += 1; *kinds.entry("Environment").or_default() += 1; }
            DefBody::EnvironmentThemeDaySet(s) => {
                sets += 1;
                *kinds.entry("EnvironmentThemeDaySet").or_default() += 1;
                if sets <= 2 {
                    println!("THEME_DAY {:?}: {} keyframes, sun_tilt={}, colour_lookup_column={}",
                        entry.def_name, s.time.len(), s.sun_tilt, s.colour_lookup_column);
                    for (i, kf) in s.time.iter().enumerate().take(3) {
                        println!("    kf[{i}] tod={} sky0={} sky1={} blend={} moonlit={}",
                            kf.time_of_day, kf.sky_texture_0, kf.sky_texture_1,
                            kf.sky_texture_1_blend, kf.moon_lit);
                    }
                }
            }
            DefBody::Unknown { .. } => { *kinds.entry("Unknown").or_default() += 1; }
            _ => { *kinds.entry("other-typed").or_default() += 1; }
        }
    }
    println!("environments={envs} theme_day_sets={sets}");
    println!("kind counts: {kinds:?}");
}
