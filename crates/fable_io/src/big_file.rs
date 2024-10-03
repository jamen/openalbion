use std::{fs::File, io, path::Path};

pub struct BigFile {
    file: File,
}

pub enum BigFileError {
    Open(io::Error),
}

impl BigFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<BigFile, BigFileError> {
        use BigFileError::*;

        let file = File::open(path).map_err(Open)?;

        Ok(BigFile { file })
    }
}
