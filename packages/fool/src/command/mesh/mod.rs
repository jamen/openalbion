mod info;
mod list;

use self::{info::MeshInfoArgs, list::MeshListArgs};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum MeshCommand {
    /// List every mesh asset in a .big archive.
    #[command(arg_required_else_help = true)]
    List(MeshListArgs),

    /// Decode and summarise a single mesh.
    #[command(arg_required_else_help = true)]
    Info(MeshInfoArgs),
}

pub fn handler(command: MeshCommand) -> anyhow::Result<()> {
    match command {
        MeshCommand::List(args) => list::handler(args),
        MeshCommand::Info(args) => info::handler(args),
    }
}
