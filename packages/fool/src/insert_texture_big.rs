use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug, Clone)]
pub struct InsertTextureBigArgs {
    /// Input big file to be extracted
    big_file: PathBuf,
}

pub fn handler(_fable_data: &Path, args: InsertTextureBigArgs) -> anyhow::Result<()> {
    todo!()
}
