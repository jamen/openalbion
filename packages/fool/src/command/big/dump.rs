use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::big::BigReader;
use std::{
    fs::{self, File},
    path::PathBuf,
};

#[derive(Parser, Debug, Clone)]
pub struct BigDumpArgs {
    /// Input big file to be extracted
    input: PathBuf,

    /// Output directory to extract into (defaults to the input file's stem)
    output: Option<PathBuf>,
}

pub fn handler(args: BigDumpArgs) -> anyhow::Result<()> {
    let input_path = args.input;

    // Get the output path, defaulting to one based on the input file path if none is provided.
    let output_path = args
        .output
        .or_else(|| {
            input_path
                .parent()
                .and_then(|parent| input_path.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory.")?;

    // Ensure the output directory and big file don't have the same path, which can happen if the
    // big file had no extension for some reason.
    if output_path == input_path {
        return Err(anyhow!("Input and output paths are the same."));
    }

    log::info!("Big file path {:?}", input_path);
    log::info!("Output path {:?}", output_path);

    let input_file = File::open(&input_path).context("Could not open big file")?;

    let mut big_reader = BigReader::new(input_file).context("Could not open file as .big file")?;

    // Collect (bank, asset) names up front. The reader hands out `&Bank`/`&AssetMetadata`
    // borrows, which would conflict with the `&mut self` needed by `read_asset`, so snapshot
    // the identifiers first and then read.
    let targets: Vec<(String, String)> = big_reader
        .bank_iter()
        .flat_map(|bank| {
            let bank_name = bank.metadata().name.to_string();
            bank.asset_iter()
                .map(move |asset| (bank_name.clone(), asset.symbol_name.to_string()))
        })
        .collect();

    log::info!("Extracting {} assets", targets.len());

    for (bank_name, symbol_name) in &targets {
        let (_metadata, bytes) = big_reader
            .read_asset(bank_name, symbol_name)
            .with_context(|| format!("Could not read asset {bank_name}/{symbol_name}"))?;

        let bank_directory = output_path.join(bank_name);

        fs::create_dir_all(&bank_directory).context("Could not create bank output directory")?;

        let asset_path = bank_directory.join(symbol_name);

        fs::write(&asset_path, bytes)
            .with_context(|| format!("Could not write asset bytes to {asset_path:?}"))?;
    }

    Ok(())
}
