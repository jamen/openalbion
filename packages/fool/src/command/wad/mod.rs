mod pack;
mod unpack;

use pack::WadPackArgs;
use unpack::WadUnpackArgs;

use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum WadCommand {
    #[command(arg_required_else_help = true)]
    Pack(WadPackArgs),

    #[command(arg_required_else_help = true)]
    Unpack(WadUnpackArgs),
}

pub fn handler(command: WadCommand) -> anyhow::Result<()> {
    match command {
        WadCommand::Pack(args) => pack::handler(args),
        WadCommand::Unpack(args) => unpack::handler(args),
    }
}
