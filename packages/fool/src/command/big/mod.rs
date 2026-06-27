mod dump;

use self::dump::BigDumpArgs;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum BigCommand {
    #[command(arg_required_else_help = true)]
    Dump(BigDumpArgs),
}

pub fn handler(command: BigCommand) -> anyhow::Result<()> {
    match command {
        BigCommand::Dump(args) => dump::handler(args),
    }
}
