mod dump;
mod info;
mod list;

use self::{dump::FmpDumpArgs, info::FmpInfoArgs, list::FmpListArgs};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum FmpCommand {
    #[command(arg_required_else_help = true)]
    Info(FmpInfoArgs),

    #[command(arg_required_else_help = true)]
    List(FmpListArgs),

    #[command(arg_required_else_help = true)]
    Dump(FmpDumpArgs),
}

pub fn handler(command: FmpCommand) -> anyhow::Result<()> {
    match command {
        FmpCommand::Info(args) => info::handler(args),
        FmpCommand::List(args) => list::handler(args),
        FmpCommand::Dump(args) => dump::handler(args),
    }
}
