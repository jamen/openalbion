use super::bytes::{put, put_bytes, take, take_bytes};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, mem};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct WadHeader {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_again: u32,
    pub entries_position: u32,
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadHeaderSection {
    Magic,
    Version,
    BlockSize,
    EntryCount,
    EntryCountRepeated,
    FirstEntryPosition,
}

impl WadHeader {
    pub const BYTE_SIZE: usize = 32;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, WadHeaderSection> {
        use WadHeaderSection as E;

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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadHeaderSection> {
        use WadHeaderSection as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(|_| E::Version)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;
        put(out, &self.entry_count.to_le()).map_err(|_| E::EntryCount)?;
        put(out, &self.entry_count_again.to_le()).map_err(|_| E::EntryCountRepeated)?;
        put(out, &self.entries_position.to_le()).map_err(|_| E::FirstEntryPosition)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WadAssetInfo<'a> {
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

pub type WadAssetInfoOwned = WadAssetInfo<'static>;

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum WadAssetInfoSection {
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

impl<'a> WadAssetInfo<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<WadAssetInfo<'a>, WadAssetInfoSection> {
        use WadAssetInfoSection as E;

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

        Ok(WadAssetInfo {
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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), WadAssetInfoSection> {
        use WadAssetInfoSection as E;

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

    pub fn into_owned(self) -> WadAssetInfoOwned {
        WadAssetInfoOwned {
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
