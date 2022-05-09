use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    println!("{:?}", args);
}
