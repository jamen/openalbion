use anyhow::anyhow;
use clap::Parser;
use fable_data::def::binary::{
    def_binary::{DefBinary, DefBody},
    names::Names,
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct DefRoundtripArgs {
    /// Path to the compiled def binary (e.g. data/CompiledDefs/game.bin)
    game_bin: PathBuf,

    /// Path to the names table (e.g. data/CompiledDefs/names.bin)
    names_bin: PathBuf,

    /// Only round-trip this def type (e.g. ENGINE). Default: every implemented type.
    #[arg(long = "type")]
    def_type: Option<String>,
}

#[derive(Default)]
struct Stats {
    pass: usize,
    fail: usize,
    unimplemented: usize,
}

pub fn handler(args: DefRoundtripArgs) -> anyhow::Result<()> {
    let names = Names::load(&args.names_bin).map_err(|e| anyhow!("load names.bin: {e:?}"))?;
    let def_binary =
        DefBinary::load_with_names(&args.game_bin, &names).map_err(|e| anyhow!("load game.bin: {e:?}"))?;

    let mut stats: BTreeMap<String, Stats> = BTreeMap::new();
    let mut first_fail: Option<String> = None;

    for entry in def_binary.entries(&names) {
        let name = entry.def_name.unwrap_or("<noname>").to_string();
        if let Some(want) = &args.def_type
            && &name != want
        {
            continue;
        }
        let record = entry.record;
        let stat = stats.entry(name.clone()).or_default();

        if matches!(record.body, DefBody::Unknown { .. }) {
            stat.unimplemented += 1;
            continue;
        }

        // Serialize into a buffer the exact size of the original record and require byte-identical
        // output — this proves the typed parser's field order/kinds/values are correct.
        let mut buf = vec![0u8; record.raw_bytes.len()];
        let mut cursor = &mut buf[..];
        match record.serialize(&mut cursor) {
            Ok(()) => {
                let unwritten = cursor.len();
                if unwritten == 0 && buf == record.raw_bytes {
                    stat.pass += 1;
                } else {
                    stat.fail += 1;
                    if first_fail.is_none() {
                        let written = record.raw_bytes.len() - unwritten;
                        let diff = buf
                            .iter()
                            .zip(&record.raw_bytes)
                            .position(|(a, b)| a != b);
                        first_fail = Some(format!(
                            "{name}: wrote {written}/{} bytes, first byte diff at {diff:?}",
                            record.raw_bytes.len(),
                        ));
                    }
                }
            }
            Err(error) => {
                stat.fail += 1;
                if first_fail.is_none() {
                    first_fail = Some(format!("{name}: serialize error: {error:?}"));
                }
            }
        }
    }

    println!("{:<52} {:>6} {:>6} {:>6}", "DEF TYPE", "PASS", "FAIL", "SKIP");
    let (mut total_pass, mut total_fail, mut total_skip) = (0, 0, 0);
    for (name, s) in &stats {
        total_pass += s.pass;
        total_fail += s.fail;
        total_skip += s.unimplemented;
        if s.pass + s.fail > 0 {
            println!(
                "{name:<52} {:>6} {:>6} {:>6}",
                s.pass, s.fail, s.unimplemented
            );
        }
    }
    println!();
    println!("{total_pass} passed, {total_fail} failed, {total_skip} skipped (no typed parser)");
    if let Some(detail) = first_fail {
        println!("first failure: {detail}");
    }

    Ok(())
}
