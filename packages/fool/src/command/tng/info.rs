use anyhow::Context;
use clap::Parser;
use fable_data::tng::Tng;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct TngInfoArgs {
    /// Input .tng file to inspect
    input: PathBuf,
}

pub fn handler(args: TngInfoArgs) -> anyhow::Result<()> {
    let bytes = fs::read(&args.input).context("Could not read tng file")?;
    let text = String::from_utf8_lossy(&bytes);

    let tng = Tng::parse(&text).context("Could not parse tng file")?;

    println!("{}", args.input.display());
    println!("  sections             {}", tng.sections.len());
    for (si, section) in tng.sections.iter().enumerate() {
        println!("  [section {}] {}", si, section.name);
        println!("    things             {}", section.things.len());
        for (ti, thing) in section.things.iter().enumerate() {
            println!(
                "    [{ti}] type={} def={} pos=({:.1},{:.1},{:.1})",
                thing.thing_type,
                thing.definition_type,
                thing.position[0],
                thing.position[1],
                thing.position[2],
            );
            if let Some(name) = &thing.script_name {
                println!("      script={name}");
            }
        }
    }

    Ok(())
}
