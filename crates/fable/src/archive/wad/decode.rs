use std::io::Read;

use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{tag,take};
use nom::sequence::tuple;
use nom::multi::count;

use crate::shared::{Decode,Error};
use crate::shared::timestamp::{decode_timestamp,decode_short_timestamp};
use crate::shared::string::decode_bytes_as_utf8;

use super::{
    Wad,
    WadEntry,
};

// impl Decode for Wad {
//     fn decode(source: impl Read) -> Result<Self, Error> {
//     }
// }

impl Wad {
    pub fn decode_header(input: &[u8]) -> IResult<&[u8], Wad, Error> {
        let (input, _magic_number) = tag("BBBB")(input)?;
        let (input, version) = tuple((le_u32, le_u32, le_u32))(input)?;
        let (input, block_size) = le_u32(input)?;
        let (input, entries_count) = le_u32(input)?;
        let (input, _entries_count_again) = le_u32(input)?;
        let (input, entries_offset) = le_u32(input)?;
        let (input, entries) = count(Self::decode_entry, entries_count as usize)(input)?;

        Ok(
            (
                input,
                Wad {
                    version: version,
                    block_size: block_size,
                    entries_count: entries_count,
                    entries_offset: entries_offset,
                    entries: entries,
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