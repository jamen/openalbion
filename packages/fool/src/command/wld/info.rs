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

    println!("  regions              {}", wld.regions.len());
    for region in &wld.regions {
        println!(
            "  region {n:3} {name}  def={def}  contains={contains}  sees={sees}{world_map}",
            n = region.region_number,
            name = region.region_name,
            def = region.region_def,
            contains = region.contains_maps.len(),
            sees = region.sees_maps.len(),
            world_map = if region.appears_on_world_map {
                "  on-world-map"
            } else {
                ""
            },
        );
    }

    Ok(())
}
