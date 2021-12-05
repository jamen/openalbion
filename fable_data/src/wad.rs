use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use crate::Bytes;

#[derive(Debug)]
pub struct WadHeader {
    pub magic_number: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeat: u32,
    pub entries_start: u32,
}

impl WadHeader {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        let magic_number = data.advance(4)?.try_into().ok()?;
        let version = [
            data.parse_u32_le()?,
            data.parse_u32_le()?,
            data.parse_u32_le()?,
        ];
        let block_size = data.parse_u32_le()?;
        let entry_count = data.parse_u32_le()?;
        let entry_count_repeat = data.parse_u32_le()?;
        let entries_start = data.parse_u32_le()?;

        Some(WadHeader {
            magic_number,
            version,
            block_size,
            entry_count,
            entry_count_repeat,
            entries_start,
        })
    }
}

#[derive(Debug)]
pub struct WadEntry {
    pub path: String,
    pub unknown_1: Vec<u8>,
    pub unknown_2: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub unknown_3: u32,
    pub unknown_4: Vec<u8>,
    pub created: WadTimestamp,
    pub accessed: WadTimestamp,
    pub modified: WadTimestampShort,
}

impl WadEntry {
    pub fn parse_all(data: &mut &[u8], header: &WadHeader) -> Option<Vec<Self>> {
        let mut entries = Vec::new();

        while entries.len() < header.entry_count as usize {
            let unknown_1 = data.advance(16)?.to_owned();
            let id = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let data_size = data.parse_u32_le()?;
            let data_start = data.parse_u32_le()?;
            let unknown_3 = data.parse_u32_le()?;
            let path = data.parse_str_with_u32_le_prefix()?.to_owned();
            let unknown_4 = data.advance(16)?.to_owned();
            let created = WadTimestamp::parse(data)?;
            let accessed = WadTimestamp::parse(data)?;
            let modified = WadTimestampShort::parse(data)?;

            if id != entries.len() as u32 {
                return None;
            }

            entries.push(WadEntry {
                path,
                unknown_1,
                unknown_2,
                data_size,
                data_start,
                unknown_3,
                unknown_4,
                created,
                accessed,
                modified,
            });
        }

        Some(entries)
    }
}

#[derive(Debug)]
pub struct WadTimestamp {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millisecond: u32,
}

impl WadTimestamp {
    fn parse(data: &mut &[u8]) -> Option<WadTimestamp> {
        Some(WadTimestamp {
            year: data.parse_u32_le()?,
            month: data.parse_u32_le()?,
            day: data.parse_u32_le()?,
            hour: data.parse_u32_le()?,
            minute: data.parse_u32_le()?,
            second: data.parse_u32_le()?,
            millisecond: data.parse_u32_le()?,
        })
    }
}

#[derive(Debug)]
pub struct WadTimestampShort {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

impl WadTimestampShort {
    fn parse(data: &mut &[u8]) -> Option<WadTimestampShort> {
        Some(WadTimestampShort {
            year: data.parse_u32_le()?,
            month: data.parse_u32_le()?,
            day: data.parse_u32_le()?,
            hour: data.parse_u32_le()?,
            minute: data.parse_u32_le()?,
        })
    }
}
