mod info;

use self::info::LevInfoArgs;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum LevCommand {
    #[command(arg_required_else_help = true)]
    Info(LevInfoArgs),
}

pub fn handler(command: LevCommand) -> anyhow::Result<()> {
    match command {
        LevCommand::Info(args) => info::handler(args),
    }
}
