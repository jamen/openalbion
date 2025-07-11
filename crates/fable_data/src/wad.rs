use super::bytes::{put, put_bytes, take, take_bytes};
use derive_more::{Display, Error};
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
    pub entries_count: u32,
    pub entries_count_again: u32,
    pub entries_position: u32,
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadHeaderFormatError {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    EntryCountRepeated,
    FirstEntryPosition,
}

#[derive(Debug, Display, Error)]
pub enum WadHeaderIoError {
    Format(WadHeaderFormatError),
    Seek(io::Error),
    Read(io::Error),
    Write(io::Error),
}

impl WadHeader {
    pub const FILE_SIGNATURE: [u8; 4] = *b"BBBB";

    pub fn parse(inp: &mut &[u8]) -> Result<Self, WadHeaderFormatError> {
        use WadHeaderFormatError as E;

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
            entries_count,
            entries_count_again,
            entries_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderFormatError> {
        use WadHeaderFormatError as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(|_| E::Version)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;
        put(out, &self.entries_count.to_le()).map_err(|_| E::EntryCount)?;
        put(out, &self.entries_count_again.to_le()).map_err(|_| E::EntryCountRepeated)?;
        put(out, &self.entries_position.to_le()).map_err(|_| E::FirstEntryPosition)?;

        Ok(())
    }

    pub fn read<S: Read + Seek>(mut source: S) -> Result<Self, WadHeaderIoError> {
        use WadHeaderIoError as E;

        let mut header_bytes = [0; Self::byte_size()];

        source.seek(SeekFrom::Start(0)).map_err(E::Seek)?;
        source.read_exact(&mut header_bytes).map_err(E::Read)?;

        Self::parse(&mut &header_bytes[..]).map_err(E::Format)
    }

    pub fn write<S: Write + Seek>(&self, mut sink: S) -> Result<(), WadHeaderIoError> {
        use WadHeaderIoError as E;

        let mut header_bytes = [0; Self::byte_size()];

        self.serialize(&mut &mut header_bytes[..])
            .map_err(E::Format);

        sink.seek(SeekFrom::Start(0)).map_err(E::Seek)?;
        sink.write_all(&header_bytes).map_err(E::Write)?;

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
    pub data_length: u32,
    pub data_position: u32,
    pub unknown_3: u32,
    pub path: Cow<'a, str>,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadEntryFormatError {
    Unknown1,
    Id,
    Unknown2,
    DataPosition,
    DataLength,
    Unknown3,
    PathLen,
    Path,
    PathToString,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

#[derive(Debug, Display, Error)]
pub enum WadEntryIoError {
    Format(WadEntryFormatError),
    Seek(io::Error),
    Read(io::Error),
    Write(io::Error),
}

impl<'a> WadEntry<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<WadEntry<'a>, WadEntryFormatError> {
        use WadEntryFormatError as E;

        let unknown_1 = take::<[u8; 16]>(inp).map_err(|_| E::Unknown1)?;
        let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
        let unknown_2 = take::<u32>(inp).map_err(|_| E::Unknown2)?.to_le();
        let data_length = take::<u32>(inp).map_err(|_| E::DataLength)?.to_le();
        let data_position = take::<u32>(inp).map_err(|_| E::DataPosition)?.to_le();
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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadEntryFormatError> {
        use WadEntryFormatError as E;

        put(out, &self.unknown_1).map_err(|_| E::Unknown1)?;
        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.unknown_2.to_le()).map_err(|_| E::Unknown2)?;
        put(out, &self.data_length.to_le()).map_err(|_| E::DataLength)?;
        put(out, &self.data_position.to_le()).map_err(|_| E::DataPosition)?;
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

    fn read_entry<S: Read + Seek>(mut source: S) -> Result<Self, WadEntryIoError> {
        todo!();
    }

    fn read_content<S: Read + Write>(&self, mut source: S) -> Result<Vec<u8>, WadEntryIoError> {
        todo!()
    }

    fn write_entry<S: Write + Seek>(&self, mut sink: S) -> Result<(), WadEntryIoError> {
        todo!()
    }

    fn write_content<S: Read + Write>(&self, mut source: S) -> Result<Vec<u8>, WadEntryIoError> {
        todo!()
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
}
