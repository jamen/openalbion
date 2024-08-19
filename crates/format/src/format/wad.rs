use crate::util::binary::{
    BinaryParser, BinaryParserError, BinarySerializer, BinarySerializerError,
};
use serde::{Deserialize, Serialize};
use std::mem;

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeated: u32,
    pub first_entry_position: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum WadHeaderPart {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    EntryCountRepeated,
    FirstEntryPosition,
}

impl WadHeader {
    pub fn parse(p: &mut BinaryParser) -> Result<Self, BinaryParserError<WadHeaderPart>> {
        use WadHeaderPart::*;

        let magic = p.take::<[u8; 4], _>(Magic)?;
        let version = p.take::<[u32; 3], _>(Version)?.map(u32::to_le);
        let block_size = p.take::<u32, _>(BlockSize)?.to_le();
        let entry_count = p.take::<u32, _>(EntryCount)?.to_le();
        let entry_count_repeated = p.take::<u32, _>(EntryCountRepeated)?.to_le();
        let first_entry_position = p.take::<u32, _>(FirstEntryPosition)?.to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entry_count,
            entry_count_repeated,
            first_entry_position,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BinaryParserError<WadHeaderPart>> {
        Self::parse(&mut BinaryParser::new(bytes))
    }

    pub fn serialize(
        &self,
        s: &mut BinarySerializer,
    ) -> Result<(), BinarySerializerError<WadHeaderPart>> {
        use WadHeaderPart::*;

        s.put(&self.magic, Magic)?;
        s.put(&self.version.map(u32::to_le), Version)?;
        s.put(&self.block_size.to_le(), BlockSize)?;
        s.put(&self.entry_count.to_le(), EntryCount)?;
        s.put(&self.entry_count_repeated.to_le(), EntryCountRepeated)?;
        s.put(&self.first_entry_position.to_le(), FirstEntryPosition)?;

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

    pub fn to_bytes(&self, out: &mut [u8]) -> Result<(), BinarySerializerError<WadHeaderPart>> {
        self.serialize(&mut BinarySerializer::new(out))
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
pub enum WadEntryPart {
    Unknown1,
    Id,
    Unknown2,
    Offset,
    Length,
    Unknown3,
    PathLen,
    Path,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

impl<'a> WadEntry<'a> {
    pub fn parse(
        p: &mut BinaryParser<'a>,
    ) -> Result<WadEntry<'a>, BinaryParserError<WadEntryPart>> {
        use WadEntryPart::*;

        let unknown_1 = p.take::<[u8; 16], _>(Unknown1)?;
        let id = p.take::<u32, _>(Id)?.to_le();
        let unknown_2 = p.take::<u32, _>(Unknown2)?.to_le();
        let length = p.take::<u32, _>(Length)?.to_le();
        let offset = p.take::<u32, _>(Offset)?.to_le();
        let unknown_3 = p.take::<u32, _>(Unknown3)?.to_le();

        let path_len = p.take::<u32, _>(PathLen)?.to_le() as usize;
        let path = p.take_bytes(path_len, Path)?;
        let path = std::str::from_utf8(path).map_err(|_| p.new_error(Path, None))?;

        let unknown_4 = p.take::<[u8; 16], _>(Unknown4)?;

        let created = p.take::<[u32; 7], _>(Created)?.map(u32::to_le);
        let accessed = p.take::<[u32; 7], _>(Accessed)?.map(u32::to_le);
        let modified = p.take::<[u32; 5], _>(Modified)?.map(u32::to_le);

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

    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, BinaryParserError<WadEntryPart>> {
        Self::parse(&mut BinaryParser::new(bytes))
    }

    pub fn serialize(
        &self,
        s: &mut BinarySerializer,
    ) -> Result<(), BinarySerializerError<WadEntryPart>> {
        use WadEntryPart::*;

        s.put(&self.unknown_1, Unknown1)?;
        s.put(&self.id.to_le(), Id)?;
        s.put(&self.unknown_2.to_le(), Unknown2)?;
        s.put(&self.length.to_le(), Length)?;
        s.put(&self.offset.to_le(), Offset)?;
        s.put(&self.unknown_3.to_le(), Unknown3)?;

        let path_len = u32::try_from(self.path.len()).map_err(|_| s.new_error(PathLen))?;

        s.put(&path_len.to_le(), PathLen)?;

        s.put_bytes(&self.path.as_bytes(), Path)?;

        s.put(&self.unknown_4, Unknown4)?;
        s.put(&self.created.map(u32::to_le), Created)?;
        s.put(&self.accessed.map(u32::to_le), Accessed)?;
        s.put(&self.modified.map(u32::to_le), Modified)?;

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

    pub fn to_bytes(&self, out: &mut [u8]) -> Result<(), BinarySerializerError<WadEntryPart>> {
        self.serialize(&mut BinarySerializer::new(out))
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
