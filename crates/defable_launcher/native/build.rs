extern crate neon_build;
extern crate cc;

fn main() {
    // must be called in build.rs
    neon_build::setup();

    // See https://github.com/jrd-rocks/neon/blob/electron_delay_hook/README%20ELECTRON%204.md
    // cc::Build::new()
    //         .cpp(true)
    //         .static_crt(true)
    //         .file("src/win_delay_load_hook.cc")
    //         .compile("hook");

    // add project-specific build logic here...
}
