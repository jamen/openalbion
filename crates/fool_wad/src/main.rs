use anyhow::anyhow;
use clap::{Parser, Subcommand};
use format::{WadEntry, WadHeader};
use std::{
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
};
use typed_path::{Utf8PathBuf, Utf8WindowsEncoding};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Command line utility to extract, pack, or inspect Fable's .wad files.",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(about = "Extract a .wad file into a directory.")]
    Extract {
        file: String,

        #[arg(long, short)]
        output: Option<String>,
    },

    #[command(about = "Pack a directory into a .wad file.")]
    Pack {
        directory: String,

        #[arg(long, short)]
        output: Option<String>,
    },

    #[command(about = "Print information about a .wad file as JSON.")]
    Inspect {
        file: String,

        /// Compress the JSON
        #[arg(long, short)]
        compress: bool,
    },
}

fn main() {
    if let Err(err) = try_main() {
        let renderer = annotate_snippets::Renderer::styled();

        err.chain().for_each(|err| {
            let message = err.to_string();
            let snippet = annotate_snippets::Level::Error.title(&message);
            anstream::eprintln!("{}", renderer.render(snippet));
        })
    }
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => Ok(()),
        Some(Commands::Extract { file, output }) => extract(file, output),
        Some(Commands::Pack { directory, output }) => pack(directory, output),
        Some(Commands::Inspect { file, compress }) => inspect(file, compress),
    }
}

fn extract(file_path: String, output_path: Option<String>) -> anyhow::Result<()> {
    let file_path = Utf8PathBuf::from(file_path);
    let file = File::open(&file_path).map_err(|_e| anyhow!("file not found."))?;
    let mut buf = BufReader::new(file);
    let mut header_bytes = vec![0; WadHeader::byte_size()];

    buf.read_exact(&mut header_bytes)
        .map_err(|_e| anyhow!("could not read file."))?;

    let header =
        WadHeader::from_bytes(&header_bytes).map_err(|_e| anyhow!("could not parse header."))?;

    let output_path = output_path
        .map(|x| Utf8PathBuf::from(x))
        .or_else(|| {
            let file_stem = file_path.file_stem()?;

            file_path
                .parent()
                .map(|x| x.to_path_buf())
                .map(|x| x.join(file_stem))
        })
        .ok_or_else(|| anyhow!("could not determine output path."))?;

    fs::create_dir_all(&output_path)
        .map_err(|_e| anyhow!("failed to establish output directory"))?;

    let entry_count = header.entry_count as usize;
    let mut current_position = header.first_entry_position as usize;
    let mut entry_bytes = vec![0; 1 << 13]; // 8KiB
    let mut entries = Vec::with_capacity(entry_count);

    for _ in 0..entry_count {
        buf.seek(SeekFrom::Start(current_position as u64))
            .map_err(|_e| anyhow!("failed to seek through wad file."))?;

        buf.read(&mut entry_bytes)
            .map_err(|_e| anyhow!("failed to read entry."))?;

        let entry = WadEntry::from_bytes(&entry_bytes)
            .map_err(|e| anyhow!("failed to parse entry. {:?}", e))?;

        current_position += entry.byte_size();

        entries.push(entry.to_owned());
    }

    for entry in &entries {
        let entry_path = std::str::from_utf8(&entry.path)
            .map_err(|_e| anyhow!("failed to convert entry path to string."))?;

        let entry_path = Utf8PathBuf::<Utf8WindowsEncoding>::from(entry_path);

        let entry_file_name = entry_path
            .file_name()
            .ok_or_else(|| anyhow!("failed to determine entry file name."))?;

        let entry_output_path = output_path.join(entry_file_name);

        buf.seek(SeekFrom::Start(entry.offset as u64))
            .map_err(|_| anyhow!("failed to seek to entry data."))?;

        let mut entry_file_bytes = vec![0; entry.length as usize];

        buf.read_exact(&mut entry_file_bytes)
            .map_err(|_| anyhow!("failed to read entry data."))?;

        fs::write(entry_output_path, &entry_file_bytes)
            .map_err(|_| anyhow!("failed to write entry."))?;
    }

    Ok(())
}

fn pack(directory: String, output: Option<String>) -> anyhow::Result<()> {
    Ok(())
}

fn inspect(file: String, compress: bool) -> anyhow::Result<()> {
    Ok(())
}
