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
            let placement = match thing.placement() {
                Some(p) => {
                    let orientation = match p.orientation {
                        Some(rh) => format!(
                            " fwd=({:.2},{:.2},{:.2}) up=({:.2},{:.2},{:.2})",
                            rh.forward[0],
                            rh.forward[1],
                            rh.forward[2],
                            rh.up[0],
                            rh.up[1],
                            rh.up[2],
                        ),
                        None => String::new(),
                    };
                    format!(
                        " pos=({:.1},{:.1},{:.1}){orientation}",
                        p.position[0], p.position[1], p.position[2],
                    )
                }
                None => String::new(),
            };
            let base = thing.base();
            println!(
                "    [{ti}] type={:?} def={}{placement}{scale}",
                thing.type_name(),
                base.definition_type,
                scale = match thing.physical().and_then(|p| p.object_scale) {
                    Some(scale) => format!(" scale={scale}"),
                    None => String::new(),
                },
            );
            if base.script_name != "NULL" && !base.script_name.is_empty() {
                println!("      script={}", base.script_name);
            }
            let classes: Vec<&str> = base.components.iter().map(|c| c.class()).collect();
            if !classes.is_empty() {
                println!("      components={}", classes.join(" "));
            }
        }
    }

    Ok(())
}
