use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::WadReader;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
};

#[derive(Parser, Debug, Clone)]
pub struct WadUnpackArgs {
    /// Input wad file to be extracted
    input: PathBuf,

    /// Output directory to extract into
    output: Option<PathBuf>,
}

pub fn handler(args: WadUnpackArgs) -> anyhow::Result<()> {
    let input_path = args.input;

    // Get the output path, defaulting to one based on the wad file path if none is provided
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

    log::info!("Wad file path {:?}", input_path);
    log::info!("Output path {:?}", output_path);

    let wad_file = File::open(&input_path).context("Could not open wad file")?;
    let wad_file = BufReader::new(wad_file);

    let mut wad_reader = WadReader::new(wad_file);

    let wad_entries_iter = wad_reader
        .read_entries()
        .context("Could not read wad entries")?;

    // Create the output directory. If an error is returned for it already existing, ignore
    fs::create_dir_all(&output_path)
        .or_else(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(error),
        })
        .context("Failed to create output directory")?;

    for wad_entry_result in wad_entries_iter.into_iterator() {
        let wad_entry = wad_entry_result.context("Failed to read wad entry")?;

        let wad_entry_file_name = wad_entry
            .path
            .split("\\")
            .last()
            .context("Failed to get wad entry's file name")?;

        let wad_entry_output_path = output_path.join(wad_entry_file_name);

        let wad_entry_contents = wad_reader
            .read_content(&wad_entry)
            .context("Failed to read wad entry's file contents")?;

        log::info!("Extracting {} \n {:#?}", wad_entry_file_name, wad_entry);

        fs::write(wad_entry_output_path, wad_entry_contents)
            .context("Failed to write wad entry's contents to file")?;
    }

    Ok(())
}
