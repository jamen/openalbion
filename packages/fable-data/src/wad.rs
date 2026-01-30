use crate::bytes::{put, put_bytes, take, take_bytes};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    io::{self, Read, Seek, SeekFrom},
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
