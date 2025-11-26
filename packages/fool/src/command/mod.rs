mod big;
mod texture;
mod wad;

use self::{big::BigCommand, texture::TextureCommand, wad::WadCommand};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[command(subcommand, arg_required_else_help = true)]
    Big(BigCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Wad(WadCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Texture(TextureCommand),
}

pub fn handler(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Big(c) => big::handler(c),
        Command::Wad(c) => wad::handler(c),
        Command::Texture(c) => texture::handler(c),
    }
}
