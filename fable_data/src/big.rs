use std::io::{Read,Seek,SeekFrom};

use views::{View,Bytes,BadPos};

use crate::BytesExt;

#[derive(Debug)]
pub struct Big {
    pub magic_number: String,
    pub version: u32,
    pub bank_address: u32,
    pub banks_count: u32,
    pub banks: Vec<BigBank>,

}

#[derive(Debug)]
pub struct BigBank {
    pub path: String,
    pub unknown_1: u32,
    pub entries_count: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
    pub index: BigIndex
}

#[derive(Debug)]
pub struct BigIndex {
    pub file_type_count: u32,
    pub file_types: Vec<(u32,u32)>,
    pub entries: Vec<BigEntry>,
}

#[derive(Debug)]
pub struct BigEntry {
    pub magic_number: u32,
    pub id: u32,
    pub kind: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub kind_2: u32,
    pub symbol: String,
    pub crc: u32,
    pub source_file_count: u32,
    pub source_file_paths: Vec<String>,
    // TODO: Figure this out
    pub sub_header_size: u32,
    pub sub_header: Vec<u8>,
}

impl Big {
    pub fn decode<T: Read + Seek>(mut source: &mut T) -> Result<Self, BadPos> {
        let mut header = &mut [0; 12][..];
        source.read_exact(&mut header).map_err(|_| BadPos)?;

        let magic_number = header.take_as_str(4)?.to_owned();
        let version = header.take_u32_le()?;
        let bank_address = header.take_u32_le()?;

        let (banks_count, banks) = Self::decode_banks(&mut source, bank_address as usize)?;

        Ok(Big {
            magic_number,
            version,
            bank_address,
            banks_count,
            banks,
        })
    }

    fn decode_banks<T: Read + Seek>(mut source: &mut T, bank_address: usize) -> Result<(u32, Vec<BigBank>), BadPos> {
        let mut banks_source = vec![];
        source.seek(SeekFrom::Start(bank_address as u64)).or(Err(BadPos))?;
        source.read_to_end(&mut banks_source).or(Err(BadPos))?;

        let mut banks_source = &banks_source[..];
        let banks_count = banks_source.take_u32_le()?;
        let mut banks = Vec::new();

        while banks.len() < banks_count as usize {
            let path = std::str::from_utf8(banks_source.take_until_nul()?).map_err(|_| BadPos)?.to_owned();
            let unknown_1 = banks_source.take_u32_le()?;
            let entries_count = banks_source.take_u32_le()?;
            let index_start = banks_source.take_u32_le()?;
            let index_size = banks_source.take_u32_le()?;
            let block_size = banks_source.take_u32_le()?;

            let index = Self::decode_index(
                &mut source,
                index_start as usize,
                index_size as usize,
                entries_count as usize,
            )?;

            banks.push(BigBank {
                path,
                unknown_1,
                entries_count,
                index_start,
                index_size,
                block_size,
                index
            });
        }

        Ok((banks_count, banks))
    }

    fn decode_index<T: Read + Seek>(
        source: &mut T,
        start: usize,
        size: usize,
        entries_count: usize
    ) -> Result<BigIndex, BadPos> {
        let mut index_source = &mut vec![0; size][..];
        source.seek(SeekFrom::Start(start as u64)).or(Err(BadPos))?;
        source.read_exact(&mut index_source).or(Err(BadPos))?;

        let file_type_count = index_source.take_u32_le()?;
        let mut file_types = Vec::new();

        while file_types.len() < file_type_count as usize {
            let a = index_source.take_u32_le()?;
            let b = index_source.take_u32_le()?;
            file_types.push((a, b));
        }

        let mut entries = Vec::new();

        while entries.len() < entries_count {
            let magic_number = index_source.take_u32_le()?;
            let id = index_source.take_u32_le()?;
            let kind = index_source.take_u32_le()?;
            let data_size = index_source.take_u32_le()?;
            let data_start = index_source.take_u32_le()?;
            let kind_2 = index_source.take_u32_le()?;
            let symbol = index_source.take_as_str_with_u32_le_prefix()?.to_owned();
            let crc = index_source.take_u32_le()?;

            let source_file_count = index_source.take_u32_le()?;
            let mut source_file_paths = Vec::new();

            while source_file_paths.len() < source_file_count as usize {
                source_file_paths.push(index_source.take_as_str_with_u32_le_prefix()?.to_owned());
            }

            let sub_header_size = index_source.take_u32_le()?;
            let sub_header = View::take(&mut index_source, sub_header_size as usize)?.to_owned();

            entries.push(BigEntry {
                magic_number,
                id,
                kind,
                data_size,
                data_start,
                kind_2,
                symbol,
                crc,
                source_file_count,
                source_file_paths,
                sub_header_size,
                sub_header,
            });
        }

        Ok(BigIndex {
            file_type_count,
            file_types,
            entries,
        })
    }
}

impl BigEntry {
    pub fn read_from<T: Read + Seek>(&self, source: &mut T, buf: &mut [u8]) -> Result<(), BadPos> {
        let read_buf = buf.get_mut(..self.data_size as usize).ok_or(BadPos)?;
        source.seek(SeekFrom::Start(self.data_start as u64)).or(Err(BadPos))?;
        source.read_exact(read_buf).or(Err(BadPos))?;
        Ok(())
    }
}