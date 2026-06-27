mod command;
mod logger;

use clap::Parser;
use command::Command;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A tool for modifying and inspecting Fable's files.",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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

    let command = match cli.command {
        None => return Ok(()),
        Some(command) => command,
    };

    command::handler(command)
}
