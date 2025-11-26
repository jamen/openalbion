use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::BigFile;
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Parser, Debug, Clone)]
pub struct BigDumpArgs {
    /// Input big file to be extracted
    input: PathBuf,

    /// Output directory to extract into
    output: Option<PathBuf>,
}

pub fn handler(args: BigDumpArgs) -> anyhow::Result<()> {
    let input_path = args.input;

    // Get the output path, defaulting to one based on the input file path if none is provided
    let output_path = args
        .output
        .or_else(|| {
            input_path
                .parent()
                .and_then(|parent| input_path.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory.")?;

    // Ensure the output directory and wad file don't have the same path, which can happen if the
    // wad file had no extension for some reason
    if output_path == input_path {
        return Err(anyhow!("Input and output paths are the same."));
    }

    // Open big file

    let input_file = File::open(&input_path).context("Could not open file")?;

    let mut big_file = BigFile::new(input_file).expect("Could not open file as .big file");

    let bank_infos = big_file.read_bank_infos().expect("Could not read banks");

    for bank_info in &bank_infos {
        let bank_directory = output_path.join(bank_info.name.as_ref());

        fs::create_dir_all(&bank_directory).context("Could not create bank output directory")?;

        let (_asset_info_header, asset_infos) = big_file
            .read_asset_infos(&bank_info)
            .expect("Could not read asset info");

        for asset_info in &asset_infos {
            let asset_path = bank_directory.join(asset_info.symbol_name.as_ref());

            let asset_bytes = big_file
                .read_asset_bytes(&asset_info)
                .expect("Could not read asset bytes");

            fs::write(asset_path, asset_bytes).context("Could not write asset bytes")?;
        }
    }

    Ok(())
}
