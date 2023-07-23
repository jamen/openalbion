use derive_more::Display;
use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    // Logger with all logs enabled
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut args = env::args().skip(1);

    // Establish base directory
    let fable_path: PathBuf = args
        .next()
        .expect("A path to Fable's directory must be given.")
        .into();

    let data_path =
        find_data_path(&fable_path, ["Data", "data"]).expect("Data directory was not found");

    let fool_path = fable_path.join("Fool");
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display)]
struct NotFound;

fn find_data_path<B: AsRef<Path>, I: AsRef<Path>, V: AsRef<[I]>>(
    base: B,
    variants: V,
) -> Result<PathBuf, NotFound> {
    let base_path = base.as_ref();

    variants
        .as_ref()
        .iter()
        .map(|candidate| base_path.join(candidate))
        .find(|x| x.exists())
        .ok_or(NotFound)
}
