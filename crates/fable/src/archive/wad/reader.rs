use std::fs::File;
use std::io::prelude::*;
use std::io::{Read,SeekFrom};

use super::{WadEntry,WadReader};

impl<'a> WadReader<'a> {
    pub fn create(source: &'a mut File, entry: WadEntry) -> Self {
        WadReader {
            source: source,
            entry: entry,
        }
    }
}

impl Read for WadReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.source.seek(SeekFrom::Start(self.entry.offset as u64))?;
        self.source.take(self.entry.length as u64).read(buf)
    }
}