//! The `.fmp` (Fable mod package) container.
//!
//! An `.fmp` is a BIG-style container with a distinct header: the magic is `B\0\0\0` rather
//! than `BIGB`, the version is 101 rather than 100, and the word BIG leaves unknown carries a
//! content type (510 or 459). The bank table that follows is byte-for-byte
//! [`crate::big::BankMetadata`] entries, so [`crate::big::BigContainer`] parses it unchanged.
//!
//! What is *not* shared is the per-bank content: an `.fmp` bank holds FMP record formats, not
//! BIG asset tables. This reader therefore stops at the container and hands each bank back as
//! raw bytes.
//!
//! Some packages are wrapped: a little-endian `u32` `12345` followed by a zlib stream, which
//! inflates to the container above.

use crate::big::{
    AssetMetadata, AssetMetadataError, AssetMetadataRef, BankMetadata, BigContainer,
    BigContainerError, Header, HeaderError, ReadAssetDataError,
};
use derive_more::{Display, Error};
use std::io::{self, Cursor, Read};

/// Magic at offset 0 of an `.fmp` container: ASCII `B` then three zero bytes.
pub const FMP_MAGIC: [u8; 4] = [0x42, 0x00, 0x00, 0x00];

/// The container version every `.fmp` carries.
pub const FMP_VERSION: u32 = 101;

/// Magic of the compressed wrapper: a little-endian `u32` `12345` before a zlib stream.
pub const WRAPPER_MAGIC: u32 = 12345;

/// Reads an `.fmp` down to its bank metadata table.
///
/// Bank contents are deliberately opaque here: an `.fmp` bank is not a BIG bank, and its
/// record formats are a separate question. Use [`FmpReader::read_bank_raw`] to get a bank's
/// bytes for decoding elsewhere.
pub struct FmpReader {
    container: BigContainer<Cursor<Vec<u8>>>,
}

#[derive(Debug, Display, Error)]
pub enum FmpReaderError {
    Read(io::Error),
    #[display("inflate wrapped .fmp: {_0}")]
    #[error(ignore)]
    Inflate(String),
    ParseHeader(HeaderError),
    #[display("bad .fmp magic: expected B\\0\\0\\0, found {_0:?}")]
    #[error(ignore)]
    Magic([u8; 4]),
    #[display("unsupported .fmp version {_0} (expected {FMP_VERSION})")]
    #[error(ignore)]
    Version(u32),
    Container(BigContainerError),
}

impl FmpReader {
    pub fn new<T: Read>(mut source: T) -> Result<Self, FmpReaderError> {
        use FmpReaderError as E;

        let mut raw = Vec::new();
        source.read_to_end(&mut raw).map_err(E::Read)?;

        let bytes = if raw.len() >= 4
            && u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) == WRAPPER_MAGIC
        {
            miniz_oxide::inflate::decompress_to_vec_zlib(&raw[4..])
                .map_err(|e| E::Inflate(format!("{:?}", e.status)))?
        } else {
            raw
        };

        if bytes.len() < Header::BYTE_SIZE {
            return Err(E::Read(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated .fmp header",
            )));
        }

        let header = Header::parse(&mut &bytes[..Header::BYTE_SIZE]).map_err(E::ParseHeader)?;

        if header.magic != FMP_MAGIC {
            return Err(E::Magic(header.magic));
        }

        if header.version != FMP_VERSION {
            return Err(E::Version(header.version));
        }

        let container = BigContainer::new(Cursor::new(bytes)).map_err(E::Container)?;

        Ok(Self { container })
    }

    pub fn header(&self) -> &Header {
        self.container.header()
    }

    pub fn version(&self) -> u32 {
        self.container.header().version
    }

    /// The word BIG leaves unknown: `510` or `459` on every shipped package.
    pub fn content_type(&self) -> u32 {
        self.container.header().unknown_1
    }

    pub fn banks(&self) -> &[BankMetadata] {
        self.container.banks()
    }

    pub fn bank(&self, name: &str) -> Option<&BankMetadata> {
        self.container.bank(name)
    }

    /// Read a bank's bytes exactly as they sit in the (possibly inflated) container.
    pub fn read_bank_raw(&mut self, bank: &BankMetadata) -> Result<Vec<u8>, ReadAssetDataError> {
        self.container.read_bank_raw(bank)
    }

    /// Read a record's payload bytes, wherever in the container they sit.
    pub fn read_payload(
        &mut self,
        record: &AssetMetadata,
    ) -> Result<Vec<u8>, ReadAssetDataError> {
        self.container.read_range(record.start, record.size)
    }

    /// Parse a bank's record table.
    ///
    /// A bank's records are BIG [`AssetMetadata`] entries — the same structure `big.rs` already
    /// parses, just without the leading type map, and preceded by a single `u32` prefix. The
    /// `extras` carry the FMP-specific meaning (a definition type for `GameBINEntries`, texture
    /// or mesh metadata for the image banks).
    ///
    /// A handful of packages use an older, capital-named dialect whose records do not fit this
    /// layout; those return [`FmpRecordError`] rather than a wrong record.
    pub fn bank_records(
        &mut self,
        bank: &BankMetadata,
    ) -> Result<Vec<AssetMetadata>, FmpRecordError> {
        let bytes = self
            .container
            .read_bank_raw(bank)
            .map_err(FmpRecordError::Read)?;

        // Skip the 4-byte bank prefix. An empty bank is exactly this prefix and nothing else.
        let table = bytes.get(4..).ok_or(FmpRecordError::Truncated)?;

        let mut table = table;
        let mut records = Vec::with_capacity(bank.asset_count as usize);

        for _ in 0..bank.asset_count {
            let record = AssetMetadataRef::parse(&mut table)
                .map_err(FmpRecordError::Metadata)?
                .into_owned();

            records.push(record);
        }

        Ok(records)
    }
}

#[derive(Debug, Display, Error)]
pub enum FmpRecordError {
    Read(ReadAssetDataError),
    Truncated,
    #[display("record metadata: {_0}")]
    #[error(ignore)]
    Metadata(AssetMetadataError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal, well-formed container: header, one bank's bytes, then the bank table.
    fn container(bank_name: &str, bank_bytes: &[u8]) -> Vec<u8> {
        let bank_pos = Header::BYTE_SIZE as u32;
        let table_pos = bank_pos + bank_bytes.len() as u32;

        let mut bank_table = Vec::new();
        bank_table.extend_from_slice(&1u32.to_le_bytes());
        bank_table.extend_from_slice(bank_name.as_bytes());
        bank_table.push(0);
        bank_table.extend_from_slice(&0u32.to_le_bytes()); // id
        bank_table.extend_from_slice(&1u32.to_le_bytes()); // asset_count
        bank_table.extend_from_slice(&bank_pos.to_le_bytes()); // position
        bank_table.extend_from_slice(&(bank_bytes.len() as u32).to_le_bytes()); // length
        bank_table.extend_from_slice(&1u32.to_le_bytes()); // block_size

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&FMP_MAGIC);
        bytes.extend_from_slice(&FMP_VERSION.to_le_bytes());
        bytes.extend_from_slice(&table_pos.to_le_bytes());
        bytes.extend_from_slice(&510u32.to_le_bytes()); // content type
        bytes.extend_from_slice(bank_bytes);
        bytes.extend_from_slice(&bank_table);
        bytes
    }

    #[test]
    fn parses_container_and_bank_bytes() {
        let bytes = container("TESTBANK", &[0xDE, 0xAD, 0xBE, 0xEF]);

        let mut reader = FmpReader::new(Cursor::new(bytes)).expect("parse fmp");

        assert_eq!(reader.version(), FMP_VERSION);
        assert_eq!(reader.content_type(), 510);
        assert_eq!(reader.banks().len(), 1);

        let bank = reader.banks()[0].clone();
        assert_eq!(bank.name, "TESTBANK");
        assert_eq!(bank.id, 0);
        assert_eq!(bank.asset_count, 1);

        let raw = reader.read_bank_raw(&bank).expect("read bank");
        assert_eq!(raw, [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn unwraps_12345_zlib_wrapper() {
        let inner = container("TESTBANK", &[1, 2, 3, 4]);

        let mut wrapped = WRAPPER_MAGIC.to_le_bytes().to_vec();
        wrapped.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(&inner, 6));

        let mut reader = FmpReader::new(Cursor::new(wrapped)).expect("parse wrapped fmp");

        assert_eq!(reader.content_type(), 510);
        assert_eq!(reader.banks().len(), 1);

        let bank = reader.banks()[0].clone();
        assert_eq!(
            reader.read_bank_raw(&bank).expect("read bank"),
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn parses_bank_asset_metadata() {
        // A bank is a `u32` prefix followed by BIG asset-metadata records.
        let mut bank = Vec::new();
        bank.extend_from_slice(&0u32.to_le_bytes()); // prefix
        bank.extend_from_slice(&0u32.to_le_bytes()); // magic
        bank.extend_from_slice(&7u32.to_le_bytes()); // id
        bank.extend_from_slice(&0u32.to_le_bytes()); // file_type
        bank.extend_from_slice(&4u32.to_le_bytes()); // size
        bank.extend_from_slice(&16u32.to_le_bytes()); // start
        bank.extend_from_slice(&0u32.to_le_bytes()); // file_type_dev
        let name = b"OBJECT_TEST";
        bank.extend_from_slice(&(name.len() as u32).to_le_bytes());
        bank.extend_from_slice(name);
        bank.extend_from_slice(&0u32.to_le_bytes()); // crc
        bank.extend_from_slice(&0u32.to_le_bytes()); // files_count
        let def = b"OBJECT\0";
        bank.extend_from_slice(&(def.len() as u32).to_le_bytes());
        bank.extend_from_slice(def);

        let bytes = container("TESTBANK", &bank);
        let mut reader = FmpReader::new(Cursor::new(bytes)).expect("parse fmp");
        let bank = reader.banks()[0].clone();

        let records = reader.bank_records(&bank).expect("parse records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].symbol_name, "OBJECT_TEST");
        assert_eq!(
            records[0].extras,
            Some(crate::big::ExtraMetadata::Unknown(def.to_vec()))
        );
    }

    #[test]
    fn rejects_non_fmp_magic() {
        let mut bytes = container("TESTBANK", &[0; 4]);
        bytes[..4].copy_from_slice(&[0x01, 0x00, 0x01, 0x00]);

        assert!(matches!(
            FmpReader::new(Cursor::new(bytes)),
            Err(FmpReaderError::Magic(_))
        ));
    }
}
