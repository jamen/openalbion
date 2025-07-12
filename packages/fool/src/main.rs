mod logger;
mod unbig;
mod unwad;
mod wad;

use crate::{unbig::UnbigArgs, unwad::UnwadArgs, wad::WadArgs};
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
    Unwad(UnwadArgs),

    #[command(arg_required_else_help = true)]
    Wad(WadArgs),

    #[command(arg_required_else_help = true)]
    Unbig(UnbigArgs),
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
        Commands::Unwad(args) => unwad::handler(fable_path.as_path(), args),
        Commands::Wad(args) => wad::handler(fable_path.as_path(), args),
        Commands::Unbig(args) => unbig::handler(fable_path.as_path(), args),
    }
}
