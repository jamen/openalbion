use std::fs::File;
use std::io::Write;
use clap::{App, SubCommand, Arg, ArgMatches};
use fable_format::lev::Lev;
use fable_gltf::encode_lev_to_mesh;
use gltf_json::serialize::to_string_pretty;

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
            Arg::with_name("unpack")
            .help("Extracts a Lev to a glTF mesh.")
            .short("u")
            .long("unpack")
            .value_names(&["LEV_FILE", "GLTF_FILE"])
        )
}

pub fn main<'a>(matches: &ArgMatches<'a>) {
    if let Some(lev_file) = matches.value_of("info") {
        // let lev = match matches.value_of("wad") {
        //     Some(wad_file) => Lev::from_wad(wad_file, lev_file),
        //     None => Lev::from_file(lev_file),
        // };

        let lev = Lev::open(lev_file).unwrap();

        println!("{:#?}", lev);
    }

    if let Some(mut inputs) = matches.values_of("unpack") {
        let lev_file = inputs.next().unwrap();
        let gltf_file = inputs.next().unwrap();

        let lev = Lev::open(lev_file).unwrap();

        let bin_file = [gltf_file, ".bin"].concat();

        let (bin_data, root) = encode_lev_to_mesh(lev, &bin_file).unwrap();

        let mut bin = File::create(&bin_file).unwrap();
        bin.write(&bin_data).unwrap();

        let gltf_data = gltf_json::serialize::to_string_pretty(&root).unwrap();
        let mut gltf = File::create(gltf_file).unwrap();
        gltf.write(gltf_data.as_bytes()).unwrap();
    }
}