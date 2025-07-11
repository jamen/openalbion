use clap::{Parser, Subcommand};
use fable_data::wad::WadReader;
use std::{env, fs::File, io::BufReader, path::PathBuf};

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
    Info(InfoArgs),
}

#[derive(Parser, Debug, Clone)]
struct InfoArgs {
    #[arg(short = 'F', long)]
    fable_path: Option<PathBuf>,
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
        Some(Commands::Info(args)) => info(args),
    }
}

fn info(args: InfoArgs) -> anyhow::Result<()> {
    let fable_path = args
        .fable_path
        .ok_or(())
        .or_else(|_| default_fable_data())
        .expect("error: Couldn't determine Fable's directory path. Try --fable-path.");

    let final_albion_wad_file = File::open(fable_path.join("data/Levels/FinalAlbion.wad"))?;
    let final_albion_wad_file = BufReader::new(final_albion_wad_file);

    let mut final_albion_wad = WadReader::new(final_albion_wad_file)?;

    let entries = final_albion_wad.entries();

    println!("{:#?}", entries);

    Ok(())
}

fn default_fable_data() -> Result<PathBuf, std::io::Error> {
    env::current_dir()
}
