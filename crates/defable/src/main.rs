pub mod commands;

use clap::App;

fn main() {
    let matches = App::new("defable")
        .version("0.1.0")
        .author("Jamen Marz <me@jamen.dev>")
        .subcommand(commands::cheat::register())
        .subcommand(commands::wad::register())
        .subcommand(commands::lev::register())
        .get_matches();

    match matches.subcommand_name() {
        None => { commands::app::main(&matches) },
        Some("cheat") => commands::cheat::main(matches.subcommand_matches("cheat").unwrap()),
        Some("wad") => commands::wad::main(matches.subcommand_matches("wad").unwrap()),
        Some("lev") => commands::lev::main(matches.subcommand_matches("lev").unwrap()),
        _ => { eprintln!("Unknown subcommand."); }
    };
}