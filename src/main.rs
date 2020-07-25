use std::path::{Path,PathBuf};
use std::fs::File;
use std::time::Instant;
// use std::ffi::OsStr;

use boom::{Game,Engine};

struct OpenAlbion {
    started: Instant,
}

impl boom::Game for OpenAlbion {}

fn main() {
    let game = OpenAlbion {
        started: Instant::now(),
    };

    game.start(
        boom::Config {
            title: "Open Albion".to_string(),
            resolution: (1280.0, 720.0),
            fullscreen: false,
            vertex_shader_path: "./out/resources/shaders/vertex.spv".to_string(),
            fragment_shader_path: "./out/resources/shaders/fragment.spv".to_string(),
            systems: Vec::new(),
        }
    )
}