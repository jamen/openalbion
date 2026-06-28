mod list;
mod roundtrip;

use self::{list::DefListArgs, roundtrip::DefRoundtripArgs};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum DefCommand {
    /// List the def types in a compiled def binary (game.bin) with instance counts and whether
    /// fable-data has a typed parser for each.
    #[command(arg_required_else_help = true)]
    List(DefListArgs),

    /// Round-trip def records (parse → serialize → compare to the original bytes) to validate
    /// typed def parsers against retail data.
    #[command(arg_required_else_help = true)]
    Roundtrip(DefRoundtripArgs),
}

pub fn handler(command: DefCommand) -> anyhow::Result<()> {
    match command {
        DefCommand::List(args) => list::handler(args),
        DefCommand::Roundtrip(args) => roundtrip::handler(args),
    }
}
