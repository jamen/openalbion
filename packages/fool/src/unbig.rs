use anyhow::{anyhow, Context};
use clap::Parser;
use fable_data::big::BigReader;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug, Clone)]
pub struct UnbigArgs {
    // Input wad file to be extracted
    big_file_path: PathBuf,

    // Output directory to extract into
    output_path: Option<PathBuf>,
}

pub fn handler(_fable_data: &Path, args: UnbigArgs) -> anyhow::Result<()> {
    let big_file_path = args.big_file_path;

    // Get the output path, defaulting to one based on the input file path if none is provided
    let output_path = args
        .output_path
        .or_else(|| {
            big_file_path
                .parent()
                .and_then(|parent| big_file_path.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory provided and failed to decide a default path.")?;

    // Ensure the output directory and wad file don't have the same path, which can happen if the
    // wad file had no extension for some reason
    if output_path == big_file_path {
        return Err(anyhow!(
            "Wad file and output directory have the same path. Please give a different output path"
        ));
    }

    log::info!("Big file path {:?}", big_file_path);
    log::info!("Output path {:?}", output_path);

    let big_file = File::open(&big_file_path).context("Could not open wad file")?;
    let big_file = BufReader::new(big_file);

    let mut big_reader = BigReader::new(big_file);

    let index = big_reader
        .read_index()
        .context("Could not read big index")?;

    // println!("{:#?}", index);

    for index_entry in &index.entries {
        let bank = big_reader
            .read_index_entry(&index_entry)
            .context("Could not read first index entry.")?;

        // println!("{:?}", bank);
    }

    Ok(())
}
