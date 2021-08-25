use std::collections::HashMap;
use std::io::{Error as IoError, Read, Seek, SeekFrom};

use crate::Bytes;

#[derive(Debug)]
pub struct Wad<T: Read + Seek> {
    source: T,
    pub magic_number: String,
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeat: u32,
    pub entries_start: u32,
    pub entries: Vec<WadEntry>,
    pub entries_by_path: HashMap<String, u32>,
}

#[derive(Debug)]
pub struct WadEntry {
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

#[derive(Debug)]
pub struct WadTimestamp {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
}

#[derive(Debug)]
pub struct WadTimestampShort {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
}

impl<T: Read + Seek> Wad<T> {
    pub fn decode(mut source: T) -> Option<Self> {
        let mut header = &mut [0; 32][..];

        source.read_exact(&mut header).ok()?;

        let magic_number = header.parse_str(4)?.to_owned();
        let version = (
            header.parse_u32_le()?,
            header.parse_u32_le()?,
            header.parse_u32_le()?,
        );
        let block_size = header.parse_u32_le()?;
        let entry_count = header.parse_u32_le()?;
        let entry_count_repeat = header.parse_u32_le()?;
        let entries_start = header.parse_u32_le()?;

        let mut entries_data = Vec::new();

        source.seek(SeekFrom::Start(entries_start as u64)).ok()?;
        source.read_to_end(&mut entries_data).ok()?;

        let mut entries_data = &entries_data[..];

        let mut entries = Vec::new();
        let mut entries_by_path = HashMap::new();

        while entries.len() < entry_count as usize {
            let unknown_1 = entries_data.advance(16)?.to_owned();
            let id = entries_data.parse_u32_le()?;
            let unknown_2 = entries_data.parse_u32_le()?;
            let data_size = entries_data.parse_u32_le()?;
            let data_start = entries_data.parse_u32_le()?;
            let unknown_3 = entries_data.parse_u32_le()?;
            let path = entries_data.parse_str_with_u32_le_prefix()?.to_owned();
            let unknown_4 = entries_data.advance(16)?.to_owned();
            let created = Self::decode_timestamp(&mut entries_data)?;
            let accessed = Self::decode_timestamp(&mut entries_data)?;
            let modified = Self::decode_timestamp_short(&mut entries_data)?;

            if id != entries.len() as u32 {
                return None;
            }

            entries_by_path.insert(path, id).xor(Some(0))?;

            entries.push(WadEntry {
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

        Some(Wad {
            source,
            magic_number,
            version,
            block_size,
            entry_count,
            entry_count_repeat,
            entries_start,
            entries,
            entries_by_path,
        })
    }

    fn decode_timestamp(data: &mut &[u8]) -> Option<WadTimestamp> {
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

    fn decode_timestamp_short(data: &mut &[u8]) -> Option<WadTimestampShort> {
        Some(WadTimestampShort {
            year: data.parse_u32_le()?,
            month: data.parse_u32_le()?,
            day: data.parse_u32_le()?,
            hour: data.parse_u32_le()?,
            minute: data.parse_u32_le()?,
        })
    }

    pub fn read_entry(&mut self, entry: &WadEntry, buf: &mut [u8]) -> Result<(), IoError> {
        let max_len = buf.len();
        let read_buf = &mut buf[..(entry.data_size as usize).min(max_len)];
        self.source.seek(SeekFrom::Start(entry.data_start as u64))?;
        self.source.read_exact(read_buf)?;
        Ok(())
    }
}
