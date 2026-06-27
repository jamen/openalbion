use crate::bytes::{TakeError, TakeNullTerminatedUtf8, take, take_null_terminated_utf8};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

#[derive(Debug)]
pub struct Names {
    pub header_bytes: [u8; 20],
    pub map: BTreeMap<u32, NamesEntry>,
}

#[derive(Debug)]
pub enum LoadError {
    Open(io::Error),
    FromReader(FromReaderError),
}

impl Names {
    pub fn load(path: &Path) -> Result<Self, LoadError> {
        use LoadError as E;
        let file = File::open(path).map_err(E::Open)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader).map_err(E::FromReader)
    }
}

#[derive(Debug)]
pub enum FromReaderError {
    Read(io::Error),
    FromBytes(FromBytesError),
}

impl Names {
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, FromReaderError> {
        use FromReaderError as E;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).map_err(E::Read)?;
        Self::from_bytes(&buf).map_err(E::FromBytes)
    }
}

#[derive(Debug)]
pub enum FromBytesError {
    ParseHeaderBytes,
    ParseEntry(usize, ParseEntryError),
}

impl Names {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FromBytesError> {
        use FromBytesError as E;

        let bytes_cursor = &mut &bytes[..];

        // Parse header bytes

        let header_bytes = bytes_cursor
            .split_off(..20)
            .ok_or(E::ParseHeaderBytes)?
            .try_into()
            .unwrap();

        // Parse map

        let mut map = BTreeMap::new();

        while !bytes_cursor.is_empty() {
            let offset = bytes.len() - bytes_cursor.len();

            let entry =
                NamesEntry::parse(bytes_cursor).map_err(|error| E::ParseEntry(offset, error))?;

            let string_offset = (offset + 4 - 20) as u32;

            map.insert(string_offset, entry);
        }

        Ok(Self { header_bytes, map })
    }
}

#[derive(Debug)]
pub struct NamesEntry {
    pub crc: u32,
    pub string: String,
}

#[derive(Debug)]
pub enum ParseEntryError {
    Crc(TakeError),
    String(TakeNullTerminatedUtf8),
}

impl NamesEntry {
    fn parse(input: &mut &[u8]) -> Result<Self, ParseEntryError> {
        use ParseEntryError as E;

        // Parse stored CRC32

        let crc = take::<u32>(input).map_err(E::Crc)?.to_le();

        // Parse null-terminated string

        let string = take_null_terminated_utf8(input)
            .map_err(E::String)?
            .to_owned();

        Ok(Self { crc, string })
    }
}
