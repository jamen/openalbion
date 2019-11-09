pub mod commands;

use clap::App;

fn main() {
    let matches = App::new("defable")
        .version("0.1.0")
        .author("Jamen Marz <me@jamen.dev>")
        .subcommand(commands::wad::register())
        .subcommand(commands::lev::register())
        .get_matches();

    if let Some(sub_name) = matches.subcommand_name() {
        let sub_matches = matches.subcommand_matches(sub_name).unwrap();

        match sub_name {
            "wad" => commands::wad::main(sub_matches),
            "lev" => commands::lev::main(sub_matches),
            _ => {},
        };
    };
}

