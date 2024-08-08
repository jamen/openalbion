use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
pub struct LevArgs {
    #[command(subcommand)]
    command: Option<LevCommand>,
}

#[derive(Subcommand, Debug, Clone)]
enum LevCommand {}

pub fn handle(args: LevArgs) -> anyhow::Result<()> {
    Ok(())
}
