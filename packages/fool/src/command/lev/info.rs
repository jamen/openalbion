use anyhow::Context;
use clap::Parser;
use fable_data::lev::Lev;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct LevInfoArgs {
    /// Input .lev file to inspect
    input: PathBuf,
}

pub fn handler(args: LevInfoArgs) -> anyhow::Result<()> {
    let bytes = fs::read(&args.input).context("Could not read lev file")?;

    let lev = Lev::from_bytes(&bytes).context("Could not parse lev file")?;
    let header = &lev.header;

    println!("{}", args.input.display());
    println!("  version              {}", header.version);
    println!("  map_version          {}", header.map_version);
    println!("  dimensions           {} x {}", header.width, header.height);
    println!("  unique_id_count      {}", header.unique_id_count);
    println!("  navigation_offset    {}", header.navigation_offset);
    println!("  checksum             {:#010x}", header.checksum);
    println!("  sound_themes         {:?}", header.sound_themes);
    println!("  heightmap_cells      {}", lev.heightmap_cells.len());
    println!("  soundmap_cells       {}", lev.soundmap_cells.len());

    if !lev.heightmap_cells.is_empty() {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        let mut walkable = 0usize;
        for cell in &lev.heightmap_cells {
            min = min.min(cell.height);
            max = max.max(cell.height);
            if cell.walkable {
                walkable += 1;
            }
        }
        println!("  height range         {min:.3} .. {max:.3}");
        println!(
            "  walkable cells       {} / {}",
            walkable,
            lev.heightmap_cells.len()
        );
    }

    let nav = &lev.navigation;
    println!("  navigation:");
    let names: Vec<&str> = nav.section_names.iter().map(|n| n.name.as_str()).collect();
    println!("    sections           {:?}", names);
    for (idx, section) in nav.sections.iter().enumerate() {
        use fable_data::lev::LevNavigationNode as N;
        let (mut interior, mut navigable, mut blocked, mut switchable) = (0, 0, 0, 0);
        for node in &section.nodes {
            match node {
                N::Interior { .. } => interior += 1,
                N::Navigable { .. } => navigable += 1,
                N::Blocked { .. } => blocked += 1,
                N::Switchable { .. } => switchable += 1,
            }
        }
        println!(
            "    section[{idx}]        {}x{}, {} action point(s), {} layer(s)",
            section.map_width,
            section.map_height,
            section.action_points.len(),
            section.layer_count,
        );
        println!(
            "      nodes            {} total (interior={interior}, navigable={navigable}, blocked={blocked}, switchable={switchable})",
            section.nodes.len(),
        );
    }

    Ok(())
}
