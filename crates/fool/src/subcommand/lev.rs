use anyhow::anyhow;
use clap::{Args, Subcommand};
use fable_format::lev::LevHeader;
use std::{
    fs::File,
    io::{BufReader, Read},
};
use typed_path::Utf8PathBuf;

#[derive(Args, Debug, Clone)]
pub struct LevArgs {
    #[command(subcommand)]
    command: Option<LevCommand>,
}

#[derive(Subcommand, Debug, Clone)]
enum LevCommand {
    #[command(about = "Print information about a .lev file as JSON.")]
    Inspect {
        file: String,

        /// Compress the JSON
        #[arg(long, short)]
        compress: bool,
    },
}

pub fn handle(args: LevArgs) -> anyhow::Result<()> {
    match args.command {
        None => Ok(()),
        Some(LevCommand::Inspect { file, compress }) => inspect(file, compress),
    }
}

fn inspect(file_path: String, compress: bool) -> anyhow::Result<()> {
    let file_path = Utf8PathBuf::from(file_path);
    let file = File::open(&file_path).map_err(|_e| anyhow!("file not found."))?;
    let mut buf = BufReader::new(file);
    let mut header_bytes = vec![0; 1 << 23]; // 8MiB

    buf.read(&mut header_bytes)
        .map_err(|_e| anyhow!("could not read file."))?;

    let header = LevHeader::from_bytes(&header_bytes)
        .map_err(|e| anyhow!("could not parse header. {:?}", e))?;

    let json = serde_json::json!({ "header": header });

    let json_str = if compress {
        serde_json::to_string(&json)
            .map_err(|_| anyhow!("failed to serialize JSON (compressed)."))?
    } else {
        serde_json::to_string_pretty(&json).map_err(|_| anyhow!("failed to serialize JSON"))?
    };

    println!("{}", json_str);

    Ok(())
}
