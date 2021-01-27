mod app;
mod renderer;

pub use app::*;

// use std::env;
// use std::path::PathBuf;

fn main() {
    let app = App::init();
    app.run()
}