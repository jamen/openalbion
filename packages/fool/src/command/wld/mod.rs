mod info;
pub use self::info::{WldInfoArgs, handler as info_handler};

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum WldCommand {
    #[command(arg_required_else_help = true)]
    Info(WldInfoArgs),
}

pub fn handler(command: WldCommand) -> anyhow::Result<()> {
    match command {
        WldCommand::Info(args) => info_handler(args),
    }
}
