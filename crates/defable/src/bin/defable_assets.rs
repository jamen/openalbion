use std::fs::File;
use std::path::Path;
use std::ffi::OsStr;

use clap::{App,Arg};

use fable::{Decode,Def};

fn main() -> Result<(), u32> {
    let matches =
        App::new("defable_assets")
        .version("0.1.0")
        .about("For using the assets of Fable, Fable: The Lost Chapters, Fable Anniversary, and their mods.")
        .author("Jamen Marz <me@jamen.dev>")
        .arg(
            Arg::with_name("decode")
            .long("decode")
            .short("d")
            .help("Parse a data file.")
            .required(false)
            .takes_value(true)
        )
        .get_matches();

    if let Some(data_file) = matches.value_of("decode") {
        let path = Path::new(data_file);

        if path.extension() == Some(OsStr::new("def")) ||
           path.extension() == Some(OsStr::new("tpl")) {
            let mut def_file = File::open(path).unwrap();
            let def: Def = def_file.decode().unwrap();
            println!("{:#?}", def);
        }
    }

    Ok(())
}