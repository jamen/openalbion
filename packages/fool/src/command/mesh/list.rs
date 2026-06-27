use anyhow::Context;
use clap::Parser;
use fable_data::big::{BigReader, ExtraMetadata};
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct MeshListArgs {
    /// A .big archive (e.g. data/graphics/graphics.big)
    input: PathBuf,
}

pub fn handler(args: MeshListArgs) -> anyhow::Result<()> {
    let file = File::open(&args.input).context("Could not open big file")?;
    let reader = BigReader::new(file).context("Could not read big file")?;

    let mut count = 0;
    for bank in reader.bank_iter() {
        let bank_name = &bank.metadata().name;
        for asset in bank.asset_iter() {
            if matches!(asset.extras, Some(ExtraMetadata::Mesh(_))) {
                println!("{}\t{}", bank_name, asset.symbol_name);
                count += 1;
            }
        }
    }

    eprintln!("{count} mesh assets");
    Ok(())
}
