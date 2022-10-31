use common::{bytemuck::PodCastError, ReadPod};

use crate::WadHeader;

pub struct WadHeaderParseError;

impl From<PodCastError> for WadHeaderParseError {
    fn from(_x: PodCastError) -> Self {
        Self
    }
}

impl WadHeader {
    pub fn parse(bytes: &mut &[u8]) -> Result<Self, WadHeaderParseError> {
        let &magic = bytes.read_pod::<[u8; 4]>()?;
        let version = bytes.read_pod::<[u32; 3]>()?.map(u32::to_le);
        let block_size = bytes.read_pod::<u32>()?.to_le();
        let entry_count = bytes.read_pod::<u32>()?.to_le();
        let repeated_entry_count = bytes.read_pod::<u32>()?.to_le();
        let first_entry_offset = bytes.read_pod::<u32>()?.to_le();

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

use crate::WadEntry;

pub struct WadEntryParseError;

impl From<PodCastError> for WadEntryParseError {
    fn from(_x: PodCastError) -> Self {
        Self
    }
}

impl WadEntry<'_> {
    pub fn parse<'a>(bytes: &mut &'a [u8]) -> Result<WadEntry<'a>, WadEntryParseError> {
        let &unknown_1 = bytes.read_pod::<[u8; 16]>()?;
        let id = bytes.read_pod::<u32>()?.to_le();
        let unknown_2 = bytes.read_pod::<u32>()?.to_le();
        let offset = bytes.read_pod::<u32>()?.to_le();
        let length = bytes.read_pod::<u32>()?.to_le();
        let unknown_3 = bytes.read_pod::<u32>()?.to_le();

        let path_len = bytes.read_pod::<u32>()?.to_le() as usize;
        let path = &bytes.get(..path_len).ok_or(WadEntryParseError)?;
        *bytes = &bytes[path_len..];

        let &unknown_4 = bytes.read_pod::<[u8; 16]>()?;

        let created = bytes.read_pod::<[u32; 7]>()?.map(u32::to_le);
        let accessed = bytes.read_pod::<[u32; 7]>()?.map(u32::to_le);
        let modified = bytes.read_pod::<[u32; 5]>()?.map(u32::to_le);

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
