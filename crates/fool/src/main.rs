mod subcommand;

use clap::{Parser, Subcommand};
use subcommand::{LevArgs, TngArgs, WadArgs};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Utility to extract, pack, or inspect Fable's files.",
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Wad(WadArgs),

    #[command(arg_required_else_help = true)]
    Lev(LevArgs),

    #[command(arg_required_else_help = true)]
    Tng(TngArgs),
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
        Some(Commands::Wad(args)) => subcommand::wad::handle(args),
        Some(Commands::Lev(args)) => subcommand::lev::handle(args),
        Some(Commands::Tng(args)) => subcommand::tng::handle(args),
    }
}
