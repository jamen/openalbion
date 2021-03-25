// use crate::{BytesExt,Result,BadPos};

// use views::{Bytes,BadPos};

use crate::BytesExt;

use std::io::{Read,Seek,SeekFrom};
use views::{Bytes,BadPos};

#[derive(Debug)]
pub struct Wad {
    pub magic_number: String,
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_count_repeat: u32,
    pub entries_start: u32,
    pub entries: Vec<WadEntry>,
}

#[derive(Debug)]
pub struct WadEntry {
    pub unknown_1: Vec<u8>,
    pub id: u32,
    pub unknown_2: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub unknown_3: u32,
    pub path: String,
    pub unknown_4: String,
    pub created: WadTimestamp,
    pub accessed: WadTimestamp,
    pub modified: WadTimestamp,
}

#[derive(Debug)]
pub enum WadTimestamp {
    Normal { year: u32, month: u32, day: u32, hour: u32, minute: u32, second: u32, millisecond: u32 },
    Short { year: u32, month: u32, day: u32, hour: u32, minute: u32 },
}

impl Wad {
    pub fn decode<T: Read + Seek>(mut source: T) -> Result<Self, BadPos> {
        let mut header = &mut [0; 32][..];

        source.read_exact(&mut header).or(Err(BadPos))?;

        let magic_number = header.take_as_str(4)?.to_owned();
        let version = (header.take_u32_le()?, header.take_u32_le()?, header.take_u32_le()?);
        let block_size = header.take_u32_le()?;
        let entry_count = header.take_u32_le()?;
        let entry_count_repeat = header.take_u32_le()?;
        let entries_start = header.take_u32_le()?;

        let mut entries_data = Vec::new();

        source.seek(SeekFrom::Start(entries_start as u64)).or(Err(BadPos))?;
        source.read_to_end(&mut entries_data).or(Err(BadPos))?;

        let mut entries_data = &entries_data[..];

        let mut entries = Vec::new();

        while entries.len() < entry_count as usize {
            let unknown_1 = Bytes::take(&mut entries_data, 16)?.to_owned();
            let id = entries_data.take_u32_le()?;
            let unknown_2 = entries_data.take_u32_le()?;
            let data_size = entries_data.take_u32_le()?;
            let data_start = entries_data.take_u32_le()?;
            let unknown_3 = entries_data.take_u32_le()?;
            let path = entries_data.take_as_str_with_u32_le_prefix()?.to_owned();
            let unknown_4 = entries_data.take_as_str(16)?.to_owned();
            let created = Self::decode_timestamp(&mut entries_data)?;
            let accessed = Self::decode_timestamp(&mut entries_data)?;
            let modified = Self::decode_timestamp_short(&mut entries_data)?;

            entries.push(WadEntry {
                unknown_1,
                id,
                unknown_2,
                data_size,
                data_start,
                unknown_3,
                path,
                unknown_4,
                created,
                accessed,
                modified,
            });
        }

        Ok(
            Wad {
                magic_number,
                version,
                block_size,
                entry_count,
                entry_count_repeat,
                entries_start,
                entries
            }
        )
    }

    fn decode_timestamp(data: &mut &[u8]) -> Result<WadTimestamp, BadPos> {
        Ok(WadTimestamp::Normal {
            year: data.take_u32_le()?,
            month: data.take_u32_le()?,
            day: data.take_u32_le()?,
            hour: data.take_u32_le()?,
            minute: data.take_u32_le()?,
            second: data.take_u32_le()?,
            millisecond: data.take_u32_le()?,
        })
    }

    fn decode_timestamp_short(data: &mut &[u8]) -> Result<WadTimestamp, BadPos> {
        Ok(WadTimestamp::Short {
            year: data.take_u32_le()?,
            month: data.take_u32_le()?,
            day: data.take_u32_le()?,
            hour: data.take_u32_le()?,
            minute: data.take_u32_le()?,
        })
    }
}

impl WadEntry {
    pub fn read_from<T: Read + Seek>(&self, mut source: T, buf: &mut [u8]) -> Result<(), BadPos> {
        let read_buf = buf.get_mut(..self.data_size as usize).ok_or(BadPos)?;
        source.seek(SeekFrom::Start(self.data_start as u64)).or(Err(BadPos))?;
        source.read_exact(read_buf).or(Err(BadPos))?;
        Ok(())
    }
}