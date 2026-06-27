use anyhow::Context;
use clap::Parser;
use fable_data::wad::{AssetMetadata, Header, WadWriter};
use std::{
    borrow::Cow,
    fs::{self, OpenOptions},
    io::{BufWriter, SeekFrom},
    path::PathBuf,
};

#[derive(Parser, Debug, Clone)]
pub struct WadPackArgs {
    /// Input directory whose files will be packed into a wad file
    input: PathBuf,

    /// Wad file output path (defaults to the input directory name with a `.wad` extension)
    output: Option<PathBuf>,

    /// Prefix to use for wad entry paths. For example `"Data\Levels\FinalAlbion\"`
    #[clap(short = 'P', long, default_value = "")]
    entry_prefix: String,
}

pub fn handler(args: WadPackArgs) -> anyhow::Result<()> {
    let input_directory_path = args.input;
    let entry_prefix = args.entry_prefix;

    let output_path = args
        .output
        .or_else(|| Some(input_directory_path.with_extension("wad")))
        .context("No output directory provided and failed to decide a default path.")?;

    log::info!("Input directory path {:?}", input_directory_path);
    log::info!("Output path {:?}", output_path);

    let wad_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_path)
        .context("Failed to open or create output file")?;

    let block_size: usize = 2048;

    let mut wad_writer = WadWriter::new(BufWriter::new(wad_file));

    // Content begins after the first block; the header lives in the first 32 bytes and the rest of
    // the block is left as zero padding (written when we seek past it).
    wad_writer
        .seek(SeekFrom::Start(block_size as u64))
        .context("Failed to seek to first content block")?;

    let mut next_entry_position = block_size;
    let mut entries: Vec<AssetMetadata> = Vec::new();

    let input_directory_iter =
        fs::read_dir(&input_directory_path).context("Failed to read input directory files")?;

    for (entry_id, entry_result) in input_directory_iter.enumerate() {
        let entry = entry_result.context("Failed to read input directory entry")?;
        let entry_path = entry.path();

        if !entry_path.is_file() {
            continue;
        }

        let entry_content = fs::read(&entry_path).context("Failed to read file")?;

        let entry_file_name = entry_path
            .file_name()
            .context("Failed to get file name")?
            .to_str()
            .context("File name is not valid UTF-8")?;

        let content_position = next_entry_position;
        let content_length = entry_content.len();

        let written = wad_writer
            .write_content(&entry_content, block_size)
            .context("Failed to write file into wad archive")?;

        entries.push(AssetMetadata {
            unknown_1: [0; 16],
            id: entry_id as u32,
            unknown_2: 0,
            content_length: content_length as u32,
            content_position: content_position as u32,
            unknown_3: 0,
            path: Cow::Owned(format!("{entry_prefix}{entry_file_name}")),
            unknown_4: [0; 16],
            created: [0; 7],
            accessed: [0; 7],
            modified: [0; 5],
        });

        log::info!("Packed {} ({} bytes)", entry_file_name, content_length);

        next_entry_position += written;
    }

    let asset_table_position = next_entry_position as u32;

    wad_writer
        .write_entries(&entries)
        .context("Failed to write entries list to wad archive")?;

    // Version [115, 2, 1] matches the retail FinalAlbion.wad header. The reader does not validate
    // it, but the game likely does, so we mirror the real value.
    let header = Header {
        magic: Header::MAGIC,
        version: [115, 2, 1],
        block_size: block_size as u32,
        asset_count: entries.len() as u32,
        asset_count_2: entries.len() as u32,
        asset_table_position,
    };

    wad_writer
        .write_header(&header)
        .context("Failed to write wad header")?;

    log::info!("Wrote {} entries to {:?}", entries.len(), output_path);

    Ok(())
}
