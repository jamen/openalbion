use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("./minilzo/minilzo.c")
        .include("./minilzo")
        .include("./minilzo/include/lzo")
        .compile("minilzo");
}
