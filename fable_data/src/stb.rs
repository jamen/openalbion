use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::Bytes;

#[derive(Debug)]
pub struct StbHeader {
    pub magic_number: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub entry_count: u32,
    pub level_count: u32,
    pub entries_start: u32,
}

impl StbHeader {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        let magic_number = data.advance(4)?.try_into().ok()?;
        let version = [
            data.parse_u32_le()?,
            data.parse_u32_le()?,
            data.parse_u32_le()?,
        ];
        let block_size = data.parse_u32_le()?;
        let entry_count = data.parse_u32_le()?;
        let level_count = data.parse_u32_le()?;
        let entries_start = data.parse_u32_le()?;
        Some(StbHeader {
            magic_number,
            version,
            block_size,
            entry_count,
            level_count,
            entries_start,
        })
    }
}

#[derive(Debug)]
pub struct StbEntries {
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub level_count_again: u32,
    pub entries: Vec<StbEntry>,
}

impl StbEntries {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        let unknown_1 = data.parse_u32_le()?;
        let unknown_2 = data.parse_u32_le()?;
        let level_count_again = data.parse_u32_le()?;
        let entries = StbEntry::parse_all(data, level_count_again as usize)?;

        Some(StbEntries {
            unknown_1,
            unknown_2,
            level_count_again,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct StbEntry {
    pub unknown_1: u32,
    pub id: u32,
    pub unknown_2: u32,
    pub len: u32,
    pub pos: u32,
    pub unknown_3: u32,
    pub name_1: String,
    pub unknown_4: u32,
    pub unknown_5: u32,
    pub name_2: String,
    pub extras_len: u32,
    pub extras: Option<StbEntryExtras>,
}

impl StbEntry {
    fn parse_all(data: &mut &[u8], entries_count: usize) -> Option<Vec<Self>> {
        let mut entries = Vec::new();

        while entries.len() < entries_count {
            let unknown_1 = data.parse_u32_le()?;
            let id = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let len = data.parse_u32_le()?;
            let pos = data.parse_u32_le()?;
            let unknown_3 = data.parse_u32_le()?;

            let name_1_len = data.parse_u32_le()?;
            let name_1 = data.parse_str(name_1_len as usize)?.to_owned();

            let unknown_4 = data.parse_u32_le()?;
            let unknown_5 = data.parse_u32_le()?;

            let name_2_len = data.parse_u32_le()?;
            let name_2 = data.parse_str(name_2_len as usize)?.to_owned();

            let (extras_len, extras) = StbEntryExtras::parse(data)?;

            entries.push(StbEntry {
                unknown_1,
                id,
                unknown_2,
                len,
                pos,
                unknown_3,
                name_1,
                unknown_4,
                unknown_5,
                name_2,
                extras_len,
                extras,
            });
        }

        Some(entries)
    }
}

#[derive(Debug)]
pub struct StbEntryExtras {
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: u32,
}

impl StbEntryExtras {
    fn parse(data: &mut &[u8]) -> Option<(u32, Option<Self>)> {
        let extras_len = data.parse_u32_le()?;
        let extras = if extras_len == 16 {
            let unknown_1 = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let unknown_3 = data.parse_u32_le()?;
            let unknown_4 = data.parse_u32_le()?;
            Some(StbEntryExtras {
                unknown_1,
                unknown_2,
                unknown_3,
                unknown_4,
            })
        } else {
            None
        };
        Some((extras_len, extras))
    }
}

#[derive(Debug)]
pub struct StbLev {
    pub offset: u32,
    pub compressed_size: u32,
    pub start_x: f32,
    pub start_y: f32,
    pub start_z: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub end_z: f32,
    pub unknown_1: u32,
}

pub struct StbStaticMapCommon {
    pub entries_count: u32,
    pub entries: Vec<(String, StbStaticMapEntry)>,
}

impl StbStaticMapCommon {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        // let _static_map_common_entry = self
        //     .entries
        //     .iter()
        //     .find(|x| x.name_1 == "__STATIC_MAP_COMMON_HEADER__")?;

        let data_lookback = &data[..];

        let entries_count = data.parse_u32_le()?;

        let mut entries = Vec::with_capacity(entries_count as usize);

        while entries.len() < entries_count as usize {
            let path = data.parse_str_until_nul()?.to_owned();
            let offset = data.parse_u32_le()?;

            let _entry_data = &data_lookback[offset as usize..];

            let unknown_1 = data.parse_u32_le()?;
            let id = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let width = data.parse_u32_le()?;
            let height = data.parse_u32_le()?;

            entries.push((
                path,
                StbStaticMapEntry {
                    offset,
                    unknown_1,
                    id,
                    unknown_2,
                    width,
                    height,
                },
            ));
        }

        Some(StbStaticMapCommon {
            entries_count,
            entries,
        })
    }
}

pub struct StbStaticMapEntry {
    pub offset: u32,
    pub unknown_1: u32,
    pub id: u32,
    pub unknown_2: u32,
    pub width: u32,
    pub height: u32,
}

impl StbLev {
    pub fn parse(data: &mut &[u8], _block_count: usize) -> Option<StbLev> {
        let _first_block = data.advance(2048)?;

        // println!("{:?}", first_block);

        let second_block_len = data.parse_u32_le()? as usize;
        let second_block = data
            .advance(second_block_len + (second_block_len % 2048))?
            .to_owned();
        let _second_block = (&second_block[..second_block_len]).to_owned();

        // let second_block = data.grab((second_block_len as usize).min(2048) - 4)?;

        // println!("{:?}", second_block);

        // println!("second block len {:?}", second_block_len);

        let mut blocks = Vec::new();

        while blocks.len() < 19 {
            let decompressed_size = data.parse_u32_le()?;
            let compressed_len = data.parse_u32_le()?;
            // println!("{:?} {:?}", decompressed_size, compressed_len);
            let compressed_data = data.advance(compressed_len as usize)?.to_owned();
            data.advance(2040usize.saturating_sub(compressed_data.len()))?;
            let mut decompressed = vec![0u8; decompressed_size as usize];
            // println!("{:?} {:?} {:x?}", decompressed_size, compressed_len, compressed_data);
            let _ = lzokay::decompress::decompress(&compressed_data, &mut decompressed).ok()?;
            // println!("{:?} {:?} {:?}", decompressed_size, compressed_len, decompressed);
            // println!("{:?} {:?} {:?}", decompressed_size, compressed_len, decompressed);
            blocks.push((decompressed_size, compressed_len, compressed_data));
        }

        // source.seek(SeekFrom::Start(2048)).ok()?;
        // source.read_to_end(&mut data).ok()?;

        // let mut data = &data[..];

        // let offset = data.parse_u32_le()?;
        // let compressed_size = data.parse_u32_le()?;
        // let start_x = data.parse_f32_le()?;
        // let start_y = data.parse_f32_le()?;
        // let start_z = data.parse_f32_le()?;
        // let end_x = data.parse_f32_le()?;
        // let end_y = data.parse_f32_le()?;
        // let end_z = data.parse_f32_le()?;
        // let unknown_1 = data.parse_u32_le()?;

        // println!("offset {:?}", offset);
        // println!("compressed_size {:?}", compressed_size);
        // println!("start_x {:?}", start_x);
        // println!("start_y {:?}", start_y);
        // println!("start_z {:?}", start_z);
        // println!("end_x {:?}", end_x);
        // println!("end_y {:?}", end_y);
        // println!("end_z {:?}", end_z);
        // println!("unknown_1 {:?}", unknown_1);

        // let compressed_data = &original[offset as usize .. offset as usize + compressed_size as
        // usize];

        // println!("{} {:x?}", compressed_data.len(), compressed_data);

        // println!("compressed_data {:?}", compressed_data);

        // let decompressed_data = minilzo::decompress(&compressed_data, 2048);

        // println!("{:?}", decompressed);

        // let lzo_ctx = rust_lzo::LZOContext::new();

        // let mut decompressed_data = Vec::with_capacity(4096);

        // rust_lzo::LZOContext::decompress_to_slice(compressed_data.clone(), &mut
        // decompressed_data[..]);

        // println!("{:?}", decompressed_data);

        None
    }
}
