use std::io::{Read,Seek,SeekFrom};

use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{tag,take};
use nom::sequence::tuple;
use nom::multi::count;
use nom::combinator::all_consuming;

use crate::shared::{Decode,Error};
use crate::shared::timestamp::{decode_timestamp,decode_short_timestamp};
use crate::shared::string::decode_bytes_as_utf8;

use super::{
    Wad,
    WadHeader,
    WadEntry,
};

static MAGIC_NUMBER: &'static str = "BBBB";

impl Decode for Wad {
    fn decode(source: &mut (impl Read + Seek)) -> Result<Self, Error> {
        let mut header_buf = Vec::with_capacity(32);
        let mut entries_buf = Vec::new();

        source.read_exact(&mut header_buf)?;
        let (_, header) = all_consuming(Self::decode_header)(&header_buf)?;

        source.seek(SeekFrom::Start(header.entries_offset as u64))?;
        source.read_to_end(&mut entries_buf)?;
        let (_, entries) = all_consuming(count(Self::decode_entry, header.entries_count as usize))(&entries_buf)?;

        Ok(Wad { header: header, entries: entries })
    }
}

impl Wad {
    pub fn decode_header(input: &[u8]) -> IResult<&[u8], WadHeader, Error> {
        let (input, _magic_number) = tag(MAGIC_NUMBER)(input)?;
        let (input, version) = tuple((le_u32, le_u32, le_u32))(input)?;
        let (input, block_size) = le_u32(input)?;
        let (input, entries_count) = le_u32(input)?;
        let (input, _entries_count_again) = le_u32(input)?;
        let (input, entries_offset) = le_u32(input)?;

        Ok(
            (
                input,
                WadHeader {
                    version: version,
                    block_size: block_size,
                    entries_count: entries_count,
                    entries_offset: entries_offset,
                }
            )
        )
    }

    pub fn decode_entry(input: &[u8]) -> IResult<&[u8], WadEntry, Error> {
        let (input, _unknown_1) = take(16usize)(input)?;
        let (input, id) = le_u32(input)?;
        let (input, _unknown_2) = le_u32(input)?;
        let (input, length) = le_u32(input)?;
        let (input, offset) = le_u32(input)?;
        let (input, _unknown_3) = le_u32(input)?;
        let (input, path_length) = le_u32(input)?;
        let (input, path) = take(path_length as usize)(input)?;

        let (_, path) = decode_bytes_as_utf8(path)?;

        let (input, _unknown_4) = take(16usize)(input)?;

        let (input, created_at) = decode_timestamp(input)?;
        let (input, accessed_at) = decode_timestamp(input)?;
        let (input, written_at) = decode_short_timestamp(input)?;

        Ok(
            (
                input,
                WadEntry {
                    id: id,
                    length: length,
                    offset: offset,
                    path: path,
                    created_at: created_at,
                    accessed_at: accessed_at,
                    written_at: written_at,
                }
            )
        )
    }
}