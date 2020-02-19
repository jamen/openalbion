use std::fs::File;
use std::io::{Read,Seek,SeekFrom};

use super::{WadEntry,WadReader};

impl<'a> WadReader<'a> {
    pub fn create(source: &'a mut File, entry: WadEntry) -> Self {
        WadReader {
            source: source.take(entry.length as u64),
            entry: entry,
            position: 0,
        }
    }
}

impl Read for WadReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let read_pos = self.entry.offset as u64 + self.position;

        if read_pos > self.entry.length as u64 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Read position outside entry bounds."))
        }

        self.source.get_mut().seek(SeekFrom::Start(read_pos))?;

        let read_count = self.source.read(buf)?;
        self.position += read_count as u64;

        Ok(read_count)
    }
}