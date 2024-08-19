use anyhow::anyhow;
use clap::{Args, Subcommand};
use format::Tng;
use std::fs;
use typed_path::Utf8PathBuf;

#[derive(Args, Debug, Clone)]
pub struct TngArgs {
    #[command(subcommand)]
    command: Option<TngCommand>,
}

#[derive(Subcommand, Debug, Clone)]
enum TngCommand {
    #[command(about = "Inspect a .tng file")]
    Inspect { file: String },
}

pub fn handle(args: TngArgs) -> anyhow::Result<()> {
    match args.command {
        None => Ok(()),
        Some(TngCommand::Inspect { file }) => inspect(file),
    }
}

fn inspect(file_path: String) -> anyhow::Result<()> {
    let file_path = Utf8PathBuf::from(file_path);
    let tng_source = fs::read_to_string(file_path).map_err(|_| anyhow!("failed to read file."))?;

    let tng = Tng::parse(&tng_source);

    println!("{:#?}", tng);

    Ok(())

    // let mut lex = Lexer::new(&tng_source);

    // loop {
    //     match lex.next_token() {
    //         Ok(Some(token)) => {
    //             println!(
    //                 "{:<5} {:<11} {:?}",
    //                 format!("{}:{}", token.location.line + 1, token.location.column + 1),
    //                 format!("{:?}", token.kind),
    //                 token.text
    //             )
    //         }
    //         Ok(None) => return Ok(()),
    //         Err(e) => return Err(anyhow!("failed to lex file: {:?}", e)),
    //     }
    // }
}
