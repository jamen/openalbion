use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::wad::WadReader;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
};

#[derive(Parser, Debug, Clone)]
pub struct WadUnpackArgs {
    /// Input wad file to be extracted
    input: PathBuf,

    /// Output directory to extract into (defaults to the input file's stem)
    output: Option<PathBuf>,
}

pub fn handler(args: WadUnpackArgs) -> anyhow::Result<()> {
    let input_path = args.input;

    // Get the output path, defaulting to one based on the wad file path if none is provided.
    let output_path = args
        .output
        .or_else(|| {
            input_path
                .parent()
                .and_then(|parent| input_path.file_stem().map(|stem| parent.join(stem)))
        })
        .context("No output directory.")?;

    // Ensure the output directory and wad file don't have the same path, which can happen if the
    // wad file had no extension for some reason.
    if output_path == input_path {
        return Err(anyhow!("Input and output paths are the same."));
    }

    log::info!("Wad file path {:?}", input_path);
    log::info!("Output path {:?}", output_path);

    let wad_file = File::open(&input_path).context("Could not open wad file")?;
    let wad_file = BufReader::new(wad_file);

    let mut wad_reader = WadReader::new(wad_file).context("Could not read wad file")?;

    // Snapshot the asset metadata first; reading content needs `&mut` on the reader.
    let assets: Vec<_> = wad_reader.asset_iter().cloned().collect();

    fs::create_dir_all(&output_path)
        .or_else(|error| match error.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(error),
        })
        .context("Failed to create output directory")?;

    for asset in &assets {
        // Wad entry paths are backslash-delimited Windows paths; take the file name.
        let file_name = asset
            .path
            .rsplit(['\\', '/'])
            .next()
            .filter(|s| !s.is_empty())
            .context("Failed to get wad entry's file name")?;

        let entry_output_path = output_path.join(file_name);

        let contents = wad_reader
            .read_content(asset)
            .with_context(|| format!("Failed to read content for {}", asset.path))?;

        log::info!("Extracting {} ({} bytes)", asset.path, contents.len());

        fs::write(&entry_output_path, contents)
            .with_context(|| format!("Failed to write {entry_output_path:?}"))?;
    }

    Ok(())
}
