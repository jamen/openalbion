use std::convert::TryInto;
use std::io::{self,Read,Seek,SeekFrom,Write};
use std::path::Path;
use std::fs::{File,OpenOptions};

use super::{
    Decode,
    Encode,
    Entry,
    Error,
    IResult,
    all_consuming,
    count,
    decode_bytes_as_utf8_string,
    do_parse,
    le_u32,
    tag,
    take,
    tuple,
};

/// The world archive.
///
/// This archive contains [`Lev`] and [`Tng`] files.
///
/// ## Format Description
///
/// The wad header's layout is:
///
/// | Field                  | Type        | Byte Size | Description                     |
/// |------------------------|-------------|-----------|---------------------------------|
/// | Magic number           | `[4; char]` | 4         | `"BBBB"` string.                |
/// | Version                | `[3; u32]`  | 12        | File version number.            |
/// | Block size             | `u32`       | 4         | Chunk size. Usually `2048`.     |
/// | Entry count            | `u32`       | 4         | Amount of entries.              |
/// | Entry count repeated   | `u32`       | 4         |                                 |
/// | First entry offset     | `u32`       | 4         | Offset to the first entry .     |
///
/// Seek the source to the "First entry offset" to begin reading the entries.
///
/// An entry's layout is:
///
/// | Field       | Type       | Byte Size | Description                                      |
/// |-------------|------------|-----------|--------------------------------------------------|
/// | Unknown     | `[16; u8]` | 16        | (Maybe hash-related.)                            |
/// | File Id     | `u32`      | 4         | Index number. (This is implicit tho lol?)        |
/// | Unknown     | `u32`      | 4         |                                                  |
/// | File size   | `u32`      | 4         | File size in the blob.                           |
/// | File offset | `u32`      | 4         | File offset in the blob.                         |
/// | Unknown     | `u32`      | 4         |                                                  |
/// | Path size   | `u32`      | 4         | Size of the path string that follows.            |
/// | Path string | `String`   | Path size | File path                                        |
/// | Unknown     | `[16; u8]` | 16        | (Maybe some kind of metadata like perms.)        |
/// | Created     | `[7; u32]` | 28        | Creation timestamp.                              |
/// | Accessed    | `[7; u32]` | 28        | Access timestamp.                                |
/// | Written     | `[5; u32]` | 20        | Write timestamp.                                 |
///
/// The timestamp layout is:
///
/// | Field       | Type  | Byte Size |
/// |-------------|-------|-----------|
/// | Year        | `u32` | 4         |
/// | Month       | `u32` | 4         |
/// | Day         | `u32` | 4         |
/// | Hour        | `u32` | 4         |
/// | Minute      | `u32` | 4         |
/// | Second      | `u32` | 4         |
/// | Millisecond | `u32` | 4         |
///
/// The "Created" and "Accessed" timestamps go up to milliseconds and the "Written" timestamp goes up to minutes.
///
/// [`WadEntry`]: ./struct.WadEntry.html
/// [`Lev`]: ../struct.Lev.html
/// [`Tng`]: ../struct.Tng.html
#[derive(Debug,PartialEq)]
pub struct Wad {
    pub header: WadHeader,
    pub entries: Vec<WadEntry>
}

#[derive(Debug,PartialEq)]
pub struct WadHeader {
    pub version: (u32, u32, u32),
    pub block_size: u32,
    pub entry_count: u32,
    pub entry_start: u32,
}

#[derive(Debug,PartialEq)]
pub struct WadEntry {
    _unknown_1: [u8; 16],
    pub id: u32,
    _unknown_2: u32,
    pub offset: u32,
    pub length: u32,
    _unknown_3: u32,
    pub path: String,
    _unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

pub struct WadEntryDescriptor {
    // pub id: u32,
    pub length: u32,
    pub path: String,
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

pub struct WadReader<'a, S: Read + Seek> {
    pub archive: &'a mut S,
    pub entry: &'a WadEntry,
    pub pos: usize,
}

pub struct WadWriter<'a, T: Write + Seek> {
    pub archive: &'a mut T,
    pub entry: &'a WadEntry,
    pub pos: usize,
}

impl Decode for Wad {
    type Error = Error;

    fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
        let mut header_buf = [0; 32];

        source.seek(SeekFrom::Start(0))?;
        source.read_exact(&mut header_buf)?;

        let (_, header) = match all_consuming(Self::decode_wad_header)(&header_buf) {
            Ok(header) => header,
            Err(x) => return Err(x.into()),
        };

        let mut entries_buf = Vec::new();

        source.seek(SeekFrom::Start(header.entry_start as u64))?;
        source.read_to_end(&mut entries_buf)?;

        let (_, entries) = count(Self::decode_wad_entry, header.entry_count as usize)(&entries_buf)?;

        Ok(Wad { header, entries })
    }
}

impl Wad {
    fn decode_wad_entry(input: &[u8]) -> IResult<&[u8], WadEntry, Error> {
        let decode_wad_entry_path = Self::decode_wad_entry_path;
        let decode_wad_timestamp = Self::decode_wad_timestamp;
        let decode_wad_timestamp_short = Self::decode_wad_timestamp_short;

        do_parse!(input,
            _unknown_1: take!(16usize) >>
            id: le_u32 >>
            _unknown_2: le_u32 >>
            length: le_u32 >>
            offset: le_u32 >>
            _unknown_3: le_u32 >>
            path: decode_wad_entry_path >>
            _unknown_4: take!(16usize) >>
            created: decode_wad_timestamp >>
            accessed: decode_wad_timestamp >>
            modified: decode_wad_timestamp_short >>
            (WadEntry {
                _unknown_1: _unknown_1.try_into().unwrap(),
                id,
                _unknown_2,
                length,
                offset,
                _unknown_3,
                path,
                _unknown_4: _unknown_4.try_into().unwrap(),
                created,
                accessed,
                modified,
            })
        )
    }

    fn decode_wad_entry_path(input: &[u8]) -> IResult<&[u8], String, Error> {
        let (input, path_length) = le_u32(input)?;
        let (input, path) = take(path_length as usize)(input)?;
        let (_, path) = all_consuming(decode_bytes_as_utf8_string)(path)?;

        Ok((input, path))
    }

    fn decode_wad_header(input: &[u8]) -> IResult<&[u8], WadHeader, Error> {
        do_parse!(input,
            _magic_number: tag!("BBBB") >>
            version: tuple!(le_u32, le_u32, le_u32) >>
            block_size: le_u32 >>
            entry_count: le_u32 >>
            _entry_count_again: le_u32 >>
            entry_start: le_u32 >>
            (WadHeader { version, block_size, entry_count, entry_start })
        )
    }

    fn decode_wad_timestamp(input: &[u8]) -> IResult<&[u8], [u32; 7], Error> {
        do_parse!(input,
            year: le_u32 >>
            month: le_u32 >>
            day: le_u32 >>
            hour: le_u32 >>
            minute: le_u32 >>
            second: le_u32 >>
            millisecond: le_u32 >>
            ([year, month, day, hour, minute, second, millisecond])
        )
    }

    fn decode_wad_timestamp_short(input: &[u8]) -> IResult<&[u8], [u32; 5], Error> {
        do_parse!(input,
            year: le_u32 >>
            month: le_u32 >>
            day: le_u32 >>
            hour: le_u32 >>
            minute: le_u32 >>
            ([year, month, day, hour, minute])
        )
    }
}

impl Encode for Wad {
    type Error = Error;

    fn encode<Target: Write + Seek>(&self, target: &mut Target) -> Result<(), Error> {
        let header_buf = Self::encode_wad_header(&self.header)?;

        target.seek(SeekFrom::Start(0))?;
        target.write_all(header_buf.as_slice())?;

        let entries_buf = Self::encode_wad_entries(self.entries.as_slice())?;

        target.seek(SeekFrom::Start(self.header.entry_start as u64))?;
        target.write_all(entries_buf.as_slice())?;

        Ok(())
    }
}

impl Wad {
    fn encode_wad_header(input: &WadHeader) -> Result<Vec<u8>, Error> {
        Ok(
            [
                b"BBBB",
                &input.version.0.to_le_bytes()[..],
                &input.version.1.to_le_bytes()[..],
                &input.version.2.to_le_bytes()[..],
                &input.block_size.to_le_bytes()[..],
                &input.entry_count.to_le_bytes()[..],
                &input.entry_count.to_le_bytes()[..], // _entry_count_again
                &input.entry_start.to_le_bytes()[..],
            ].concat()
        )
    }

    fn encode_wad_entries(input: &[WadEntry]) -> Result<Vec<u8>, Error> {
        let mut result: Vec<u8> = Vec::new();

        for entry in input {
            result.extend_from_slice(&Self::encode_entry(entry)?[..])
        }

        Ok(result)
    }

    fn encode_entry(input: &WadEntry) -> Result<Vec<u8>, Error> {
        Ok(
            [
                &input._unknown_1[..],
                &input.id.to_le_bytes()[..],
                &input._unknown_2.to_le_bytes()[..],
                &input.length.to_le_bytes()[..],
                &input.offset.to_le_bytes()[..],
                &input._unknown_3.to_le_bytes()[..],

                &(input.path.len() as u32).to_le_bytes()[..],
                input.path.as_bytes(),

                &input._unknown_4[..],

                &Self::encode_timestamp(input.created)?,
                &Self::encode_timestamp(input.accessed)?,
                &Self::encode_timestamp_short(input.modified)?,
            ].concat()
        )
    }

    fn encode_timestamp(input: [u32; 7]) -> Result<Vec<u8>, Error> {
        Ok(
            [
                &input[0].to_le_bytes()[..],
                &input[1].to_le_bytes()[..],
                &input[2].to_le_bytes()[..],
                &input[3].to_le_bytes()[..],
                &input[4].to_le_bytes()[..],
                &input[5].to_le_bytes()[..],
                &input[6].to_le_bytes()[..],
            ].concat()
        )
    }

    fn encode_timestamp_short(input: [u32; 5]) -> Result<Vec<u8>, Error> {
        Ok(
            [
                &input[0].to_le_bytes()[..],
                &input[1].to_le_bytes()[..],
                &input[2].to_le_bytes()[..],
                &input[3].to_le_bytes()[..],
                &input[4].to_le_bytes()[..],
            ].concat()
        )
    }
}

impl Wad {
    // TODO: Make these async?

    pub fn unpack<Source: Read + Seek, P: AsRef<Path>>(&self, mut source: &mut Source, to: P) -> Result<(), Error> {
        let to = to.as_ref();

        for entry in &self.entries {
            let to = to.join(entry.path.clone());
            let mut file = OpenOptions::new().write(true).create(true).open(to)?;
            let mut sub_source = entry.source(&mut source)?;
            io::copy(&mut sub_source, &mut file)?;
        }

        Ok(())
    }

    // TODO: This isn't useful. The implementation needs to change to: get files from a directory, create the entries, and then write the archive.

    // pub fn pack<Sink: Write + Seek, P: AsRef<Path>>(mut sink: &mut Sink, from: P) -> Result<(), Error>

    pub fn pack<Sink: Write + Seek, P: AsRef<Path>>(&self, mut sink: &mut Sink, from: P) -> Result<(), Error> {
        let from = from.as_ref();

        self.encode(sink)?;

        for entry in &self.entries {
            let from = from.join(entry.path.clone());
            let mut file = File::open(from)?;
            let mut sub_source = entry.source(&mut sink)?;
            io::copy(&mut file, &mut sub_source)?;
        }

        Ok(())
    }
}

impl WadEntry {
    pub fn source<S: Seek>(&self, source: S) -> Result<Entry<S>, Error> {
        Ok(Entry::new(source, self.offset as u64, self.offset as u64 + self.length as u64)?)
    }
}
