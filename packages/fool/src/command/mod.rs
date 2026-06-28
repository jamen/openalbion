mod big;
mod def;
mod lev;
mod mesh;
mod texture;
mod tng;
mod wad;
mod wld;

use self::{
    big::BigCommand, def::DefCommand, lev::LevCommand, mesh::MeshCommand, texture::TextureCommand,
    tng::TngCommand, wad::WadCommand, wld::WldCommand,
};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    #[command(subcommand, arg_required_else_help = true)]
    Big(BigCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Def(DefCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Wad(WadCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Lev(LevCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Mesh(MeshCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Texture(TextureCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Tng(TngCommand),

    #[command(subcommand, arg_required_else_help = true)]
    Wld(WldCommand),
}

pub fn handler(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Big(c) => big::handler(c),
        Command::Def(c) => def::handler(c),
        Command::Wad(c) => wad::handler(c),
        Command::Lev(c) => lev::handler(c),
        Command::Mesh(c) => mesh::handler(c),
        Command::Texture(c) => texture::handler(c),
        Command::Tng(c) => tng::handler(c),
        Command::Wld(c) => wld::handler(c),
    }
}
