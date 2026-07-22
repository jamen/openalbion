use crate::bytes::{TakeError, TakeNullTerminatedUtf8, take, take_null_terminated_utf8};
use crate::bytes::{put, put_bytes, put_null_terminated_utf8};
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

    /// Serialize to the wire format (header + entries).
    pub fn to_bytes(&self) -> Vec<u8> {
        let total: usize = 20 + self.map.values().map(|e| e.byte_size()).sum::<usize>();
        let mut out = vec![0u8; total];
        let mut cur: &mut [u8] = &mut out;
        put_bytes(&mut cur, &self.header_bytes).unwrap();
        for entry in self.map.values() {
            entry.serialize(&mut cur).unwrap();
        }
        debug_assert!(cur.is_empty(), "Names::to_bytes: buffer over/underflow");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_names_bin() {
        // Load the real OG retail names.bin and verify byte-exact roundtrip.
        let path = Path::new(env!("HOME")).join("Fable/data/CompiledDefs/names.bin");
        if !path.exists() {
            return; // skip if data not available
        }
        let original = std::fs::read(&path).unwrap();
        let names = Names::from_bytes(&original).unwrap();
        let re_serialized = names.to_bytes();
        assert_eq!(re_serialized.len(), original.len(), "length mismatch");
        assert_eq!(re_serialized, original, "byte-exact roundtrip failed");
    }
}

#[derive(Debug, Clone)]
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

    fn byte_size(&self) -> usize {
        4 + self.string.len() + 1
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), crate::bytes::UnexpectedEnd> {
        put(out, &self.crc)?;
        put_null_terminated_utf8(out, &self.string)?;
        Ok(())
    }
}
