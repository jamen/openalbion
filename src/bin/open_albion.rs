use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let maybe_fable_directory = args.get(0).map(|x| PathBuf::from(x));

    open_albion::App::run(maybe_fable_directory)
}