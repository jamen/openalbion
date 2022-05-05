use std::{
    fs::{self, File, OpenOptions},
    io::BufReader,
};

use clap::Parser;

use fable_data::Big;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let file = fs::read(&args.file).expect("Failed to open file");

    let (_, big) = Big::parse(&file[..]).unwrap();

    println!("{:?}", big);
}
