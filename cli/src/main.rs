use unfable_parser::wad;
use std::fs::File;
use std::env;
use std::collections::HashMap;

fn main() {
    let input = env::args().nth(1).expect("Input required.");
    let output = env::args().nth(2).expect("Output required.");

    let mut wad_file = File::open(input).expect("Could not open file.");

    let mut wad = wad::Wad::new(&mut wad_file).expect("Could not create wad.");

    let empty: HashMap<String, wad::FileOption> = HashMap::new();

    wad.copy(output, empty).expect("Failed to copy files to output.");
}
