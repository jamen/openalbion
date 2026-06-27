use crate::bytes::{put, put_bytes, take, take_bytes};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Read, Seek, SeekFrom, Write},
    mem,
};

pub struct WadReader<T: Read + Seek> {
    source: T,
    header: Header,
    assets: HashMap<String, AssetMetadata>,
}

#[derive(Debug, Display, Error)]
pub enum WadReaderError {
    SeekToHeader(io::Error),
    ReadHeader(io::Error),
    ParseHeader(HeaderError),
    ReadAssetTable(ReadAssetTableError),
}

impl<T: Read + Seek> WadReader<T> {
    pub fn new(mut source: T) -> Result<Self, WadReaderError> {
        use WadReaderError as E;

        let mut header_bytes = [0; Header::BYTE_SIZE];

        source.seek(SeekFrom::Start(0)).map_err(E::SeekToHeader)?;
        source
            .read_exact(&mut header_bytes)
            .map_err(E::ReadHeader)?;

        let header = Header::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)?;

        let assets = Self::read_asset_table(&mut source, &header).map_err(E::ReadAssetTable)?;

        Ok(Self {
            source,
            header,
            assets,
        })
    }
}

#[derive(Debug, Display, Error)]
pub enum ReadAssetTableError {
    Seek(io::Error),
    Read(io::Error),
    #[display("asset metadata ${_0}: ${_1}")]
    ParseAssetMetadata(u32, AssetMetadataError),
}

impl<T: Read + Seek> WadReader<T> {
    fn read_asset_table(
        source: &mut T,
        header: &Header,
    ) -> Result<HashMap<String, AssetMetadata>, ReadAssetTableError> {
        use ReadAssetTableError as E;

        let pos = SeekFrom::Start(header.asset_table_position as u64);

        let mut table_bytes = Vec::new();

        source.seek(pos).map_err(E::Seek)?;

        source.read_to_end(&mut table_bytes).map_err(E::Read)?;

        let mut table_bytes = &table_bytes[..];

        let mut table = HashMap::with_capacity(header.asset_count as usize);

        for i in 0..header.asset_count {
            let metadata = AssetMetadataRef::parse(&mut table_bytes)
                .map_err(|e| E::ParseAssetMetadata(i, e))?
                .into_owned();

            table.insert(metadata.path.to_string(), metadata);
        }

        Ok(table)
    }
}

#[derive(Debug, Display, Error)]
pub enum ReadContentError {
    Seek(io::Error),
    Read(io::Error),
}

impl<T: Read + Seek> WadReader<T> {
    /// The parsed header of this wad file.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Iterate over the metadata of every asset in the wad's asset table.
    ///
    /// Entries are keyed by their backslash-delimited in-game path (e.g.
    /// `"Data\Levels\FinalAlbion\FinalAlbion.lev"`).
    pub fn asset_iter(&self) -> impl Iterator<Item = &AssetMetadata> {
        self.assets.values()
    }

    /// Look up a single asset's metadata by its full in-game path.
    pub fn asset(&self, path: &str) -> Option<&AssetMetadata> {
        self.assets.get(path)
    }

    /// Read the raw file content for an asset.
    pub fn read_content(
        &mut self,
        asset: &AssetMetadata,
    ) -> Result<Vec<u8>, ReadContentError> {
        use ReadContentError as E;

        let mut content = vec![0u8; asset.content_length as usize];
        let pos = SeekFrom::Start(asset.content_position as u64);
        self.source.seek(pos).map_err(E::Seek)?;
        self.source.read_exact(&mut content).map_err(E::Read)?;

        Ok(content)
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: [u32; 3],
    pub block_size: u32,
    pub asset_count: u32,
    pub asset_count_2: u32,
    pub asset_table_position: u32,
}

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum HeaderError {
    Magic,
    Version,
    BlockSize,
    AssetCount,
    AssetCount2,
    AssetTablePosition,
}

impl Header {
    pub const BYTE_SIZE: usize = 32;

    /// File signature found at the start of every `.wad` file.
    pub const MAGIC: [u8; 4] = *b"BBBB";

    pub fn parse(inp: &mut &[u8]) -> Result<Self, HeaderError> {
        use HeaderError as E;

        let magic = take::<[u8; 4]>(inp).map_err(|_| E::Magic)?;
        let version = take::<[u32; 3]>(inp)
            .map_err(|_| E::Version)?
            .map(u32::to_le);
        let block_size = take::<u32>(inp).map_err(|_| E::BlockSize)?.to_le();
        let entries_count = take::<u32>(inp).map_err(|_| E::AssetCount)?.to_le();
        let entries_count_again = take::<u32>(inp).map_err(|_| E::AssetCount2)?.to_le();
        let entries_position = take::<u32>(inp).map_err(|_| E::AssetTablePosition)?.to_le();

        Ok(Header {
            magic,
            version,
            block_size,
            asset_count: entries_count,
            asset_count_2: entries_count_again,
            asset_table_position: entries_position,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), HeaderError> {
        use HeaderError as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.map(u32::to_le)).map_err(|_| E::Version)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;
        put(out, &self.asset_count.to_le()).map_err(|_| E::AssetCount)?;
        put(out, &self.asset_count_2.to_le()).map_err(|_| E::AssetCount2)?;
        put(out, &self.asset_table_position.to_le()).map_err(|_| E::AssetTablePosition)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetMetadataRef<'a> {
    pub unknown_1: [u8; 16],
    pub id: u32,
    pub unknown_2: u32,
    pub content_length: u32,
    pub content_position: u32,
    pub unknown_3: u32,
    pub path: Cow<'a, str>,
    pub unknown_4: [u8; 16],
    pub created: [u32; 7],
    pub accessed: [u32; 7],
    pub modified: [u32; 5],
}

pub type AssetMetadata = AssetMetadataRef<'static>;

#[derive(Copy, Clone, Debug, Display, Error)]
pub enum AssetMetadataError {
    Unknown1,
    Id,
    Unknown2,
    ContentPosition,
    ContentLength,
    Unknown3,
    PathLen,
    Path,
    PathToString,
    Unknown4,
    Created,
    Accessed,
    Modified,
}

impl<'a> AssetMetadataRef<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<AssetMetadataRef<'a>, AssetMetadataError> {
        use AssetMetadataError as E;

        let unknown_1 = take::<[u8; 16]>(inp).map_err(|_| E::Unknown1)?;
        let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
        let unknown_2 = take::<u32>(inp).map_err(|_| E::Unknown2)?.to_le();
        let data_length = take::<u32>(inp).map_err(|_| E::ContentLength)?.to_le();
        let data_position = take::<u32>(inp).map_err(|_| E::ContentPosition)?.to_le();
        let unknown_3 = take::<u32>(inp).map_err(|_| E::Unknown3)?.to_le();

        let path_len = usize::try_from(take::<u32>(inp).map_err(|_| E::PathLen)?.to_le())
            .map_err(|_| E::PathLen)?;

        let path = take_bytes(inp, path_len).map_err(|_| E::Path)?;
        let path = str::from_utf8(path).map_err(|_| E::PathToString)?;
        let path = Cow::from(path);

        let unknown_4 = take::<[u8; 16]>(inp).map_err(|_| E::Unknown4)?;

        let created = take::<[u32; 7]>(inp)
            .map_err(|_| E::Created)?
            .map(u32::to_le);
        let accessed = take::<[u32; 7]>(inp)
            .map_err(|_| E::Accessed)?
            .map(u32::to_le);
        let modified = take::<[u32; 5]>(inp)
            .map_err(|_| E::Modified)?
            .map(u32::to_le);

        Ok(AssetMetadataRef {
            unknown_1,
            id,
            unknown_2,
            content_length: data_length,
            content_position: data_position,
            unknown_3,
            path,
            unknown_4,
            created,
            accessed,
            modified,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), AssetMetadataError> {
        use AssetMetadataError as E;

        put(out, &self.unknown_1).map_err(|_| E::Unknown1)?;
        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.unknown_2.to_le()).map_err(|_| E::Unknown2)?;
        put(out, &self.content_length.to_le()).map_err(|_| E::ContentLength)?;
        put(out, &self.content_position.to_le()).map_err(|_| E::ContentPosition)?;
        put(out, &self.unknown_3.to_le()).map_err(|_| E::Unknown3)?;

        let path_len = u32::try_from(self.path.len()).map_err(|_| E::PathLen)?;

        put(out, &path_len.to_le()).map_err(|_| E::PathLen)?;

        put_bytes(out, &self.path.as_bytes()).map_err(|_| E::Path)?;

        put(out, &self.unknown_4).map_err(|_| E::Unknown4)?;
        put(out, &self.created.map(u32::to_le)).map_err(|_| E::Created)?;
        put(out, &self.accessed.map(u32::to_le)).map_err(|_| E::Accessed)?;
        put(out, &self.modified.map(u32::to_le)).map_err(|_| E::Modified)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        mem::size_of::<[u8; 16]>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + mem::size_of::<u32>()
            + self.path.len()
            + mem::size_of::<[u8; 16]>()
            + mem::size_of::<[u32; 7]>()
            + mem::size_of::<[u32; 7]>()
            + mem::size_of::<[u32; 5]>()
    }

    pub fn into_owned(self) -> AssetMetadata {
        AssetMetadata {
            unknown_1: self.unknown_1,
            id: self.id,
            unknown_2: self.unknown_2,
            content_length: self.content_length,
            content_position: self.content_position,
            unknown_3: self.unknown_3,
            path: Cow::Owned(self.path.into_owned()),
            unknown_4: self.unknown_4,
            created: self.created,
            accessed: self.accessed,
            modified: self.modified,
        }
    }
}

/// Writes a `.wad` archive.
///
/// A wad file is laid out as: a 32-byte [`Header`] (the rest of the first block is zero padding),
/// then each asset's content padded up to `block_size`, then the asset metadata table at the end.
///
/// Typical usage: seek the sink to `block_size`, [`write_content`](WadWriter::write_content) each
/// asset (tracking positions), then [`write_entries`](WadWriter::write_entries) and finally
/// [`write_header`](WadWriter::write_header) once the table position is known.
pub struct WadWriter<Sink: Write + Seek> {
    sink: Sink,
}

#[derive(Debug, Display, Error)]
pub enum WadWriterError {
    SerializeHeader(HeaderError),
    SeekHeader(io::Error),
    WriteHeader(io::Error),
    SerializeEntry(AssetMetadataError),
    SeekEntries(io::Error),
    WriteEntries(io::Error),
    WriteContent(io::Error),
}

impl<Sink: Write + Seek> WadWriter<Sink> {
    pub fn new(sink: Sink) -> Self {
        Self { sink }
    }

    pub fn into_inner(self) -> Sink {
        self.sink
    }

    /// Seek within the underlying sink (e.g. to the first content block).
    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.sink.seek(pos)
    }

    /// Write the header at the start of the file.
    pub fn write_header(&mut self, header: &Header) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut header_bytes = [0u8; Header::BYTE_SIZE];
        header
            .serialize(&mut &mut header_bytes[..])
            .map_err(E::SerializeHeader)?;

        self.sink.seek(SeekFrom::Start(0)).map_err(E::SeekHeader)?;
        self.sink.write_all(&header_bytes).map_err(E::WriteHeader)?;

        Ok(())
    }

    /// Write an asset's content at the current position, padded up to `block_size`.
    ///
    /// Returns the number of bytes written including padding.
    pub fn write_content(
        &mut self,
        content: &[u8],
        block_size: usize,
    ) -> Result<usize, WadWriterError> {
        use WadWriterError as E;

        let padding_amount = if block_size == 0 {
            0
        } else {
            (block_size - (content.len() % block_size)) % block_size
        };

        self.sink.write_all(content).map_err(E::WriteContent)?;

        if padding_amount > 0 {
            self.sink
                .write_all(&vec![0u8; padding_amount])
                .map_err(E::WriteContent)?;
        }

        Ok(content.len() + padding_amount)
    }

    /// Write the asset metadata table at the end of the file.
    pub fn write_entries(&mut self, entries: &[AssetMetadata]) -> Result<(), WadWriterError> {
        use WadWriterError as E;

        let mut entries_bytes = Vec::new();

        for entry in entries {
            let start = entries_bytes.len();
            entries_bytes.resize(start + entry.byte_size(), 0);
            let mut slice: &mut [u8] = &mut entries_bytes[start..];
            entry.serialize(&mut slice).map_err(E::SerializeEntry)?;
        }

        self.sink.seek(SeekFrom::End(0)).map_err(E::SeekEntries)?;
        self.sink
            .write_all(&entries_bytes)
            .map_err(E::WriteEntries)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn wad_pack_unpack_round_trip() {
        let block_size: usize = 2048;

        let files: &[(&str, &[u8])] = &[
            ("Data\\Levels\\Foo\\a.bin", b"hello world"),
            ("Data\\Levels\\Foo\\b.bin", &[0xAB; 5000]),
            ("Data\\Levels\\Foo\\empty.bin", b""),
        ];

        // Pack.
        let mut cursor = Cursor::new(Vec::<u8>::new());
        cursor.seek(SeekFrom::Start(block_size as u64)).unwrap();

        let mut writer = WadWriter::new(cursor);
        let mut entries = Vec::new();
        let mut position = block_size;

        for (id, (path, content)) in files.iter().enumerate() {
            let written = writer.write_content(content, block_size).unwrap();
            entries.push(AssetMetadata {
                unknown_1: [0; 16],
                id: id as u32,
                unknown_2: 0,
                content_length: content.len() as u32,
                content_position: position as u32,
                unknown_3: 0,
                path: Cow::Owned((*path).to_string()),
                unknown_4: [0; 16],
                created: [0; 7],
                accessed: [0; 7],
                modified: [0; 5],
            });
            position += written;
        }

        let asset_table_position = position as u32;
        writer.write_entries(&entries).unwrap();

        let header = Header {
            magic: Header::MAGIC,
            version: [1, 0, 0],
            block_size: block_size as u32,
            asset_count: files.len() as u32,
            asset_count_2: files.len() as u32,
            asset_table_position,
        };
        writer.write_header(&header).unwrap();

        let bytes = writer.into_inner().into_inner();

        // Unpack.
        let mut reader = WadReader::new(Cursor::new(bytes)).unwrap();

        assert_eq!(reader.header().magic, Header::MAGIC);
        assert_eq!(reader.header().asset_count, files.len() as u32);

        for (path, content) in files {
            let asset = reader.asset(path).expect("asset present").clone();
            let read_back = reader.read_content(&asset).unwrap();
            assert_eq!(&read_back[..], *content, "content mismatch for {path}");
        }
    }
}
