// use ember_build::EmberBuild;

fn main() {
    let current_dir = std::env::current_dir().unwrap();

    // let config = EmberBuild {
    //     shaders: current_dir.join("shaders"),
    // };

    // ember_build::build(config);

    ember_build::build();
}