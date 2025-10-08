use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::big::BigReader;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug, Clone)]
pub struct ExtractBigArgs {
    /// Input big file to be extracted
    big_file: PathBuf,

    /// Output directory to extract into
    output: Option<PathBuf>,

    /// Select specific banks to extract. Defaults to all banks
    banks: Option<String>,
}

pub fn handler(_fable_data: &Path, args: ExtractBigArgs) -> anyhow::Result<()> {
    let big_file = args.big_file;

    // Get the output path, defaulting to one based on the input file path if none is provided
    let output = args
        .output
        .or_else(|| {
            big_file
                .parent()
                .and_then(|parent| big_file.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory provided and failed to decide a default path.")?;

    // Ensure the output directory and wad file don't have the same path, which can happen if the
    // wad file had no extension for some reason
    if output == big_file {
        return Err(anyhow!(
            "Wad file and output directory have the same path. Please give a different output path"
        ));
    }

    // Open big file

    let big_file = File::open(&big_file).context("Could not open wad file")?;
    let big_file = BufReader::new(big_file);

    let mut big_reader = BigReader::new(big_file);

    let index = big_reader
        .read_index()
        .context("Could not read big index")?;

    log::info!("Opened big file {:?}", output);

    // Create base directory

    fs::create_dir_all(&output)
        .or_else(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(error),
        })
        .context("Could not create output directory")?;

    log::info!("Created out directory {:?}", output);

    // Create bank sub-directories

    for index_entry in &index.entries {
        let bank_output = output.join(&index_entry.name);

        fs::create_dir(&bank_output).context("Could not create bank output directory")?;

        log::info!("Created bank output directory {:?}", bank_output);

        // Extract textures

        let dxt1_textures_output = bank_output.join("DXT1");

        fs::create_dir(&bank_output);

        // TODO: Extract any other possible assets
    }

    Ok(())
}
