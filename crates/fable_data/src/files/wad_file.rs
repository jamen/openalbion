use std::{fs::File, io, path::Path};

use crate::format::wad::{WadEntryOwned, WadHeader};

pub struct WadFile {
    file: File,
    // header: WadHeader,
    // entries: Vec<WadEntryOwned>,
}

pub enum WadFileError {
    Open(io::Error),
}

impl WadFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, WadFileError> {
        use WadFileError::*;

        let file = File::open(path).map_err(Open)?;

        let header_bytes = vec![0; WadHeader::byte_size()];

        Ok(Self { file })
    }
}
