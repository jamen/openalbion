use anyhow::{Context, anyhow};
use clap::Parser;
use fable_data::fmp::FmpReader;
use std::{fs, fs::File, path::PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct FmpDumpArgs {
    /// Input fmp file to be extracted
    input: PathBuf,

    /// Output directory to extract into (defaults to the input file's stem)
    output: Option<PathBuf>,
}

pub fn handler(args: FmpDumpArgs) -> anyhow::Result<()> {
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

    // Ensure the output directory and fmp file don't have the same path, which can happen if the
    // fmp file had no extension for some reason.
    if output_path == input_path {
        return Err(anyhow!("Input and output paths are the same."));
    }

    log::info!("Fmp file path {:?}", input_path);
    log::info!("Output path {:?}", output_path);

    let input_file = File::open(&input_path).context("Could not open fmp file")?;

    let mut reader =
        FmpReader::new(input_file).map_err(|e| anyhow!("Could not read fmp file: {e}"))?;

    // Bank contents are opaque at the container level, so each is written out as raw bytes for a
    // format-specific tool to read. Snapshot the metadata first: `read_bank_raw` needs `&mut`.
    let banks: Vec<_> = reader.banks().to_vec();

    fs::create_dir_all(&output_path).context("Could not create output directory")?;

    for bank in &banks {
        let data = reader
            .read_bank_raw(bank)
            .with_context(|| format!("Could not read bank {}", bank.name))?;

        let bank_path = output_path.join(format!("{}.bin", bank.name));

        log::info!("Extracting {} ({} bytes)", bank.name, data.len());

        fs::write(&bank_path, data)
            .with_context(|| format!("Could not write bank bytes to {bank_path:?}"))?;
    }

    Ok(())
}
