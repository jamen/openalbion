use anyhow::anyhow;
use clap::Parser;
use fable_data::def::binary::{
    def_binary::{DefBinary, DefBody},
    names::Names,
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct DefListArgs {
    /// Path to the compiled def binary (e.g. data/CompiledDefs/game.bin)
    game_bin: PathBuf,

    /// Path to the names table (e.g. data/CompiledDefs/names.bin)
    names_bin: PathBuf,

    /// Only show def types without a typed parser yet (the worklist).
    #[arg(long)]
    unknown_only: bool,
}

pub fn handler(args: DefListArgs) -> anyhow::Result<()> {
    let names = Names::load(&args.names_bin).map_err(|e| anyhow!("load names.bin: {e:?}"))?;
    let def_binary =
        DefBinary::load_with_names(&args.game_bin, &names).map_err(|e| anyhow!("load game.bin: {e:?}"))?;

    // def type name -> (record count, implemented?)
    let mut by_type: BTreeMap<String, (usize, bool)> = BTreeMap::new();
    for entry in def_binary.entries(&names) {
        let name = entry.def_name.unwrap_or("<noname>").to_string();
        let implemented = !matches!(entry.record.body, DefBody::Unknown { .. });
        let slot = by_type.entry(name).or_insert((0, implemented));
        slot.0 += 1;
        slot.1 = implemented;
    }

    let mut rows: Vec<(&String, &(usize, bool))> = by_type
        .iter()
        .filter(|(_, (_, implemented))| !args.unknown_only || !implemented)
        .collect();
    // Most-common types first (best validation fixtures); ties broken by name.
    rows.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then(a.0.cmp(b.0)));

    println!("{:<52} {:>7}  STATUS", "DEF TYPE", "COUNT");
    for (name, (count, implemented)) in &rows {
        let status = if *implemented { "impl" } else { "unknown" };
        println!("{name:<52} {count:>7}  {status}");
    }

    let total_types = by_type.len();
    let impl_types = by_type.values().filter(|(_, i)| *i).count();
    let total_records: usize = by_type.values().map(|(c, _)| c).sum();
    println!();
    println!(
        "{total_types} def types ({impl_types} implemented, {} unknown), {total_records} total records",
        total_types - impl_types,
    );

    Ok(())
}
