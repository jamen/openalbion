mod extract_big;
mod extract_wad;
mod insert_texture_big;
mod logger;
mod pack_wad;

use crate::{
    extract_big::ExtractBigArgs, extract_wad::ExtractWadArgs,
    insert_texture_big::InsertTextureBigArgs, pack_wad::PackWadArgs,
};
use anyhow::Context;
use clap::{Parser, Subcommand};
use log::LevelFilter;
use std::{env, path::PathBuf};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool for modifying and inspecting Fable's files.",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'F', long)]
    fable_path: Option<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(arg_required_else_help = true)]
    ExtractWad(ExtractWadArgs),

    #[command(arg_required_else_help = true)]
    PackWad(PackWadArgs),

    #[command(arg_required_else_help = true)]
    ExtractBig(ExtractBigArgs),

    #[command(arg_required_else_help = true)]
    InsertTextureBig(InsertTextureBigArgs),
}

fn main() {
    if let Err(err) = try_main() {
        err.chain()
            .for_each(|err| log::error!("{}", err.to_string()))
    }
}

fn try_main() -> anyhow::Result<()> {
    logger::init(LevelFilter::Trace)?;

    let cli = Cli::parse();

    let fable_path = cli
        .fable_path
        .or_else(|| env::current_dir().ok())
        .context("Fable's directory not found. Try using the --fable-path flag")?;

    let command = match cli.command {
        None => return Ok(()),
        Some(command) => command,
    };

    match command {
        Commands::ExtractWad(args) => extract_wad::handler(fable_path.as_path(), args),
        Commands::PackWad(args) => pack_wad::handler(fable_path.as_path(), args),
        Commands::ExtractBig(args) => extract_big::handler(fable_path.as_path(), args),
        Commands::InsertTextureBig(args) => insert_texture_big::handler(fable_path.as_path(), args),
    }
}
