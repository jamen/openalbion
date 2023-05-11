use common::take;

use crate::{WadHeader, WadHeaderPart};

impl WadHeader {
    pub fn parse(src: &[u8]) -> Result<Self, (WadHeaderPart, &[u8])> {
        let &magic = take::<[u8; 4], _>(src, WadHeaderPart::Magic)?;
        let version = take::<[u32; 3], _>(src, WadHeaderPart::Version)?.map(u32::to_le);
        let block_size = take::<u32, _>(src, WadHeaderPart::BlockSize)?.to_le();
        let entry_count = take::<u32, _>(src, WadHeaderPart::EntryCount)?.to_le();
        let repeated_entry_count = take::<u32, _>(src, WadHeaderPart::RepeatedEntryCount)?.to_le();
        let first_entry_offset = take::<u32, _>(src, WadHeaderPart::FirstEntryOffset)?.to_le();

        Ok(WadHeader {
            magic,
            version,
            block_size,
            entry_count,
            repeated_entry_count,
            first_entry_offset,
        })
    }
}

use crate::{WadEntry, WadEntryPart};

impl WadEntry<'_> {
    pub fn parse<'a>(mut src: &'a [u8]) -> Result<WadEntry<'a>, (WadEntryPart, &'a [u8])> {
        let &unknown_1 = take::<[u8; 16], _>(src, WadEntryPart::Unknown1)?;
        let id = take::<u32, _>(src, WadEntryPart::Id)?.to_le();
        let unknown_2 = take::<u32, _>(src, WadEntryPart::Unknown2)?.to_le();
        let offset = take::<u32, _>(src, WadEntryPart::Offset)?.to_le();
        let length = take::<u32, _>(src, WadEntryPart::Length)?.to_le();
        let unknown_3 = take::<u32, _>(src, WadEntryPart::Unknown3)?.to_le();

        let path_len = take::<u32, _>(src, WadEntryPart::PathLen)?.to_le() as usize;
        let path = src
            .take(..path_len)
            .ok_or_else(|| (WadEntryPart::Path, src))?;

        let &unknown_4 = take::<[u8; 16], _>(src, WadEntryPart::Unknown4)?;
        let created = take::<[u32; 7], _>(src, WadEntryPart::Created)?.map(u32::to_le);
        let accessed = take::<[u32; 7], _>(src, WadEntryPart::Accessed)?.map(u32::to_le);
        let modified = take::<[u32; 5], _>(src, WadEntryPart::Modified)?.map(u32::to_le);

        Ok(WadEntry {
            unknown_1,
            id,
            unknown_2,
            offset,
            length,
            unknown_3,
            path,
            unknown_4,
            created,
            accessed,
            modified,
        })
    }
}
