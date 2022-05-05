use std::{
    fs::{self, File, OpenOptions},
    io::BufReader,
};

use clap::Parser;

use fable_data::{Big, BigKind};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let file = fs::read(&args.file).expect("Failed to open file");

    let big = Big::parse(&mut &file[..], BigKind::guess_from_path(&args.file)).unwrap();

    println!("file: {:?}", big);
}
