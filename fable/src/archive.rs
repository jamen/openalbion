use std::io::{self,Read,Seek,SeekFrom,BufReader};
use std::convert::TryFrom;

pub struct ArchiveReader<Source, Entry> where
    Source: Read + Seek,
    Entry: ArchiveEntry,
{
    pub source: BufReader<Source>,
    pub entry: Entry,
}

pub trait ArchiveEntry {
    fn offset(&self) -> u64;
    fn length(&self) -> u64;
}

impl<Source, Entry> ArchiveReader<Source, Entry> where
    Source: Read + Seek,
    Entry: ArchiveEntry,
{
    pub fn create(source: Source, entry: Entry) -> Self {
        let source = BufReader::new(source);

        source.seek(SeekFrom::Start(entry.offset()));

        ArchiveReader {
            source: source,
            entry: entry,
        }
    }
}

impl<Source, Entry> Seek for ArchiveReader<Source, Entry> where
    Source: Read + Seek,
    Entry: ArchiveEntry,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        self.source.seek(
            match pos {
                SeekFrom::Start(pos) => {
                    if pos < self.entry.offset() || pos >= self.entry.length() {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Seeked outside entry bounds."))
                    }
                    SeekFrom::Start(pos)
                },
                SeekFrom::End(x) => {
                    let pos = x - (self.entry.offset() + self.entry.length()) as i64;
                    if pos < self.entry.offset() as i64 || pos >= self.entry.length() as i64 {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Seeked outside entry bounds."))
                    }
                    SeekFrom::End(pos)
                },
                SeekFrom::Current(x) => {
                    let current = self.source.seek(SeekFrom::Current(0))?;
                    let pos = i64::try_from(current).unwrap() + x;
                    if pos < self.entry.offset() as i64 || pos >= self.entry.length() as i64 {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Seeked outside entry bounds."))
                    }
                    SeekFrom::Current(pos)
                },
            }
        )
    }
}

impl<Source, Entry> Read for ArchiveReader<Source, Entry> where
    Source: Read + Seek,
    Entry: ArchiveEntry,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let source_position = self.source.seek(SeekFrom::Current(0))?;
        let entry_position = source_position - self.entry.offset();
        self.take(self.entry.length() - entry_position).read(&mut buf)
    }
}