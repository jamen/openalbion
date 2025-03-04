use super::bytes::{put, put_bytes, take, take_bytes, TakeError, UnexpectedEnd};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Read, Seek, SeekFrom, Write},
    mem,
    num::TryFromIntError,
    str::Utf8Error,
};

pub struct WadReader<S: Read + Seek> {
    source: S,
    header: WadHeader,
    entries_bytes: Vec<u8>,
}

#[derive(Debug, Display, Error)]
pub enum WadReaderError {
    HeaderSeek(io::Error),
    HeaderRead(io::Error),
    HeaderParse(WadHeaderError<TakeError>),
    EntriesSeek(io::Error),
    EntriesRead(io::Error),
    DataSeek(io::Error),
    DataRead(io::Error),
}

impl<S: Read + Seek> WadReader<S> {
    pub fn new(mut source: S) -> Result<Self, WadReaderError> {
        use WadReaderError::*;

        let mut header_bytes = vec![0; WadHeader::BYTE_SIZE];

        source.seek(SeekFrom::Start(0)).map_err(HeaderSeek)?;
        source.read_exact(&mut header_bytes).map_err(HeaderRead)?;

        let header = WadHeader::parse(&mut &header_bytes[..]).map_err(HeaderParse)?;

        // Uninitialized until we read the entries.
        let entries_bytes = Vec::new();

        Ok(Self {
            source,
            header,
            entries_bytes,
        })
    }

    pub fn header(&self) -> &WadHeader {
        &self.header
    }

    pub fn entries(&mut self) -> Result<WadEntryIter, WadReaderError> {
        use WadReaderError::*;

        let header = self.header();
        let entries_position = header.entries_position;
        let entries_count = header.entries_count;

        self.entries_bytes = Vec::with_capacity(8192);

        self.source
            .seek(SeekFrom::Start(entries_position as u64))
            .map_err(EntriesSeek)?;

        self.source
            .read_to_end(&mut self.entries_bytes)
            .map_err(EntriesRead)?;

        let iter = WadEntryIter::new(&self.entries_bytes[..], entries_count as usize);

        Ok(iter)
    }

    pub fn data(&mut self, entry: &WadEntry<'_>) -> Result<Vec<u8>, WadReaderError> {
        use WadReaderError::*;

        let mut data = vec![0; entry.data_length as usize];

        self.source
            .seek(SeekFrom::Start(entry.data_position as u64))
            .map_err(DataSeek)?;

        self.source.read_exact(&mut data).map_err(DataRead)?;

        Ok(data)
    }
}

pub struct WadWriter<S: Read + Write + Seek> {
    reader: WadReader<S>,
}

pub enum WadWriterError {
    Reader(WadReaderError),

    SeekHeader(io::Error),
}

impl<S: Read + Write + Seek> WadWriter<S> {
    pub fn new(source: S, header: WadHeader) -> Result<Self, WadWriterError> {
        use WadWriterError::*;

        let reader = WadReader::new(source).map_err(Reader)?;

        Ok(Self { reader })
    }

    fn source(&mut self) -> &mut S {
        &mut self.reader.source
    }

    pub fn pop(&mut self) -> Result<(), WadWriterError> {
        self.pop_many(1)
    }

    pub fn pop_many(&mut self, count: usize) -> Result<(), WadWriterError> {
        todo!()
    }

    pub fn push(&mut self, item: WadItem) -> Result<(), WadWriterError> {
        self.push_many([item])
    }

    pub fn push_many<'a, 'b, T>(&mut self, items: T) -> Result<(), WadWriterError>
    where
        T: IntoIterator<Item = WadItem<'a, 'b>>,
    {
        todo!()
    }

    pub fn compact(&mut self) {
        todo!()
    }
}

// Based on WadEntry, the differences being this
// - Includes the data for the entry
// - Excludes unknown fields
// - Excludes fields determined at insertion time (e.g. data_position and id)
struct WadItem<'a, 'b> {
    path: &'a str,
    created: [u32; 7],
    accessed: [u32; 7],
    modified: [u32; 5],
    data: &'b [u8],
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entries_count: u32,
    pub entries_count_again: u32,
    pub entries_position: u32,
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadHeaderError<E> {
    Magic(E),
    Version(E),
    BlockSize(E),
    EntryCount(E),
    EntryCountRepeated(E),
    FirstEntryPosition(E),
}

impl WadHeader {
    pub const MAGIC: [u8; 4] = *b"BBBB";
    pub const BYTE_SIZE: usize = 32;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, WadHeaderError<TakeError>> {
        use WadHeaderError::*;

        let magic = take::<[u8; 4]>(inp).map_err(Magic)?;
        let version = take::<[u32; 3]>(inp).map_err(Version)?.map(u32::to_le);
        let block_size = take::<u32>(inp).map_err(BlockSize)?.to_le();
        let entries_count = take::<u32>(inp).map_err(EntryCount)?.to_le();
        let entries_count_again = take::<u32>(inp).map_err(EntryCountRepeated)?.to_le();
        let entries_position = take::<u32>(inp).map_err(FirstEntryPosition)?.to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entries_count,
            entries_count_again,
            entries_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderError<UnexpectedEnd>> {
        use WadHeaderError::*;

        put(out, &self.magic).map_err(Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(Version)?;
        put(out, &self.block_size.to_le()).map_err(BlockSize)?;
        put(out, &self.entries_count.to_le()).map_err(EntryCount)?;
        put(out, &self.entries_count_again.to_le()).map_err(EntryCountRepeated)?;
        put(out, &self.entries_position.to_le()).map_err(FirstEntryPosition)?;

        Ok(())
    }
}

// impl Default for WadHeader {
//     fn default() -> Self {
//         WadHeader {
//             magic: b'BBBB',
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WadEntry<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub data_length: u32,
    pub data_position: u32,
    pub unknown_3: u32,
    pub path: &'a str,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadEntryError<E> {
    Unknown1(E),
    Id(E),
    Unknown2(E),
    DataPosition(E),
    DataLength(E),
    Unknown3(E),
    PathLen(E),
    PathLenInt(TryFromIntError),
    Path(E),
    PathString(Utf8Error),
    Unknown4(E),
    Created(E),
    Accessed(E),
    Modified(E),
}

impl<'a> WadEntry<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<WadEntry<'a>, WadEntryError<TakeError>> {
        use WadEntryError::*;

        let unknown_1 = take::<[u8; 16]>(inp).map_err(Unknown1)?;
        let id = take::<u32>(inp).map_err(Id)?.to_le();
        let unknown_2 = take::<u32>(inp).map_err(Unknown2)?.to_le();
        let data_length = take::<u32>(inp).map_err(DataLength)?.to_le();
        let data_position = take::<u32>(inp).map_err(DataPosition)?.to_le();
        let unknown_3 = take::<u32>(inp).map_err(Unknown3)?.to_le();

        let path_len =
            usize::try_from(take::<u32>(inp).map_err(PathLen)?.to_le()).map_err(PathLenInt)?;
        let path = take_bytes(inp, path_len).map_err(|e| Path(TakeError::UnexpectedEnd(e)))?;
        let path = std::str::from_utf8(path).map_err(PathString)?;

        let unknown_4 = take::<[u8; 16]>(inp).map_err(Unknown4)?;

        let created = take::<[u32; 7]>(inp).map_err(Created)?.map(u32::to_le);
        let accessed = take::<[u32; 7]>(inp).map_err(Accessed)?.map(u32::to_le);
        let modified = take::<[u32; 5]>(inp).map_err(Modified)?.map(u32::to_le);

        Ok(WadEntry {
            unknown_1,
            id,
            unknown_2,
            data_length,
            data_position,
            unknown_3,
            path,
            unknown_4,
            created,
            accessed,
            modified,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadEntryError<UnexpectedEnd>> {
        use WadEntryError::*;

        put(out, &self.unknown_1).map_err(Unknown1)?;
        put(out, &self.id.to_le()).map_err(Id)?;
        put(out, &self.unknown_2.to_le()).map_err(Unknown2)?;
        put(out, &self.data_length.to_le()).map_err(DataLength)?;
        put(out, &self.data_position.to_le()).map_err(DataPosition)?;
        put(out, &self.unknown_3.to_le()).map_err(Unknown3)?;

        let path_len = u32::try_from(self.path.len()).map_err(PathLenInt)?;

        put(out, &path_len.to_le()).map_err(PathLen)?;

        put_bytes(out, &self.path.as_bytes()).map_err(Path)?;

        put(out, &self.unknown_4).map_err(Unknown4)?;
        put(out, &self.created.map(u32::to_le)).map_err(Created)?;
        put(out, &self.accessed.map(u32::to_le)).map_err(Accessed)?;
        put(out, &self.modified.map(u32::to_le)).map_err(Modified)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        // Unknown 1
        mem::size_of::<[u8; 16]>() +
        // Id
        mem::size_of::<u32>() +
        // Unknown 2
        mem::size_of::<u32>() +
        // Offset
        mem::size_of::<u32>() +
        // Length
        mem::size_of::<u32>() +
        // Unknown 3
        mem::size_of::<u32>() +
        // Path len
        mem::size_of::<u32>() +
        // Path
        self.path.len() +
        // Unknown 4
        mem::size_of::<[u8; 16]>() +
        // Created
        mem::size_of::<[u32; 7]>() +
        // Accessed
        mem::size_of::<[u32; 7]>() +
        // Modified
        mem::size_of::<[u32; 5]>()
    }

    pub fn to_owned(&self) -> WadEntryOwned {
        WadEntryOwned::from_ref(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WadEntryOwned {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub length: u32,
    pub offset: u32,
    pub unknown_3: u32,
    pub path: String,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

impl WadEntryOwned {
    pub fn from_ref(entry: &WadEntry<'_>) -> Self {
        WadEntryOwned {
            unknown_1: entry.unknown_1,
            id: entry.id,
            unknown_2: entry.unknown_2,
            length: entry.data_length,
            offset: entry.data_position,
            unknown_3: entry.unknown_3,
            path: entry.path.to_owned(),
            unknown_4: entry.unknown_4,
            created: entry.created,
            accessed: entry.accessed,
            modified: entry.modified,
        }
    }

    pub fn to_ref(&self) -> WadEntry<'_> {
        WadEntry {
            unknown_1: self.unknown_1,
            id: self.id,
            unknown_2: self.unknown_2,
            data_length: self.length,
            data_position: self.offset,
            unknown_3: self.unknown_3,
            path: self.path.as_str(),
            unknown_4: self.unknown_4,
            created: self.created,
            accessed: self.accessed,
            modified: self.modified,
        }
    }
}

#[derive(Debug)]
pub struct WadEntryIter<'a> {
    input: &'a [u8],
    entry_count: usize,
    current_entry: usize,
}

impl<'a> WadEntryIter<'a> {
    pub fn new(input: &'a [u8], entry_count: usize) -> Self {
        Self {
            input,
            entry_count,
            current_entry: 0,
        }
    }
}

impl<'a> Iterator for WadEntryIter<'a> {
    type Item = Result<WadEntry<'a>, WadEntryError<TakeError>>;

    fn next(&mut self) -> Option<Result<WadEntry<'a>, WadEntryError<TakeError>>> {
        if self.current_entry < self.entry_count {
            match WadEntry::parse(&mut self.input) {
                Ok(x) => {
                    self.current_entry += 1;
                    Some(Ok(x))
                }
                Err(x) => {
                    self.current_entry = usize::MAX;
                    Some(Err(x))
                }
            }
        } else {
            None
        }
    }
}
