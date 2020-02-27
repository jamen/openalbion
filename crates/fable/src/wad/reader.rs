use std::fs::File;
use std::io::{self,Read,Seek,SeekFrom};
use std::convert::TryInto;

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

impl Seek for WadReader<'_> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        self.position = match pos {
            SeekFrom::Start(pos) => {
                if pos > self.entry.length.into() {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Position out of bounds."))
                } else {
                    pos
                }
            },
            SeekFrom::End(x) => {
                let pos = (self.entry.length - 1) as i64 - x;

                if pos < 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Position out of bounds."))
                } else {
                    pos.try_into().unwrap()
                }
            },
            SeekFrom::Current(x) => {
                let pos = if x > 0 {
                    self.position - (-x) as u64
                } else {
                    self.position + x as u64
                };

                if pos >= self.position {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Position out of bounds."))
                }

                pos
            },
        };

        Ok(self.position)
    }
}

impl Read for WadReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let read_pos = self.entry.offset as u64 + self.position;

        if read_pos > self.entry.length as u64 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Read position outside entry bounds."))
        }

        self.source.get_mut().seek(SeekFrom::Start(read_pos))?;

        let read_count = self.source.read(buf)?;

        self.seek(SeekFrom::Start(self.position + read_count as u64))?;

        Ok(read_count)
    }
}