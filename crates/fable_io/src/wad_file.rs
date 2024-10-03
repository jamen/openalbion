use std::{fs::File, io, path::Path};

pub struct WadFile {
    file: File,
}

pub enum WadFileError {
    Open(io::Error),
}

impl WadFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<WadFile, WadFileError> {
        use WadFileError::*;

        let file = File::open(path).map_err(Open)?;

        Ok(WadFile { file })
    }
}
