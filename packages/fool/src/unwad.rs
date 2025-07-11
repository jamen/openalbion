use anyhow::{anyhow, Context};
use clap::Parser;
use fable_data::wad::WadReader;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug, Clone)]
pub struct UnwadArgs {
    // Input wad file to be extracted
    wad_file_path: PathBuf,

    // Output directory to extract into
    output_path: Option<PathBuf>,
}

pub fn handler(_fable_path: &Path, args: UnwadArgs) -> anyhow::Result<()> {
    let wad_file_path = args.wad_file_path;

    // Get the output path, defaulting to one based on the wad file path if none is provided
    let output_path = args
        .output_path
        .or_else(|| {
            wad_file_path
                .parent()
                .and_then(|parent| wad_file_path.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory provided and failed to decide a default path.")?;

    // Ensure the output directory and wad file don't have the same path, which can happen if the
    // wad file had no extension for some reason
    if output_path == wad_file_path {
        return Err(anyhow!(
            "Wad file and output directory have the same path. Please give a different output path"
        ));
    }

    log::info!("Wad file path {:?}", wad_file_path);
    log::info!("Output path {:?}", output_path);

    // Create the output directory. If an error is returned for it already existing, ignore
    fs::create_dir_all(&output_path)
        .or_else(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(error),
        })
        .context("Failed to create output directory")?;

    let wad_file = File::open(&wad_file_path).context("Could not open wad file")?;
    let wad_file = BufReader::new(wad_file);

    let mut wad_reader = WadReader::new(wad_file);

    let wad_entries_iter = wad_reader
        .read_entries()
        .context("Could not read wad entries")?;

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
