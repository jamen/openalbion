mod export;
mod import;

use self::{export::TextureExportArgs, import::TextureImportArgs};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum TextureCommand {
    #[command(arg_required_else_help = true)]
    Export(TextureExportArgs),

    #[command(arg_required_else_help = true)]
    Import(TextureImportArgs),
}

pub fn handler(command: TextureCommand) -> anyhow::Result<()> {
    match command {
        TextureCommand::Export(args) => export::handler(args),
        TextureCommand::Import(args) => import::handler(args),
    }
}
