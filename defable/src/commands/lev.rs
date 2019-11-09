use clap::{App, SubCommand, Arg, ArgMatches};
use fable_format::lev::Lev;

pub fn register<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("lev")
        .about("Lev map data tool.")
        .arg(
            Arg::with_name("wad")
            .help("Select the Lev from a Wad archive.")
            .short("w")
            .long("wad")
            .value_name("WAD_FILE")
            .number_of_values(1)
        )
        .arg(
            Arg::with_name("info")
            .help("Show info about a Lev file.")
            .short("i")
            .long("info")
            .value_name("LEV_FILE")
        )
        .arg(
            Arg::with_name("create")
            .help("Create a Lev from a glTF mesh.")
            .short("c")
            .long("create")
            .value_names(&["GLTF_FILE", "LEV_FILE"])
            .conflicts_with("wad")
        )
        .arg(
            Arg::with_name("extract")
            .help("Extracts a Lev to a glTF mesh.")
            .short("e")
            .long("extract")
            .value_names(&["LEV_FILE", "GLTF_FILE"])
        )
}

pub fn main<'a>(matches: &ArgMatches<'a>) {
    if let Some(lev_file) = matches.value_of("info") {
        // let lev = match matches.value_of("wad") {
        //     Some(wad_file) => Lev::from_wad(wad_file, lev_file),
        //     None => Lev::from_file(lev_file),
        // };

        let lev = Lev::open(lev_file);

        match lev {
            Ok(lev) => println!("{:#?}", lev),
            Err(error) => println!("Failed: {:?}", error),
        }
    }
}