use clap::{App, SubCommand, Arg, ArgMatches};
use fable_format::wad::Wad;
use std::collections::HashMap;

pub fn register<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("wad")
        .about("Wad archive tool.")
        .arg(
            Arg::with_name("extract")
            .help("Unpack a Wad to a directory.")
            .short("e")
            .long("extract")
            .value_names(&["WAD_FILE", "DIRECTORY"])
        )
        .arg(
            Arg::with_name("create")
            .help("Pack a directory into a Wad.")
            .short("c")
            .long("create")
            .value_names(&["DIRECTORY", "WAD_FILE"])
        )
        .arg(
            Arg::with_name("info")
            .help("Show info about a Wad file.")
            .short("i")
            .long("info")
            .value_name("WAD_FILE")
        )
}

pub fn main<'a>(matches: &ArgMatches<'a>) {
    if let Some(wad_file) = matches.value_of("info") {
        let wad = Wad::open(wad_file);

        match wad {
            Ok(wad) => println!("{:#?}", wad),
            Err(error) => println!("Failed: {:?}", error),
        }
    }

    if let Some(mut inputs) = matches.values_of("unpack") {
        let wad_file = inputs.next().unwrap();
        let directory = inputs.next().unwrap();

        let wad = Wad::open(wad_file).unwrap();

        let file_options = HashMap::default();

        // TODO: Populate file_options (excludes and includes) from matches.

        wad.unpack(directory, file_options).unwrap();
    }
}