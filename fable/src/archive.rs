use std::io::{Read,Seek,SeekFrom,BufReader};
use std::iter::Take;

use crate::{Decode,Error};

pub trait Entry {
    fn length() -> u64;
    fn offset() -> u64;
    fn reader<Source: Read + Seek>(&mut self, source: &mut Source) -> Result<EntryReader<Source, Self>, Error> {
        EntryReader::new(source, &mut self)
    }
}

pub struct EntryReader<'a, Source, EntryItem> where
    Source: Read + Seek,
    EntryItem: Entry,
{
    pub source: Take<BufReader<&'a mut Source>>,
    pub entry: &'a mut EntryItem,
}

impl<'a, Source, EntryItem> EntryReader<'a, Source, EntryItem> where
    Source: Read + Seek,
    EntryItem: Entry,
{
    fn new(source: &mut Source, entry: &mut EntryItem) -> std::io::Result<Self> {
        let mut source = BufReader::new(&mut source);

        source.seek(SeekFrom::Start(entry.offset()))?;

        EntryReader {
            source: source.take(entry.length()),
            entry: entry,
        }
    }
}

impl<'a, Source, EntryItem> Read for EntryReader<'a, Source, EntryItem> where
    Source: Read + Seek,
    EntryItem: Entry,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.source.read(buf)
    }
}

impl<'a, Source, EntryItem> Seek for EntryReader<'a, Source, EntryItem> where
    Source: Read + Seek,
    EntryItem: Entry,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<usize> {
        self.source.seek(pos)
    }
}