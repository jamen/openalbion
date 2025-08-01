use anyhow::Context;
use clap::Parser;
use fable_data::wad::{WadEntry, WadWriter};
use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Seek, SeekFrom},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug, Clone)]
pub struct PackWadArgs {
    /// Input directory whose files will be packed into a wad file
    input_directory_path: PathBuf,

    /// Wad file output path
    output_file_path: Option<PathBuf>,

    /// Prefix to use for wad entry paths. For example `"Data\Levels\FinalAlbion\"`
    #[clap(short = 'P', long, default_value = "")]
    entry_prefix: String,
}

pub fn handler(_fable_path: &Path, args: PackWadArgs) -> anyhow::Result<()> {
    let input_directory_path = args.input_directory_path;
    let entry_prefix = args.entry_prefix;

    // Get the output path, defaulting to one based on the input directory if none is provided.
    let output_path = args
        .output_file_path
        .or_else(|| Some(input_directory_path.with_extension("wad")))
        .context("No output directory provided and failed to decide a default path.")?;

    log::info!("Input directory path {:?}", input_directory_path);
    log::info!("Output path {:?}", output_path);

    let wad_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path)
        .context("Failed to open or create output file")?;

    let mut wad_file = BufWriter::new(wad_file);

    let block_size = 2048;
    let mut next_entry_position = block_size;
    let mut entries = vec![];

    wad_file
        .seek(SeekFrom::Start(block_size as u64))
        .context("Failed to seek file to first block")?;

    let mut wad_writer = WadWriter::new(wad_file);

    let input_directory_iter =
        fs::read_dir(input_directory_path).context("Failed to read input directory files")?;

    for (entry_id, entry_result) in input_directory_iter.enumerate() {
        let entry = entry_result.context("Failed to read input directory entry")?;

        let entry_path = entry.path();

        let entry_content = fs::read(&entry_path).context("Failed to read file")?;

        let entry_file_name = entry_path
            .file_name()
            .context("Failed to get file name")?
            .to_str()
            .context("Failed to get file name as string")?;

        let written = wad_writer
            .write_content(&entry_content, block_size)
            .context("Failed to write file into wad archive")?;

        let content_position = next_entry_position;
        let content_length = entry_content.len();
        let path = format!("{}{}", entry_prefix, entry_file_name);

        let entry = WadEntry {
            id: entry_id as u32,
            content_length: content_length as u32,
            content_position: content_position as u32,
            path,
            unknown_1: [0; 16],
            unknown_2: 0,
            unknown_3: 0,
            unknown_4: [0; 16],
            created: [0; 7],
            accessed: [0; 7],
            modified: [0; 5],
        };

        entries.push(entry);

        next_entry_position += written;
    }

    wad_writer
        .write_entries(&entries)
        .context("Failed to write entries list to wad archive")?;

    Ok(())
}
