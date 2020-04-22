use std::io::{Read,Seek,SeekFrom};

use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{tag,take};
use nom::sequence::tuple;
use nom::multi::count;
use nom::combinator::all_consuming;

use chrono::naive::{NaiveDateTime,NaiveDate,NaiveTime};

use crate::{Decode,Error};
use crate::shared::decode_bytes_as_utf8_string;
use crate::Entry;

use super::{Wad,WadHeader,WadEntry};

impl Decode for Wad {
    fn decode<Source>(source: &mut Source) -> Result<Self, Error> where
        Source: Read + Seek
    {
        let mut header_buf = [0; 32];
        let mut entries_buf = Vec::new();

        source.read_exact(&mut header_buf)?;
        let (_, header) = all_consuming(Wad::decode_header)(&header_buf)?;

        source.seek(SeekFrom::Start(header.entries_offset as u64))?;
        source.read_to_end(&mut entries_buf)?;
        let (_, entries) = count(Wad::decode_entry, header.entries_count as usize)(&entries_buf)?;

        Ok(Wad { header: header, entries: entries })
    }
}

impl Wad {
    pub fn decode_header(input: &[u8]) -> IResult<&[u8], WadHeader, Error> {
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

    pub fn decode_entry(input: &[u8]) -> IResult<&[u8], WadEntry, Error> {
        let (input, _unknown_1) = take(16usize)(input)?;
        let (input, id) = le_u32(input)?;
        let (input, _unknown_2) = le_u32(input)?;
        let (input, length) = le_u32(input)?;
        let (input, offset) = le_u32(input)?;
        let (input, _unknown_3) = le_u32(input)?;
        let (input, path_length) = le_u32(input)?;
        let (input, path) = take(path_length as usize)(input)?;

        let (_, path) = decode_bytes_as_utf8_string(path)?;

        let (input, _unknown_4) = take(16usize)(input)?;

        let (input, created) = Self::decode_timestamp(input)?;
        let (input, accessed) = Self::decode_timestamp(input)?;
        let (input, written) = Self::decode_short_timestamp(input)?;

        Ok(
            (
                input,
                WadEntry {
                    id: id,
                    length: length,
                    offset: offset,
                    path: path,
                    created: created,
                    accessed: accessed,
                    written: written,
                }
            )
        )
    }

    pub fn decode_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime, Error> {
        let (input, year) = le_u32(input)?;
        let (input, month) = le_u32(input)?;
        let (input, day) = le_u32(input)?;
        let (input, hour) = le_u32(input)?;
        let (input, minute) = le_u32(input)?;
        let (input, second) = le_u32(input)?;
        let (input, millisecond) = le_u32(input)?;

        let ymd = NaiveDate::from_ymd(year as i32, month, day);
        let hms = NaiveTime::from_hms_milli(hour, minute, second, millisecond);
        let date_time = NaiveDateTime::new(ymd, hms);

        Ok((input, date_time))
    }

    pub fn decode_short_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime, Error> {
        let (input, year) = le_u32(input)?;
        let (input, month) = le_u32(input)?;
        let (input, day) = le_u32(input)?;
        let (input, hour) = le_u32(input)?;
        let (input, minute) = le_u32(input)?;

        let ymd = NaiveDate::from_ymd(year as i32, month, day);
        let hms = NaiveTime::from_hms(hour, minute, 0);
        let date_time = NaiveDateTime::new(ymd, hms);

        Ok((input, date_time))
    }
}

impl Entry for WadEntry {
    fn len(&self) -> u64 { self.length as u64 }
    fn pos(&self) -> u64 { self.offset as u64 }
}