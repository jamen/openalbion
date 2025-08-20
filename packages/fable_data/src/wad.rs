use super::bytes::{put, put_bytes, take, take_bytes};
use derive_more::{Display, Error};
use fallible_iterator::FallibleIterator;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{self, Read, Seek, SeekFrom, Write},
    mem,
};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    // Not sure what this is, but its the same number as the entry count from what I've seen. This
    // may have another purpose, or it could be the real entry count and entry count field has
    // another purpose.
    pub entry_count_again: u32,
    pub entries_position: u32,
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadHeaderError {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    EntryCountRepeated,
    FirstEntryPosition,
}

impl WadHeader {
    pub const FILE_SIGNATURE: [u8; 4] = *b"BBBB";

    pub fn parse(inp: &mut &[u8]) -> Result<Self, WadHeaderError> {
        use WadHeaderError as E;

        let magic = take::<[u8; 4]>(inp).map_err(|_| E::Magic)?;
        let version = take::<[u32; 3]>(inp)
            .map_err(|_| E::Version)?
            .map(u32::to_le);
        let block_size = take::<u32>(inp).map_err(|_| E::BlockSize)?.to_le();
        let entries_count = take::<u32>(inp).map_err(|_| E::EntryCount)?.to_le();
        let entries_count_again = take::<u32>(inp).map_err(|_| E::EntryCountRepeated)?.to_le();
        let entries_position = take::<u32>(inp).map_err(|_| E::FirstEntryPosition)?.to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entry_count: entries_count,
            entry_count_again: entries_count_again,
            entries_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderError> {
        use WadHeaderError as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(|_| E::Version)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;
        put(out, &self.entry_count.to_le()).map_err(|_| E::EntryCount)?;
        put(out, &self.entry_count_again.to_le()).map_err(|_| E::EntryCountRepeated)?;
        put(out, &self.entries_position.to_le()).map_err(|_| E::FirstEntryPosition)?;

        Ok(())
    }

    pub const fn byte_size() -> usize {
        32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WadEntry<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub content_length: u32,
    pub content_position: u32,
    pub unknown_3: u32,
    pub path: Cow<'a, str>,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadEntryError {
    Unknown1,
    Id,
    Unknown2,
    ContentPosition,
    ContentLength,
    Unknown3,
    PathLen,
    Path,
    PathToString,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

impl<'a> WadEntry<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<WadEntry<'a>, WadEntryError> {
        use WadEntryError as E;

        let unknown_1 = take::<[u8; 16]>(inp).map_err(|_| E::Unknown1)?;
        let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
        let unknown_2 = take::<u32>(inp).map_err(|_| E::Unknown2)?.to_le();
        let data_length = take::<u32>(inp).map_err(|_| E::ContentLength)?.to_le();
        let data_position = take::<u32>(inp).map_err(|_| E::ContentPosition)?.to_le();
        let unknown_3 = take::<u32>(inp).map_err(|_| E::Unknown3)?.to_le();

        let path_len = usize::try_from(take::<u32>(inp).map_err(|_| E::PathLen)?.to_le())
            .map_err(|_| E::PathLen)?;

        let path = take_bytes(inp, path_len).map_err(|_| E::Path)?;
        let path = str::from_utf8(path).map_err(|_| E::PathToString)?;
        let path = Cow::from(path);

        let unknown_4 = take::<[u8; 16]>(inp).map_err(|_| E::Unknown4)?;

        let created = take::<[u32; 7]>(inp)
            .map_err(|_| E::Created)?
            .map(u32::to_le);
        let accessed = take::<[u32; 7]>(inp)
            .map_err(|_| E::Accessed)?
            .map(u32::to_le);
        let modified = take::<[u32; 5]>(inp)
            .map_err(|_| E::Modified)?
            .map(u32::to_le);

        Ok(WadEntry {
            unknown_1,
            id,
            unknown_2,
            content_length: data_length,
            content_position: data_position,
            unknown_3,
            path,
            unknown_4,
            created,
            accessed,
            modified,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadEntryError> {
        use WadEntryError as E;

        put(out, &self.unknown_1).map_err(|_| E::Unknown1)?;
        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.unknown_2.to_le()).map_err(|_| E::Unknown2)?;
        put(out, &self.content_length.to_le()).map_err(|_| E::ContentLength)?;
        put(out, &self.content_position.to_le()).map_err(|_| E::ContentPosition)?;
        put(out, &self.unknown_3.to_le()).map_err(|_| E::Unknown3)?;

        let path_len = u32::try_from(self.path.len()).map_err(|_| E::PathLen)?;

        put(out, &path_len.to_le()).map_err(|_| E::PathLen)?;

        put_bytes(out, &self.path.as_bytes()).map_err(|_| E::Path)?;

        put(out, &self.unknown_4).map_err(|_| E::Unknown4)?;
        put(out, &self.created.map(u32::to_le)).map_err(|_| E::Created)?;
        put(out, &self.accessed.map(u32::to_le)).map_err(|_| E::Accessed)?;
        put(out, &self.modified.map(u32::to_le)).map_err(|_| E::Modified)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        mem::size_of::<[u8; 16]>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + self.path.len()
            + mem::size_of::<[u8; 16]>()
            + mem::size_of::<[u32; 7]>()
            + mem::size_of::<[u32; 7]>()
            + mem::size_of::<[u32; 5]>()
    }

    pub fn into_owned(self) -> WadEntry<'static> {
        WadEntry {
            unknown_1: self.unknown_1,
            id: self.id,
            unknown_2: self.unknown_2,
            content_length: self.content_length,
            content_position: self.content_position,
            unknown_3: self.unknown_3,
            path: Cow::Owned(self.path.into_owned()),
            unknown_4: self.unknown_4,
            created: self.created,
            accessed: self.accessed,
            modified: self.modified,
        }
    }
}

pub struct WadReader<Source: Read + Seek> {
    source: Source,
}

impl<Source: Read + Seek> WadReader<Source> {
    pub fn new(source: Source) -> Self {
        Self { source }
    }

    pub fn read_header(&mut self) -> Result<WadHeader, WadReaderError> {
        use WadReaderError as E;

        let mut header_bytes = [0; WadHeader::byte_size()];

        self.source
            .seek(SeekFrom::Start(0))
            .map_err(E::SeekHeader)?;

        self.source
            .read_exact(&mut header_bytes)
            .map_err(E::ReadHeader)?;

        WadHeader::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)
    }

    /// Read the wad entry list.
    ///
    /// Returns an [fallible iterator](https://docs.rs/fallible-iterator) that parses entries
    /// one-by-one. See [`WadEntryReader`] for more details.
    pub fn read_entries<'a>(&mut self) -> Result<WadEntryReader<'a>, WadReaderError> {
        use WadReaderError as E;

        let header = self.read_header()?;

        let entries_position = u64::try_from(header.entries_position)
            .map_err(|_| E::ParseHeader(WadHeaderError::FirstEntryPosition))?;

        let entry_count = usize::try_from(header.entry_count)
            .map_err(|_| E::ParseHeader(WadHeaderError::EntryCount))?;

        let mut entries_bytes = Vec::new();

        self.source
            .seek(SeekFrom::Start(entries_position))
            .map_err(E::SeekEntries)?;

        self.source
            .read_to_end(&mut entries_bytes)
            .map_err(E::ReadEntries)?;

        Ok(WadEntryReader::new(entries_bytes.into(), entry_count))
    }

    /// Read the file content of an entry.
    ///
    /// *Important:* The entry must originate from the same wad file and it shouldn't be invalidated
    /// by writing to the wad file inbetween reading entries and reading file contents. Failing to
    /// do this can return corrupted data.
    ///
    /// Entries can be obtained with [`WadReader::read_entries`].
    pub fn read_content(&mut self, entry: &WadEntry) -> Result<Vec<u8>, WadReaderError> {
        use WadReaderError as E;

        let content_position = u64::try_from(entry.content_position)
            .map_err(|_| E::ParseEntry(WadEntryError::ContentPosition))?;

        let content_length = usize::try_from(entry.content_length)
            .map_err(|_| E::ParseEntry(WadEntryError::ContentLength))?;

        let mut content_bytes = vec![0; content_length];

        self.source
            .seek(SeekFrom::Start(content_position))
            .map_err(E::SeekContents)?;

        self.source
            .read_exact(&mut content_bytes)
            .map_err(E::ReadContents)?;

        Ok(content_bytes)
    }
}

#[derive(Debug, Display, Error)]
pub enum WadReaderError {
    ParseHeader(WadHeaderError),
    SeekHeader(io::Error),
    ReadHeader(io::Error),
    ParseEntry(WadEntryError),
    SeekEntries(io::Error),
    ReadEntries(io::Error),
    SeekContents(io::Error),
    ReadContents(io::Error),
}

pub struct WadEntryReader<'a> {
    bytes: Cow<'a, [u8]>,
    position: usize,
    entries_left: usize,
}

impl<'a> WadEntryReader<'a> {
    fn new(bytes: Cow<'a, [u8]>, entry_count: usize) -> Self {
        Self {
            bytes,
            position: 0,
            entries_left: entry_count,
        }
    }

    pub fn into_owned(self) -> WadEntryReader<'static> {
        WadEntryReader {
            bytes: Cow::Owned(self.bytes.into_owned()),
            position: self.position,
            entries_left: self.entries_left,
        }
    }

    pub fn into_iterator(self) -> fallible_iterator::Iterator<Self> {
        self.iterator()
    }
}

impl<'a> FallibleIterator for WadEntryReader<'a> {
    type Item = WadEntry<'static>;
    type Error = WadEntryError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.entries_left > 0 {
            let mut entry_bytes = &self.bytes[self.position..];
            let entry = WadEntry::parse(&mut entry_bytes)?;
            self.position += entry.byte_size();
            self.entries_left -= 1;
            Ok(Some(entry.into_owned()))
        } else {
            Ok(None)
        }
    }
}

pub struct WadWriter<Sink: Write + Seek> {
    sink: Sink,
}

impl<Sink: Write + Seek> WadWriter<Sink> {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }

    pub fn write_header<S: Write + Seek>(
        &mut self,
        header: &WadHeader,
    ) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut header_bytes = [0; WadHeader::byte_size()];

        header
            .serialize(&mut &mut header_bytes[..])
            .map_err(E::SerializeHeader)?;

        self.sink.seek(SeekFrom::Start(0)).map_err(E::SeekHeader)?;
        self.sink.write_all(&header_bytes).map_err(E::WriteHeader)?;

        Ok(())
    }

    pub fn write_content(
        &mut self,
        content: &[u8],
        block_size: usize,
    ) -> Result<usize, WadWriterError> {
        use WadWriterError as E;

        let padding_amount = (block_size - (content.len() % block_size)) % block_size;
        let padding = vec![0; padding_amount];

        self.sink.write_all(&content).map_err(E::WriteContent)?;
        self.sink.write_all(&padding).map_err(E::WriteContent)?;

        let written = content.len() + padding_amount;

        Ok(written)
    }

    pub fn write_entries(&mut self, entries: &[WadEntry]) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut entries_bytes = Vec::new();

        for entry in entries {
            let entries_bytes_end = entries_bytes.len();

            entries_bytes.resize(entries_bytes_end + entry.byte_size(), 0);

            let entry_bytes = &mut &mut entries_bytes[entries_bytes_end..];

            entry
                .serialize(&mut &mut entry_bytes[..])
                .map_err(E::SerializeEntry)?;
        }

        self.sink.seek(SeekFrom::End(0)).map_err(E::SeekEntries)?;
        self.sink
            .write_all(&entries_bytes)
            .map_err(E::WriteEntries)?;

        Ok(())
    }
}

#[derive(Error, Debug, Display)]
pub enum WadWriterError {
    SerializeHeader(WadHeaderError),
    SeekHeader(io::Error),
    WriteHeader(io::Error),
    SerializeEntry(WadEntryError),
    SeekEntries(io::Error),
    WriteEntries(io::Error),
    WriteContent(io::Error),
}
