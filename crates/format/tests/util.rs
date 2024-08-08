use std::env::{self, VarError};
use std::path::PathBuf;

pub fn fable_path() -> Result<PathBuf, VarError> {
    let fable_data = env::var("FABLE_PATH")?;
    Ok(PathBuf::from(fable_data))
}
