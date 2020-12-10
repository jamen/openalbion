use std::env;
use std::path::PathBuf;

use open_albion::Engine;

struct OpenAlbion {
    engine: Engine
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // TODO: Use some GUI like an open dialog instead.
    let fable_directory = args.get(0)
        .map(|x| PathBuf::from(x))
        .or(env::current_dir().ok())
        .expect("Could not determine Fable's directory.");

    let open_albion = Engine::create(fable_directory);

    open_albion.run()
}