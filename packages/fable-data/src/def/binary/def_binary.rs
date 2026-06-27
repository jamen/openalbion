use crate::{
    bytes::{TakeError, UnexpectedEnd, put, put_bytes, take, take_bytes},
    def::{
        ConfigOptionsDefaultsDef, ControlsDef, EngineDef, EngineVideoOptionsDef, EnvironmentDef,
        EnvironmentThemeDaySetDef, FrontEndDef, UiDef, UiIconsDef, UiMiscThingsDef,
        binary::{
            control::{ParseControlError, SerializeControlError, SerializeControlErrorReason},
            names::{Names, NamesEntry},
        },
    },
};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

#[derive(Debug)]
pub struct DefBinary {
    pub header: DefBinaryHeader,
    pub name_refs: Vec<NameRef>,
    pub chunk_index: ChunkIndex,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug)]
pub enum LoadError<'a> {
    Open(io::Error),
    FromReader(FromReaderError<'a>),
}

impl DefBinary {
    pub fn load_with_names<'a>(path: &Path, names: &'a Names) -> Result<Self, LoadError<'a>> {
        use LoadError as E;
        let file = File::open(path).map_err(E::Open)?;
        let reader = BufReader::new(file);
        Self::from_reader_with_names(reader, names).map_err(E::FromReader)
    }
}

#[derive(Debug)]
pub enum FromReaderError<'a> {
    Read(io::Error),
    FromBytes(FromBytesError<'a>),
}

impl DefBinary {
    pub fn from_reader_with_names<'a, R: Read>(
        mut reader: R,
        names: &'a Names,
    ) -> Result<Self, FromReaderError<'a>> {
        use FromReaderError as E;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).map_err(E::Read)?;
        Self::from_bytes_with_names(&buf, &names).map_err(E::FromBytes)
    }
}

#[derive(Debug)]
pub enum FromBytesError<'a> {
    ParseHeader(ParseHeaderError),
    NameRefs(ParseNameRefListError),
    ParseChunkIndex(ParseChunkIndexError),
    ParseChunks(ParseChunkListError<'a>),
}

impl DefBinary {
    pub fn from_bytes_with_names<'a>(
        bytes: &[u8],
        names: &Names,
    ) -> Result<Self, FromBytesError<'a>> {
        use FromBytesError as E;

        let bytes_cursor = &mut &bytes[..];

        let header = DefBinaryHeader::parse(bytes_cursor).map_err(E::ParseHeader)?;

        let name_refs =
            NameRef::parse_list(bytes_cursor, header.entry_count).map_err(E::NameRefs)?;

        let chunk_index = ChunkIndex::parse(bytes_cursor).map_err(E::ParseChunkIndex)?;

        let chunks = Chunk::parse_list(bytes_cursor, &chunk_index, &name_refs, &names)
            .map_err(E::ParseChunks)?;

        Ok(Self {
            header,
            name_refs,
            chunk_index,
            chunks,
        })
    }
}

#[derive(Debug)]
pub struct DefBinaryHeader {
    pub use_names_bin: bool,
    pub file_indicator: u32,
    pub platform_indicator: u32,
    pub entry_count: u32,
}

#[derive(Debug)]
pub enum ParseHeaderError {
    UseNamesBin(TakeError),
    FileIndicator(TakeError),
    PlatformIndicator(TakeError),
    EntryCount(TakeError),
}

impl DefBinaryHeader {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseHeaderError> {
        use ParseHeaderError as E;
        let use_names_bin = take::<u8>(cur).map_err(E::UseNamesBin)? == 0x1;
        let file_indicator = take::<u32>(cur).map_err(E::FileIndicator)?.to_le();
        let platform_indicator = take::<u32>(cur).map_err(E::PlatformIndicator)?.to_le();
        let entry_count = take::<u32>(cur).map_err(E::EntryCount)?.to_le();
        Ok(Self {
            use_names_bin,
            file_indicator,
            platform_indicator,
            entry_count,
        })
    }
}

#[derive(Debug)]
pub enum SerializeDefBinaryHeaderError {
    UseNamesBin(UnexpectedEnd),
    FileIndicator(UnexpectedEnd),
    PlatformIndicator(UnexpectedEnd),
    EntryCount(UnexpectedEnd),
}

impl DefBinaryHeader {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeDefBinaryHeaderError> {
        use SerializeDefBinaryHeaderError as E;
        put(out, &(self.use_names_bin as u8)).map_err(E::UseNamesBin)?;
        put(out, &self.file_indicator).map_err(E::FileIndicator)?;
        put(out, &self.platform_indicator).map_err(E::PlatformIndicator)?;
        put(out, &self.entry_count).map_err(E::EntryCount)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NameRef {
    pub def_name_offset: u32,
    pub file_name_offset: u32,
    pub counter: u32,
}

#[derive(Debug)]
pub enum ParseNameRefListError {
    Entry(u32, ParseNameRefError),
}

impl NameRef {
    fn parse_list(cur: &mut &[u8], count: u32) -> Result<Vec<NameRef>, ParseNameRefListError> {
        (0..count)
            .map(|i| Self::parse(cur).map_err(|error| ParseNameRefListError::Entry(i, error)))
            .collect()
    }
}

#[derive(Debug)]
pub enum ParseNameRefError {
    DefNameOffset(TakeError),
    FileNameOffset(TakeError),
    Counter(TakeError),
}

impl NameRef {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseNameRefError> {
        use ParseNameRefError as E;
        let def_name_offset = take::<u32>(cur).map_err(E::DefNameOffset)?.to_le();
        let file_name_offset = take::<u32>(cur).map_err(E::FileNameOffset)?.to_le();
        let counter = take::<u32>(cur).map_err(E::Counter)?.to_le();
        Ok(Self {
            def_name_offset,
            file_name_offset,
            counter,
        })
    }
}

#[derive(Debug)]
pub enum SerializeNameRefError {
    DefNameOffset(UnexpectedEnd),
    FileNameOffset(UnexpectedEnd),
    Counter(UnexpectedEnd),
}

impl NameRef {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeNameRefError> {
        use SerializeNameRefError as E;
        put(out, &self.def_name_offset).map_err(E::DefNameOffset)?;
        put(out, &self.file_name_offset).map_err(E::FileNameOffset)?;
        put(out, &self.counter).map_err(E::Counter)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChunkIndex {
    pub header: ChunkIndexHeader,
    pub entries: Vec<ChunkIndexEntry>,
}

#[derive(Debug)]
pub enum ParseChunkIndexError {
    ParseChunkIndexHeader(ParseChunkIndexHeaderError),
    ParseChunkIndexEntry(ParseChunkIndexEntryError),
    ParseChunkIndexSentinel(ParseChunkIndexSentinelError),
}

impl ChunkIndex {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseChunkIndexError> {
        use ParseChunkIndexError as E;

        // Parse chunk table header

        let header = ChunkIndexHeader::parse(cur).map_err(E::ParseChunkIndexHeader)?;
        let mut entries = Vec::new();

        // Parse entries

        for _ in 0..(header.chunk_count - 1) {
            let entry = ChunkIndexEntry::parse(cur).map_err(E::ParseChunkIndexEntry)?;

            entries.push(entry);
        }

        // Parse optional sentinel entry

        let _sentinel = ChunkIndexEntry::parse_sentinel(cur).map_err(E::ParseChunkIndexSentinel)?;

        Ok(Self { header, entries })
    }
}

#[derive(Debug)]
pub struct ChunkIndexHeader {
    pub chunk_count: u32,
    pub reserved: u32,
}

#[derive(Debug)]
pub enum ParseChunkIndexHeaderError {
    ChunkCount(TakeError),
    Reserved(TakeError),
}

impl ChunkIndexHeader {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseChunkIndexHeaderError> {
        use ParseChunkIndexHeaderError as E;
        let chunk_count = take::<u32>(cur).map_err(E::ChunkCount)?.to_le();
        let reserved = take::<u32>(cur).map_err(E::Reserved)?.to_le();
        Ok(Self {
            chunk_count,
            reserved,
        })
    }
}

#[derive(Debug)]
pub enum SerializeChunkIndexHeaderError {
    ChunkCount(UnexpectedEnd),
    Reserved(UnexpectedEnd),
}

impl ChunkIndexHeader {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeChunkIndexHeaderError> {
        use SerializeChunkIndexHeaderError as E;
        put(out, &self.chunk_count).map_err(E::ChunkCount)?;
        put(out, &self.reserved).map_err(E::Reserved)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChunkIndexEntry {
    pub compressed_offset: u32,
    pub cumulative_entry_count: u32,
}

#[derive(Debug)]
pub enum ParseChunkIndexEntryError {
    CompressedOffset(TakeError),
    CumulativeEntryCount(TakeError),
}

impl ChunkIndexEntry {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseChunkIndexEntryError> {
        use ParseChunkIndexEntryError as E;
        let compressed_offset = take::<u32>(cur).map_err(E::CompressedOffset)?.to_le();
        let cumulative_entry_count = take::<u32>(cur).map_err(E::CumulativeEntryCount)?.to_le();
        Ok(Self {
            compressed_offset,
            cumulative_entry_count,
        })
    }
}

#[derive(Debug)]
pub enum SerializeChunkIndexEntryError {
    CompressedOffset(UnexpectedEnd),
    CumulativeEntryCount(UnexpectedEnd),
}

impl ChunkIndexEntry {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeChunkIndexEntryError> {
        use SerializeChunkIndexEntryError as E;
        put(out, &self.compressed_offset).map_err(E::CompressedOffset)?;
        put(out, &self.cumulative_entry_count).map_err(E::CumulativeEntryCount)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParseChunkIndexSentinelError {
    ParseChunkIndexEntry(ParseChunkIndexEntryError),
}

impl ChunkIndexEntry {
    fn parse_sentinel(cur: &mut &[u8]) -> Result<Option<Self>, ParseChunkIndexSentinelError> {
        use ParseChunkIndexSentinelError as E;

        let input_copied = &mut *cur;

        let sentinel_entry =
            ChunkIndexEntry::parse(input_copied).map_err(E::ParseChunkIndexEntry)?;

        let sentinel_entry = if sentinel_entry.compressed_offset
            == sentinel_entry.cumulative_entry_count
            && sentinel_entry.compressed_offset == input_copied.len() as u32
        {
            *cur = *input_copied;
            Some(sentinel_entry)
        } else {
            None
        };

        Ok(sentinel_entry)
    }
}

#[derive(Debug)]
pub enum ParseChunkListError<'a> {
    ParseChunk(ParseChunkError<'a>),
}

impl Chunk {
    fn parse_list<'a>(
        cur: &mut &[u8],
        chunk_index: &ChunkIndex,
        name_refs: &[NameRef],
        names: &Names,
    ) -> Result<Vec<Self>, ParseChunkListError<'a>> {
        use ParseChunkListError as E;

        let mut list = Vec::new();
        let mut chunk_entry_base = 0;

        for (i, entry) in chunk_index.entries.iter().enumerate() {
            let start = entry.compressed_offset;

            let end = chunk_index
                .entries
                .get(i + 1)
                .map(|x| x.compressed_offset)
                .unwrap_or(cur.len() as u32);

            let mut compressed_data = &cur[start as usize..end as usize];

            let compressed_data_cursor = &mut compressed_data;

            let chunk_entry_count = entry.cumulative_entry_count - chunk_entry_base;

            let chunk = Chunk::parse(
                compressed_data_cursor,
                chunk_entry_base,
                chunk_entry_count,
                &name_refs,
                &names,
            )
            .map_err(E::ParseChunk)?;

            list.push(chunk);

            chunk_entry_base = entry.cumulative_entry_count;
        }

        Ok(list)
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub entry_base: u32,
    pub entry_count: u32,
    pub entries: Vec<EntryRecord>,
}

#[derive(Debug)]
pub enum ParseChunkError<'a> {
    MinizOxideDecompress(miniz_oxide::inflate::DecompressError),
    ParseEntries(ParseEntryRecordListError<'a>),
    TrailingBytes {
        base: u32,
        count: u32,
        remaining: usize,
    },
}

const MAX_CHUNK_DECOMPRESS_SIZE: usize = 32768; // 32KiB, just a guess for now

impl Chunk {
    fn parse<'a>(
        cur: &mut &[u8],
        entry_base: u32,
        entry_count: u32,
        name_refs: &[NameRef],
        names: &Names,
    ) -> Result<Self, ParseChunkError<'a>> {
        use ParseChunkError as E;

        let decompressed_bytes = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(
            *cur,
            MAX_CHUNK_DECOMPRESS_SIZE,
        )
        .map_err(E::MinizOxideDecompress)?;

        let decompressed_bytes_cursor = &mut &decompressed_bytes[..];

        let entries = EntryRecord::parse_list(
            decompressed_bytes_cursor,
            entry_base,
            entry_count,
            name_refs,
            names,
        )
        .map_err(E::ParseEntries)?;

        if !decompressed_bytes_cursor.is_empty() {
            return Err(E::TrailingBytes {
                base: entry_base,
                count: entry_count,
                remaining: decompressed_bytes_cursor.len(),
            });
        }

        Ok(Self {
            entry_base,
            entry_count,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct EntryRecord {
    pub preamble: EntryPreamble,
    /// "Instance" defs carry a 2-byte prefix (always `0x0000` observed) between
    /// the preamble and the control body; class templates don't. Preserved here
    /// so the entry round-trips.
    pub instance_prefix: Option<[u8; 2]>,
    pub chunk_start: usize,
    pub chunk_end: usize,
    pub body: DefBody,
    pub raw_bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum ParseEntryRecordListError<'a> {
    Offset(TakeError),
    EntryBytes(UnexpectedEnd),
    EntryRecord(u32, ParseEntryRecordError<'a>),
    NoNameEntry {
        position: u32,
        name_ref: NameRef,
    },
    TrailingBytes {
        position: u32,
        remaining: usize,
        name: String,
    },
    InvalidOffset {
        start: usize,
        end: usize,
        expected_start: usize,
    },
}

impl EntryRecord {
    pub fn parse_list<'a>(
        cur: &mut &[u8],
        chunk_entry_base: u32,
        chunk_entry_count: u32,
        name_refs: &[NameRef],
        names: &Names,
    ) -> Result<Vec<Self>, ParseEntryRecordListError<'a>> {
        use ParseEntryRecordListError as E;

        let original_cur = &mut &cur[..];

        let mut offsets = Vec::new();

        for _ in 0..chunk_entry_count {
            let offset = take::<u16>(cur).map_err(E::Offset)?.to_le();
            offsets.push(offset)
        }

        let payload_base = chunk_entry_count as usize * 2;

        let mut expected_chunk_start = payload_base;

        let mut entries = Vec::new();

        for i in 0..chunk_entry_count {
            let chunk_start = offsets[i as usize] as usize;

            let chunk_end = offsets
                .get(i as usize + 1)
                .map(|&x| x as usize)
                .unwrap_or(original_cur.len());

            if chunk_start != expected_chunk_start {
                return Err(E::InvalidOffset {
                    start: chunk_start,
                    end: chunk_end,
                    expected_start: expected_chunk_start,
                });
            }

            expected_chunk_start = chunk_end;

            let entry_len = chunk_end - chunk_start;

            let entry_position = chunk_entry_base + i;
            let entry_name_ref = &name_refs[entry_position as usize];
            let entry_name = &names.map.get(&entry_name_ref.def_name_offset);

            match entry_name {
                Some(entry_name) => {
                    let mut entry_bytes = take_bytes(cur, entry_len).map_err(E::EntryBytes)?;

                    let entry_bytes_cursor = &mut entry_bytes;

                    let entry_record =
                        EntryRecord::parse(entry_bytes_cursor, &entry_name, chunk_start, chunk_end)
                            .map_err(|error| E::EntryRecord(entry_position, error))?;

                    if !entry_bytes_cursor.is_empty() {
                        return Err(E::TrailingBytes {
                            position: entry_position,
                            remaining: entry_bytes_cursor.len(),
                            name: entry_name.string.clone(),
                        });
                    }

                    entries.push(entry_record);
                }
                None => {
                    return Err(E::NoNameEntry {
                        position: entry_position,
                        name_ref: entry_name_ref.clone(),
                    });
                }
            }
        }

        Ok(entries)
    }
}

#[derive(Debug)]
pub enum ParseEntryRecordError<'a> {
    Preamble(ParseEntryPreambleError),
    DefBody((&'a str, ParseControlError)),
}

impl EntryRecord {
    fn parse<'a>(
        cur: &mut &[u8],
        name: &NamesEntry,
        chunk_start: usize,
        chunk_end: usize,
    ) -> Result<Self, ParseEntryRecordError<'a>> {
        use ParseEntryRecordError as E;

        let raw_bytes = cur.to_vec();

        let preamble = EntryPreamble::parse(cur).map_err(E::Preamble)?;

        // The body is a sequence of `(crc_id, value)` controls. "Instance" defs
        // carry a 2-byte prefix before the first control that templates lack,
        // and it isn't flagged anywhere in the header. Since `parse_id`
        // validates each id against `crc(field_name)`, a misaligned parse fails
        // loudly — so try the body as-is, and on failure retry past a 2-byte
        // prefix. (Unknown def types swallow their whole body and never reach
        // the retry; their prefix is preserved in the raw bytes.)
        // Full body bytes, used for the `Unknown` fallback below.
        let body_bytes = cur.to_vec();

        let mut attempt = *cur;
        let typed = match DefBody::parse(&mut attempt, &name.string) {
            Ok(body) => {
                *cur = attempt;
                Some((None, body))
            }
            // When the as-is parse fails and a 2-byte instance prefix is present,
            // it's an instance def — retry past the prefix.
            Err(_) if cur.starts_with(&[0x00, 0x00]) => {
                let mut skipped = &cur[2..];
                match DefBody::parse(&mut skipped, &name.string) {
                    Ok(body) => {
                        *cur = skipped;
                        Some((Some([0x00, 0x00]), body))
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        };

        // A def type we model but whose retail layout we don't match exactly
        // falls back to raw bytes rather than aborting the whole file. (Truly
        // unknown def types already parse as `DefBody::Unknown` above.)
        let (instance_prefix, body) = match typed {
            Some(parsed) => parsed,
            None => {
                *cur = &cur[cur.len()..];
                (
                    None,
                    DefBody::Unknown {
                        name: name.string.clone(),
                        bytes: body_bytes,
                    },
                )
            }
        };

        Ok(Self {
            chunk_start,
            chunk_end,
            raw_bytes,
            preamble,
            instance_prefix,
            body,
        })
    }
}

#[derive(Debug)]
pub enum SerializeEntryRecordError {
    Preamble(SerializeEntryPreambleError),
    InstancePrefix(UnexpectedEnd),
    Body((&'static str, SerializeControlError)),
}

impl EntryRecord {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeEntryRecordError> {
        use SerializeEntryRecordError as E;
        self.preamble.serialize(out).map_err(E::Preamble)?;
        if let Some(prefix) = &self.instance_prefix {
            put_bytes(out, prefix).map_err(E::InstancePrefix)?;
        }
        self.body.serialize(out).map_err(E::Body)?;
        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        EntryPreamble::BYTE_SIZE
            + self.instance_prefix.map_or(0, |p| p.len())
            + self.body.byte_size()
    }

    pub fn payload_size(&self) -> usize {
        self.chunk_end - self.chunk_start
    }
}

#[derive(Debug)]
pub enum DefBody {
    Engine(EngineDef),
    Controls(ControlsDef),
    FrontEnd(FrontEndDef),
    Ui(UiDef),
    UiIcons(UiIconsDef),
    UiMiscThings(UiMiscThingsDef),
    EngineVideoOptions(EngineVideoOptionsDef),
    ConfigOptionsDefaults(ConfigOptionsDefaultsDef),
    Environment(EnvironmentDef),
    EnvironmentThemeDaySet(EnvironmentThemeDaySetDef),
    Unknown { name: String, bytes: Vec<u8> },
}

impl DefBody {
    pub(crate) fn parse<'a>(
        cur: &mut &[u8],
        name: &'a str,
    ) -> Result<Self, (&'a str, ParseControlError)> {
        Ok(match name {
            "ENGINE" => DefBody::Engine(EngineDef::parse(cur).map_err(|e| (name, e))?),
            "CONTROL_SCHEME" => DefBody::Controls(ControlsDef::parse(cur).map_err(|e| (name, e))?),
            "FRONT_END" => DefBody::FrontEnd(FrontEndDef::parse(cur).map_err(|e| (name, e))?),
            "UI" => DefBody::Ui(UiDef::parse(cur).map_err(|e| (name, e))?),
            "UI_ICONS_DEF" => DefBody::UiIcons(UiIconsDef::parse(cur).map_err(|e| (name, e))?),
            "UI_MISC_THINGS_DEF" => {
                DefBody::UiMiscThings(UiMiscThingsDef::parse(cur).map_err(|e| (name, e))?)
            }
            "ENGINE_VIDEO_OPTIONS" => DefBody::EngineVideoOptions(
                EngineVideoOptionsDef::parse(cur).map_err(|e| (name, e))?,
            ),
            "CONFIG_OPTIONS_DEFAULTS_DEF" => DefBody::ConfigOptionsDefaults(
                ConfigOptionsDefaultsDef::parse(cur).map_err(|e| (name, e))?,
            ),
            "CENVIRONMENT_DEF" | "ENVIRONMENT" => {
                DefBody::Environment(EnvironmentDef::parse(cur).map_err(|e| (name, e))?)
            }
            "CENVIRONMENT_THEME_DAY" | "ENVIRONMENT_THEME_DAY" => DefBody::EnvironmentThemeDaySet(
                EnvironmentThemeDaySetDef::parse(cur).map_err(|e| (name, e))?,
            ),
            _ => DefBody::Unknown {
                name: name.to_string(),
                bytes: core::mem::take(cur).to_vec(),
            },
        })
    }

    pub fn serialize(
        &self,
        out: &mut &mut [u8],
    ) -> Result<(), (&'static str, SerializeControlError)> {
        use DefBody as D;
        match self {
            D::Engine(d) => d.serialize(out).map_err(|e| ("ENGINE", e))?,
            D::Controls(d) => d.serialize(out).map_err(|e| ("CONTROL_SCHEME", e))?,
            D::FrontEnd(d) => d.serialize(out).map_err(|e| ("FRONT_END", e))?,
            D::Ui(d) => d.serialize(out).map_err(|e| ("UI", e))?,
            D::UiIcons(d) => d.serialize(out).map_err(|e| ("UI_ICONS_DEF", e))?,
            D::UiMiscThings(d) => d.serialize(out).map_err(|e| ("UI_MISC_THINGS_DEF", e))?,
            D::EngineVideoOptions(d) => {
                d.serialize(out).map_err(|e| ("ENGINE_VIDEO_OPTIONS", e))?
            }
            D::ConfigOptionsDefaults(d) => d
                .serialize(out)
                .map_err(|e| ("CONFIG_OPTIONS_DEFAULTS_DEF", e))?,
            D::Environment(d) => d.serialize(out).map_err(|e| ("ENVIRONMENT", e))?,
            D::EnvironmentThemeDaySet(d) => {
                d.serialize(out).map_err(|e| ("ENVIRONMENT_THEME_DAY", e))?
            }
            D::Unknown { .. } => {
                return Err((
                    "UNKNOWN",
                    SerializeControlError {
                        name: "<none>",
                        reason: SerializeControlErrorReason::Value(UnexpectedEnd),
                    },
                ));
            }
        }
        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        use DefBody as D;
        match self {
            D::Engine(d) => d.byte_size(),
            D::Controls(d) => d.byte_size(),
            D::FrontEnd(d) => d.byte_size(),
            D::Ui(d) => d.byte_size(),
            D::UiIcons(d) => d.byte_size(),
            D::UiMiscThings(d) => d.byte_size(),
            D::EngineVideoOptions(d) => d.byte_size(),
            D::ConfigOptionsDefaults(d) => d.byte_size(),
            D::Environment(d) => d.byte_size(),
            D::EnvironmentThemeDaySet(d) => d.byte_size(),
            D::Unknown { bytes, .. } => bytes.len(),
        }
    }
}

/// 3-byte record preamble that precedes each def body. Verified against retail
/// `game.bin`: bodies are `(u32 id, u32 value)` control pairs starting at byte
/// 3, and a body-less entry (e.g. a `CHeroCentreDef` template) is exactly these
/// 3 bytes.
#[derive(Debug)]
pub struct EntryPreamble {
    pub is_real: bool,
    pub is_template: bool,
    pub unknown_0: u8,
}

#[derive(Debug)]
pub enum ParseEntryPreambleError {
    IsReal(TakeError),
    IsTemplate(TakeError),
    Unknown0(TakeError),
}

impl EntryPreamble {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseEntryPreambleError> {
        use ParseEntryPreambleError as E;
        let is_real = take::<u8>(cur).map_err(E::IsReal)? == 0x1;
        let is_template = take::<u8>(cur).map_err(E::IsTemplate)? == 0x1;
        let unknown_0 = take::<u8>(cur).map_err(E::Unknown0)?;
        Ok(Self {
            is_real,
            is_template,
            unknown_0,
        })
    }
}

#[derive(Debug)]
pub enum SerializeEntryPreambleError {
    IsReal(UnexpectedEnd),
    IsTemplate(UnexpectedEnd),
    Unknown0(UnexpectedEnd),
}

impl EntryPreamble {
    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), SerializeEntryPreambleError> {
        use SerializeEntryPreambleError as E;
        put(out, &(self.is_real as u8)).map_err(E::IsReal)?;
        put(out, &(self.is_template as u8)).map_err(E::IsTemplate)?;
        put(out, &self.unknown_0).map_err(E::Unknown0)?;
        Ok(())
    }

    pub const BYTE_SIZE: usize = 3;
}

#[derive(Debug)]
pub struct DefBinaryEntry<'a> {
    pub global_index: usize,
    pub chunk_index: usize,
    pub chunk_local_index: usize,

    pub name_ref: &'a NameRef,
    pub def_name: Option<&'a str>,
    pub file_name: Option<&'a str>,

    pub record: &'a EntryRecord,
}

impl DefBinary {
    pub fn entries<'a>(
        &'a self,
        names: &'a Names,
    ) -> impl Iterator<Item = DefBinaryEntry<'a>> + 'a {
        self.chunks
            .iter()
            .enumerate()
            .flat_map(move |(chunk_index, chunk)| {
                chunk
                    .entries
                    .iter()
                    .enumerate()
                    .map(move |(chunk_local_index, record)| {
                        let global_index = chunk.entry_base as usize + chunk_local_index;
                        let name_ref = &self.name_refs[global_index];

                        let def_name = names
                            .map
                            .get(&name_ref.def_name_offset)
                            .map(|x| x.string.as_str());

                        let file_name = names
                            .map
                            .get(&name_ref.file_name_offset)
                            .map(|x| x.string.as_str());

                        DefBinaryEntry {
                            global_index,
                            chunk_index,
                            chunk_local_index,
                            name_ref,
                            def_name,
                            file_name,
                            record,
                        }
                    })
            })
    }
}
