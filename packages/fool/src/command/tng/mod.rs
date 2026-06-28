mod info;
pub use self::info::{TngInfoArgs, handler as info_handler};

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum TngCommand {
    #[command(arg_required_else_help = true)]
    Info(TngInfoArgs),
}

pub fn handler(command: TngCommand) -> anyhow::Result<()> {
    match command {
        TngCommand::Info(args) => info_handler(args),
    }
}
