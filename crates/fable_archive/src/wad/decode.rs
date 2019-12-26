use fable_base::nom::IResult;
use fable_base::nom::number::complete::le_u32;
use fable_base::nom::bytes::complete::{tag,take};
use fable_base::nom::sequence::tuple;
use fable_base::timestamp::{decode_timestamp,decode_short_timestamp};

use crate::wad::{
    WadHeader,
    WadEntry,
};

pub fn decode_header(input: &[u8]) -> IResult<&[u8], WadHeader> {
    let (input, _magic_number) = tag("BBBB")(input)?;
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

pub fn decode_entry(input: &[u8]) -> IResult<&[u8], WadEntry> {
    let (input, _unknown_1) = take(16usize)(input)?;
    let (input, id) = le_u32(input)?;
    let (input, _unknown_2) = le_u32(input)?;
    let (input, length) = le_u32(input)?;
    let (input, offset) = le_u32(input)?;
    let (input, _unknown_3) = le_u32(input)?;
    let (input, path_length) = le_u32(input)?;
    let (input, path) = take(path_length as usize)(input)?;

    let path = match std::str::from_utf8(path) {
        Err(_error) => return Err(fable_base::nom::Err::Error((input, fable_base::nom::error::ErrorKind::ParseTo))),
        Ok(value) => value.to_owned()
    };

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