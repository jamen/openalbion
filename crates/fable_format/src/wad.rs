use crate::common::bytes::{put, put_bytes, take, take_bytes, TakeError, UnexpectedEnd};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{mem, num::TryFromIntError, str::Utf8Error};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeated: u32,
    pub first_entry_position: u32,
}

#[derive(Copy, Clone, Debug, Display)]
pub enum WadHeaderError<E> {
    Magic(E),
    Version(E),
    BlockSize(E),
    EntryCount(E),
    EntryCountRepeated(E),
    FirstEntryPosition(E),
}

impl WadHeader {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, WadHeaderError<TakeError>> {
        use WadHeaderError::*;

        let magic = take::<[u8; 4]>(inp).map_err(Magic)?;
        let version = take::<[u32; 3]>(inp).map_err(Version)?.map(u32::to_le);
        let block_size = take::<u32>(inp).map_err(BlockSize)?.to_le();
        let entry_count = take::<u32>(inp).map_err(EntryCount)?.to_le();
        let entry_count_repeated = take::<u32>(inp).map_err(EntryCountRepeated)?.to_le();
        let first_entry_position = take::<u32>(inp).map_err(FirstEntryPosition)?.to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entry_count,
            entry_count_repeated,
            first_entry_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderError<UnexpectedEnd>> {
        use WadHeaderError::*;

        put(out, &self.magic).map_err(Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(Version)?;
        put(out, &self.block_size.to_le()).map_err(BlockSize)?;
        put(out, &self.entry_count.to_le()).map_err(EntryCount)?;
        put(out, &self.entry_count_repeated.to_le()).map_err(EntryCountRepeated)?;
        put(out, &self.first_entry_position.to_le()).map_err(FirstEntryPosition)?;

        Ok(())
    }

    /// Computes the minimum size of an output buffer needed for serialization.
    pub const fn byte_size() -> usize {
        // Magic
        mem::size_of::<[u8; 4]>() +
        // Version
        mem::size_of::<[u32; 3]>() +
        // Block size
        mem::size_of::<u32>() +
        // Entry count
        mem::size_of::<u32>() +
        // Entry count repeated
        mem::size_of::<u32>() +
        // First entry position
        mem::size_of::<u32>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WadEntry<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub length: u32,
    pub offset: u32,
    pub unknown_3: u32,
    pub path: &'a str,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
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

#[derive(Copy, Clone, Debug)]
pub enum WadEntryError<E> {
    Unknown1(E),
    Id(E),
    Unknown2(E),
    Offset(E),
    Length(E),
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
        let length = take::<u32>(inp).map_err(Length)?.to_le();
        let offset = take::<u32>(inp).map_err(Offset)?.to_le();
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
            length,
            offset,
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
        put(out, &self.length.to_le()).map_err(Length)?;
        put(out, &self.offset.to_le()).map_err(Offset)?;
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
        WadEntryOwned {
            unknown_1: self.unknown_1,
            id: self.id,
            unknown_2: self.unknown_2,
            length: self.length,
            offset: self.offset,
            unknown_3: self.unknown_3,
            path: self.path.to_owned(),
            unknown_4: self.unknown_4,
            created: self.created,
            accessed: self.accessed,
            modified: self.modified,
        }
    }
}
