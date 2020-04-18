use std::io::{Read,Seek,SeekFrom};

use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::tag;
use nom::combinator::all_consuming;
use nom::multi::count;

use crate::{Error,Decode};
use crate::shared::decode_rle_string;
use crate::Entry;

use super::{
    Stb,
    StbHeader,
    StbEntriesHeader,
    StbEntry,
    StbEntryExtras,
};

impl Decode for Stb {
    fn decode<Source>(source: &mut Source) -> Result<Self, Error> where
        Source: Read + Seek
    {
        let mut header_buf = [0; 32];
        let mut entries_header_buf = [0; 12];
        let mut entries_buf = Vec::new();

        source.read_exact(&mut header_buf)?;
        let (_, header) = all_consuming(Stb::decode_header)(&header_buf)?;

        source.seek(SeekFrom::Start(header.entries_offset as u64))?;
        source.read_exact(&mut entries_header_buf)?;
        let (_, entries_header) = all_consuming(Stb::decode_entries_header)(&header_buf)?;

        source.read_to_end(&mut entries_buf)?;
        let (_, entries) = count(Stb::decode_entry, header.files_count as usize)(&entries_buf)?;

        Ok(Stb { header: header, entries_header: entries_header, entries: entries })
    }
}

impl Stb {
    pub fn decode_header(input: &[u8]) -> IResult<&[u8], StbHeader, Error> {
        let (input, _magic_number) = tag("BBBB")(input)?;
        let (input, version) = le_u32(input)?;
        let (input, _unknown_1) = le_u32(input)?;
        let (input, _unknown_2) = le_u32(input)?;
        let (input, header_size) = le_u32(input)?;
        let (input, files_count) = le_u32(input)?;
        let (input, levels_count) = le_u32(input)?;
        let (input, entries_offset) = le_u32(input)?;

        Ok(
            (
                input,
                StbHeader {
                    version: version,
                    header_size: header_size,
                    files_count: files_count,
                    levels_count: levels_count,
                    entries_offset: entries_offset,
                }
            )
        )
    }

    pub fn decode_entries_header(input: &[u8]) -> IResult<&[u8], StbEntriesHeader, Error> {
        let (input, start) = le_u32(input)?;
        let (input, _null) = le_u32(input)?;
        let (input, levels_count) = le_u32(input)?;

        Ok(
            (
                input,
                StbEntriesHeader {
                    start: start,
                    levels_count: levels_count,
                }
            )
        )
    }

    pub fn decode_entry(input: &[u8]) -> IResult<&[u8], StbEntry, Error> {
        let (input, listing_start) = le_u32(input)?;
        let (input, id) = le_u32(input)?;
        let (input, _null) = le_u32(input)?;
        let (input, length) = le_u32(input)?;
        let (input, offset) = le_u32(input)?;
        let (input, _null) = le_u32(input)?;
        let (input, name_1) = decode_rle_string(input)?;
        let (input, _null) = le_u32(input)?;
        let (input, _unknown_1) = le_u32(input)?;
        let (input, name_2) = decode_rle_string(input)?;
        let (input, bytes_left) = le_u32(input)?;

        // These aren't very useful until they can be understood.

        let (input, extras) =
            // TODO: Is this being misused? Maybe there can be different sized extras.
            if bytes_left != 0 {
                let (input, field_1) = le_u32(input)?;
                let (input, field_2) = le_u32(input)?;
                let (input, field_3) = le_u32(input)?;
                let (input, field_4) = le_u32(input)?;
                (
                    input,
                    Some(
                        StbEntryExtras {
                            field_1: field_1,
                            field_2: field_2,
                            field_3: field_3,
                            field_4: field_4,
                        }
                    )
                )
            } else {
                (input, None)
            };

        Ok(
            (
                input,
                StbEntry {
                    listing_start: listing_start,
                    id: id,
                    length: length,
                    offset: offset,
                    name_1: name_1,
                    name_2: name_2,
                    extras: extras,
                }
            )
        )
    }
}

impl Entry for StbEntry {
    fn len(&self) -> u64 { self.length as u64 }
    fn pos(&self) -> u64 { self.offset as u64 }
}