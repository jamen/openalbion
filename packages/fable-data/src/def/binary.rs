use crate::bytes::{TakeError, UnexpectedEnd, take, put, put_bytes};
use derive_more::{Display, Error, From};

/// Compiled def binary format (game.bin, frontend.bin, script.bin).
///
/// The file consists of:
/// 1. Header (13 bytes)
/// 2. Name lookup table (12 bytes per entry)
/// 3. Chunk table header (8 bytes)
/// 4. Chunk table (8 bytes per chunk)
/// 5. Zlib-compressed data chunks
///
/// Each compressed chunk, when decompressed, contains:
/// - A u16 offset table (2 bytes per entry in the chunk)
/// - Concatenated entry data (positional fields per schema)
pub struct DefBin {
    pub header: Header,
    pub name_lookups: Vec<NameLookup>,
    pub chunk_table_header: ChunkTableHeader,
    pub chunk_table: Vec<ChunkTableRow>,
    pub chunks_data: Vec<u8>,
}

// -- Header (13 bytes) --------------------------------------------------------

/// Byte 0:    use_names_bin (u8, always 0x00)
/// Bytes 1-4: file_indicator (u32 LE)
/// Bytes 5-8: platform_indicator (u32 LE)
/// Bytes 9-12: entry_count (u32 LE)
pub struct Header {
    pub use_names_bin: u8,
    pub file_indicator: u32,
    pub platform_indicator: u32,
    pub entry_count: u32,
}

#[derive(Debug, Display, Error)]
pub enum HeaderError {
    UseNamesBin,
    FileIndicator,
    PlatformIndicator,
    EntryCount,
}

impl Header {
    pub const BYTE_SIZE: usize = 13;

    pub fn parse(i: &mut &[u8]) -> Result<Self, HeaderError> {
        use HeaderError as E;
        let use_names_bin = take::<u8>(i).map_err(|_| E::UseNamesBin)?;
        let file_indicator = take::<u32>(i).map_err(|_| E::FileIndicator)?.to_le();
        let platform_indicator = take::<u32>(i).map_err(|_| E::PlatformIndicator)?.to_le();
        let entry_count = take::<u32>(i).map_err(|_| E::EntryCount)?.to_le();
        Ok(Self { use_names_bin, file_indicator, platform_indicator, entry_count })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.use_names_bin)?;
        put(out, &self.file_indicator.to_le())?;
        put(out, &self.platform_indicator.to_le())?;
        put(out, &self.entry_count.to_le())?;
        Ok(())
    }
}

// -- Name lookup table (12 bytes per entry) -----------------------------------

/// Each row maps a definition to its name in Names.Bin.
///
/// Bytes 0-3: definition_name_offset (u32 LE) — byte offset into Names.Bin
/// Bytes 4-7: file_name_offset (u32 LE) — byte offset into Names.Bin,
///            or 0xFFFFFFFF if defined outside Names.Bin (PC),
///            or 0x00000000 if enumerator is in Names.Bin (Xbox)
/// Bytes 8-11: counter (u32 LE) — per-type counter (frontend/script)
///             or sequential index (game)
pub struct NameLookup {
    pub definition_name_offset: u32,
    pub file_name_offset: u32,
    pub counter: u32,
}

#[derive(Debug, Display, Error)]
pub enum NameLookupError {
    DefinitionNameOffset,
    FileNameOffset,
    Counter,
}

impl NameLookup {
    pub const BYTE_SIZE: usize = 12;

    pub fn parse(i: &mut &[u8]) -> Result<Self, NameLookupError> {
        use NameLookupError as E;
        let definition_name_offset = take::<u32>(i).map_err(|_| E::DefinitionNameOffset)?.to_le();
        let file_name_offset = take::<u32>(i).map_err(|_| E::FileNameOffset)?.to_le();
        let counter = take::<u32>(i).map_err(|_| E::Counter)?.to_le();
        Ok(Self { definition_name_offset, file_name_offset, counter })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.definition_name_offset.to_le())?;
        put(out, &self.file_name_offset.to_le())?;
        put(out, &self.counter.to_le())?;
        Ok(())
    }
}

// -- Chunk table header (8 bytes) ---------------------------------------------

/// Bytes 0-3: chunk_count (u32 LE) — actual chunk count + 1
/// Bytes 4-7: reserved (u32 LE) — always 0
pub struct ChunkTableHeader {
    pub chunk_count: u32,
    pub reserved: u32,
}

#[derive(Debug, Display, Error)]
pub enum ChunkTableHeaderError {
    ChunkCount,
    Reserved,
}

impl ChunkTableHeader {
    pub const BYTE_SIZE: usize = 8;

    /// The stored count is always one more than the actual number of chunks.
    pub fn actual_chunk_count(&self) -> u32 {
        self.chunk_count.saturating_sub(1)
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, ChunkTableHeaderError> {
        use ChunkTableHeaderError as E;
        let chunk_count = take::<u32>(i).map_err(|_| E::ChunkCount)?.to_le();
        let reserved = take::<u32>(i).map_err(|_| E::Reserved)?.to_le();
        Ok(Self { chunk_count, reserved })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.chunk_count.to_le())?;
        put(out, &self.reserved.to_le())?;
        Ok(())
    }
}

// -- Chunk table rows (8 bytes each) ------------------------------------------

/// Bytes 0-3: compressed_offset (u32 LE) — offset from end of chunk table
///            to start of this chunk's compressed data
/// Bytes 4-7: cumulative_entry_count (u32 LE) — running total of entries
///            through this chunk
pub struct ChunkTableRow {
    pub compressed_offset: u32,
    pub cumulative_entry_count: u32,
}

#[derive(Debug, Display, Error)]
pub enum ChunkTableRowError {
    CompressedOffset,
    CumulativeEntryCount,
}

impl ChunkTableRow {
    pub const BYTE_SIZE: usize = 8;

    pub fn parse(i: &mut &[u8]) -> Result<Self, ChunkTableRowError> {
        use ChunkTableRowError as E;
        let compressed_offset = take::<u32>(i).map_err(|_| E::CompressedOffset)?.to_le();
        let cumulative_entry_count = take::<u32>(i).map_err(|_| E::CumulativeEntryCount)?.to_le();
        Ok(Self { compressed_offset, cumulative_entry_count })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.compressed_offset.to_le())?;
        put(out, &self.cumulative_entry_count.to_le())?;
        Ok(())
    }
}

// -- Decompressed chunk entry offsets -----------------------------------------

/// Within a decompressed chunk, entry data is located via a u16 offset table.
/// The first `entry_count * 2` bytes are u16 LE offsets pointing to each entry's
/// data within the chunk. Dividing the first offset by 2 gives the entry count.
pub struct ChunkEntryOffset {
    pub offset: u16,
}

impl ChunkEntryOffset {
    pub const BYTE_SIZE: usize = 2;

    pub fn parse(i: &mut &[u8]) -> Result<Self, TakeError> {
        let offset = take::<u16>(i)?.to_le();
        Ok(Self { offset })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.offset.to_le())?;
        Ok(())
    }
}

// -- Top-level parse/serialize ------------------------------------------------

#[derive(Debug, Display, Error, From)]
pub enum DefBinError {
    Header(HeaderError),
    NameLookup(NameLookupError),
    ChunkTableHeader(ChunkTableHeaderError),
    ChunkTableRow(ChunkTableRowError),
    #[display("Unexpected end of input")]
    UnexpectedEnd,
}

impl From<UnexpectedEnd> for DefBinError {
    fn from(_: UnexpectedEnd) -> Self {
        DefBinError::UnexpectedEnd
    }
}

impl DefBin {
    pub fn parse(i: &mut &[u8]) -> Result<Self, DefBinError> {
        let header = Header::parse(i)?;

        let entry_count = header.entry_count as usize;
        let mut name_lookups = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            name_lookups.push(NameLookup::parse(i)?);
        }

        let chunk_table_header = ChunkTableHeader::parse(i)?;

        let actual_chunks = chunk_table_header.actual_chunk_count() as usize;
        let mut chunk_table = Vec::with_capacity(actual_chunks);
        for _ in 0..actual_chunks {
            chunk_table.push(ChunkTableRow::parse(i)?);
        }

        let chunks_data = i.to_vec();

        Ok(Self {
            header,
            name_lookups,
            chunk_table_header,
            chunk_table,
            chunks_data,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, UnexpectedEnd> {
        let entry_count = self.name_lookups.len();
        let chunk_count = self.chunk_table.len();

        let size = Header::BYTE_SIZE
            + entry_count * NameLookup::BYTE_SIZE
            + ChunkTableHeader::BYTE_SIZE
            + chunk_count * ChunkTableRow::BYTE_SIZE
            + self.chunks_data.len();

        let mut buf = vec![0u8; size];
        let mut out = buf.as_mut_slice();

        self.header.serialize(&mut out)?;

        for lookup in &self.name_lookups {
            lookup.serialize(&mut out)?;
        }

        self.chunk_table_header.serialize(&mut out)?;

        for row in &self.chunk_table {
            row.serialize(&mut out)?;
        }

        put_bytes(&mut out, &self.chunks_data)?;

        Ok(buf)
    }
}
