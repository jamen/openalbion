use std::env;

fn main() {
    let mut args = env::args();

    let fable_path = args
        .next()
        .expect("Must provide a path to Fable's directory");

    println!("{:?}", fable_path);
}
