use anyhow::Context;
use clap::Parser;
use fable_data::wld::Wld;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct WldInfoArgs {
    /// Input .wld file to inspect
    input: PathBuf,
}

pub fn handler(args: WldInfoArgs) -> anyhow::Result<()> {
    let bytes = fs::read(&args.input).context("Could not read wld file")?;
    let text = String::from_utf8_lossy(&bytes);

    let wld = Wld::parse(&text).context("Could not parse wld file")?;

    println!("{}", args.input.display());
    println!("  maps                 {}", wld.maps.len());
    for map in &wld.maps {
        println!(
            "  map {n:3} ({x:4},{y:4}) {lev}  script={script}  uid={uid}  sea={sea}  prox={prox}",
            n = map.map_number,
            x = map.map_x,
            y = map.map_y,
            lev = map.level_name,
            script = map.level_script_name,
            uid = map.map_uid,
            sea = map.is_sea,
            prox = map.loaded_on_proximity,
        );
    }

    Ok(())
}
