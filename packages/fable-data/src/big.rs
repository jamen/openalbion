use crate::bytes::{
    TakeError, UnexpectedEnd, put, put_bytes, take, take_bytes, take_null_terminated_bytes,
};
use derive_more::{Display, Error};
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    io::{self, Read, Seek, SeekFrom},
};

pub struct BigReader<T: Read + Seek> {
    source: T,
    /// Parsed file header, retained for completeness though not yet read back.
    #[allow(dead_code)]
    header: Header,
    banks: HashMap<String, Bank>,
}

#[derive(Debug, Display, Error)]
pub enum BigReaderError {
    SeekToHeader(io::Error),
    ReadHeader(io::Error),
    ParseHeader(HeaderError),
    ReadBankTable(ReadBankTableError),
}

impl<T: Read + Seek> BigReader<T> {
    pub fn new(mut source: T) -> Result<Self, BigReaderError> {
        use BigReaderError as E;

        let mut header_bytes = [0; Header::BYTE_SIZE];

        source.seek(SeekFrom::Start(0)).map_err(E::SeekToHeader)?;
        source
            .read_exact(&mut header_bytes)
            .map_err(E::ReadHeader)?;

        let header = Header::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)?;

        let banks = Self::read_bank_table(&mut source, &header).map_err(E::ReadBankTable)?;

        Ok(Self {
            source,
            header,
            banks,
        })
    }

    pub fn bank_iter(&self) -> impl Iterator<Item = &Bank> {
        self.banks.values()
    }

    pub fn bank(&self, name: &str) -> Option<&Bank> {
        self.banks.get(name)
    }

    pub fn asset_metadata(&self, bank_name: &str, symbol_name: &str) -> Option<&AssetMetadata> {
        self.banks
            .get(bank_name)
            .and_then(|bank| bank.assets.get(symbol_name))
    }
}

#[derive(Debug, Display, Error)]
pub enum ReadAssetDataError {
    NotFound,
    Seek(io::Error),
    Read(io::Error),
}

impl<T: Read + Seek> BigReader<T> {
    /// Read the raw data for an asset from the archive.
    pub fn read_asset_from_metadata(
        &mut self,
        asset: &AssetMetadata,
    ) -> Result<Vec<u8>, ReadAssetDataError> {
        use ReadAssetDataError as E;

        let mut data = vec![0u8; asset.size as usize];
        let pos = SeekFrom::Start(asset.start as u64);
        self.source.seek(pos).map_err(E::Seek)?;
        self.source.read_exact(&mut data).map_err(E::Read)?;

        Ok(data)
    }

    pub fn read_asset(
        &mut self,
        bank_name: &str,
        symbol_name: &str,
    ) -> Result<(AssetMetadata, Vec<u8>), ReadAssetDataError> {
        use ReadAssetDataError as E;

        let metadata = self
            .asset_metadata(bank_name, symbol_name)
            .ok_or(E::NotFound)?
            .clone();

        let data = self.read_asset_from_metadata(&metadata)?;

        Ok((metadata, data))
    }
}

#[derive(Debug, Display, Error)]
pub enum ReadBankTableError {
    SeekCount(io::Error),
    ReadCount(io::Error),
    ParseCount(TakeError),
    ReadMetadata(io::Error),
    #[display("read bank metadata ${_0}: ${_1}")]
    ReadBankMetadata(u32, BankMetadataError),
    #[display("read bank ${_0}: ${_1}")]
    ReadBank(u32, ReadBankError),
}

impl<T: Read + Seek> BigReader<T> {
    fn read_bank_table(
        mut source: &mut T,
        header: &Header,
    ) -> Result<HashMap<String, Bank>, ReadBankTableError> {
        use ReadBankTableError as E;

        let mut count = [0u8; BankCount::BYTE_SIZE];

        let pos = SeekFrom::Start(header.banks_position as u64);

        source.seek(pos).map_err(E::SeekCount)?;

        source.read_exact(&mut count).map_err(E::ReadCount)?;

        let BankCount { count } = BankCount::parse(&mut &count[..]).map_err(E::ParseCount)?;

        let mut table_bytes = Vec::new();

        source
            .read_to_end(&mut table_bytes)
            .map_err(E::ReadMetadata)?;

        let mut table_bytes = &table_bytes[..];

        let mut table = HashMap::new();

        for i in 0..count {
            let metadata = BankMetadataRef::parse(&mut table_bytes)
                .map_err(|e| E::ReadBankMetadata(i, e))?
                .into_owned();

            let name = metadata.name.to_string();

            let bank = Bank::read(&mut source, metadata).map_err(|e| E::ReadBank(i, e))?;

            table.insert(name, bank);
        }

        Ok(table)
    }
}

pub struct Bank {
    metadata: BankMetadata,
    /// Type-id → type-name map parsed from the bank, retained for completeness though not yet used.
    #[allow(dead_code)]
    type_map: TypeMap,
    assets: HashMap<String, AssetMetadata>,
}

impl Bank {
    pub fn metadata(&self) -> &BankMetadata {
        &self.metadata
    }

    pub fn asset_iter(&self) -> impl Iterator<Item = &AssetMetadata> {
        self.assets.values()
    }

    pub fn asset(&self, symbol_name: &str) -> Option<&AssetMetadata> {
        self.assets.get(symbol_name)
    }

    /// Look up an asset by its numeric id. Def files reference graphics by this
    /// id (e.g. `EnvironmentThemeDef::sky_texture_0`); this resolves it back to
    /// the asset (and thus its `symbol_name`).
    pub fn asset_by_id(&self, id: u32) -> Option<&AssetMetadata> {
        self.assets.values().find(|a| a.id == id)
    }

    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }
}

#[derive(Debug, Display, Error)]
pub enum ReadBankError {
    SeekHeader(io::Error),
    ReadHeader(io::Error),
    ParseHeader(AssetTableHeaderError),
    #[display("asset metadata ${_0}: ${_1}")]
    ParseAssetMetadata(u32, AssetMetadataError),
}

impl Bank {
    fn read<T: Read + Seek>(mut file: T, metadata: BankMetadata) -> Result<Self, ReadBankError> {
        use ReadBankError as E;

        let mut table_bytes = vec![0u8; metadata.length as usize];

        let pos = SeekFrom::Start(metadata.position as u64);

        file.seek(pos).map_err(E::SeekHeader)?;

        file.read_exact(&mut table_bytes).map_err(E::ReadHeader)?;

        let mut table_bytes = &table_bytes[..];

        let header = TypeMap::parse(&mut table_bytes).map_err(E::ParseHeader)?;

        let mut assets = HashMap::with_capacity(metadata.asset_count as usize);

        for i in 0..metadata.asset_count {
            let asset_metadata = AssetMetadataRef::parse(&mut table_bytes)
                .map_err(|e| E::ParseAssetMetadata(i, e))?
                .into_owned();

            assets.insert(asset_metadata.symbol_name.to_string(), asset_metadata);
        }

        Ok(Self {
            metadata,
            type_map: header,
            assets,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub banks_position: u32,
    pub unknown_1: u32,
}

#[derive(Display, Copy, Clone, Debug, Error)]
pub enum HeaderError {
    Magic,
    Version,
    BanksPosition,
    Unknown1,
}

impl Header {
    pub const BYTE_SIZE: usize = 16;

    pub fn parse(i: &mut &[u8]) -> Result<Self, HeaderError> {
        use HeaderError as E;

        let magic = take::<[u8; 4]>(i).map_err(|_| E::Magic)?;
        let version = take::<u32>(i).map_err(|_| E::Version)?.to_le();
        let banks_position = take::<u32>(i).map_err(|_| E::BanksPosition)?.to_le();
        let unknown_1 = take::<u32>(i).map_err(|_| E::Unknown1)?.to_le();

        Ok(Header {
            magic,
            version,
            banks_position,
            unknown_1,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), HeaderError> {
        use HeaderError as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.to_le()).map_err(|_| E::Version)?;
        put(out, &self.banks_position.to_le()).map_err(|_| E::BanksPosition)?;
        put(out, &self.unknown_1.to_le()).map_err(|_| E::Unknown1)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BankCount {
    pub count: u32,
}

impl BankCount {
    pub const BYTE_SIZE: usize = 4;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, TakeError> {
        let bank_info_count = take::<u32>(inp)?.to_le();
        Ok(Self {
            count: bank_info_count,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.count.to_le())?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BankMetadataRef<'a> {
    pub name: Cow<'a, str>,
    pub id: u32,
    pub asset_count: u32,
    pub position: u32,
    pub length: u32,
    pub block_size: u32,
}

pub type BankMetadata = BankMetadataRef<'static>;

#[derive(Display, Clone, Debug, Error)]
pub enum BankMetadataError {
    NameBytes,
    NameUtf8String,
    Id,
    AssetCount,
    Position,
    Length,
    BlockSize,
}

impl<'a> BankMetadataRef<'a> {
    pub fn parse(input: &mut &'a [u8]) -> Result<Self, BankMetadataError> {
        use BankMetadataError as E;

        let name_bytes = take_null_terminated_bytes(input).map_err(|_| E::NameBytes)?;
        let name_string = str::from_utf8(name_bytes).map_err(|_| E::NameUtf8String)?;
        let name = Cow::from(name_string);
        let id = take::<u32>(input).map_err(|_| E::Id)?.to_le();
        let asset_count = take::<u32>(input).map_err(|_| E::AssetCount)?.to_le();
        let position = take::<u32>(input).map_err(|_| E::Position)?.to_le();
        let length = take::<u32>(input).map_err(|_| E::Length)?.to_le();
        let block_size = take::<u32>(input).map_err(|_| E::BlockSize)?.to_le();

        Ok(Self {
            name,
            id,
            asset_count,
            position,
            length,
            block_size,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BankMetadataError> {
        use BankMetadataError as E;

        put_bytes(out, self.name.as_bytes()).map_err(|_| E::NameBytes)?;
        put(out, b"\0").map_err(|_| E::NameBytes)?;
        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.asset_count.to_le()).map_err(|_| E::AssetCount)?;
        put(out, &self.position.to_le()).map_err(|_| E::Position)?;
        put(out, &self.length.to_le()).map_err(|_| E::Length)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;

        Ok(())
    }

    pub fn into_owned(self) -> BankMetadata {
        BankMetadata {
            name: Cow::Owned(self.name.into_owned()),
            id: self.id,
            asset_count: self.asset_count,
            position: self.position,
            length: self.length,
            block_size: self.block_size,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeMap {
    pub file_type: u32,
    pub type_map: BTreeMap<u32, u32>,
}

#[derive(Display, Clone, Debug, Error)]
pub enum AssetTableHeaderError {
    FileType,
    TypeMapCount,
    TypeMapCountInt,
    TypeMap,
    AssetCount,
    AssetCountInt,
}

impl TypeMap {
    pub fn parse(i: &mut &[u8]) -> Result<TypeMap, AssetTableHeaderError> {
        use AssetTableHeaderError as E;

        let types_count = take::<u32>(i).map_err(|_| E::TypeMapCount)?.to_le();
        let file_type = take::<u32>(i).map_err(|_| E::FileType)?.to_le();
        let _asset_count = take::<u32>(i).map_err(|_| E::AssetCount)?.to_le();
        let mut type_map = BTreeMap::new();
        let type_map_count = types_count.saturating_sub(1);

        for _ in 0..type_map_count {
            let v1 = take::<u32>(i).map_err(|_| E::TypeMap)?.to_le();
            let v2 = take::<u32>(i).map_err(|_| E::TypeMap)?.to_le();
            type_map.insert(v1, v2);
        }

        Ok(Self {
            file_type,
            type_map,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssetMetadataRef<'a> {
    pub magic: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: Cow<'a, str>,
    pub crc: u32,
    pub files: Vec<Cow<'a, str>>,
    pub extras: Option<ExtraMetadata>,
}

pub type AssetMetadata = AssetMetadataRef<'static>;

#[derive(Debug, Display, Clone)]
pub enum AssetMetadataError {
    Magic,
    Id,
    FileType,
    Size,
    Start,
    FileTypeDev,
    SymbolNameLen,
    SymbolNameLenInt,
    SymbolName,
    Crc,
    FileNamesCount,
    FileNamesCountInt,
    FileNameLen,
    FileNameLenInt,
    FileName,
    ExtrasLen,
    ExtrasLenInt,
    ExtrasBytes,
    DialogueExtra(DialogueExtraError),
    AnimationExtras(AnimationExtraError),
    TextureExtras(TextureExtraError),
    MeshExtras(MeshExtraError),
}

impl<'a> AssetMetadataRef<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<Self, AssetMetadataError> {
        use AssetMetadataError as E;

        let magic = take::<u32>(inp).map_err(|_| E::Magic)?.to_le();
        let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
        let file_type = take::<u32>(inp).map_err(|_| E::FileType)?.to_le();
        let size = take::<u32>(inp).map_err(|_| E::Size)?.to_le();
        let start = take::<u32>(inp).map_err(|_| E::Start)?.to_le();
        let file_type_dev = take::<u32>(inp).map_err(|_| E::FileTypeDev)?.to_le();
        let symbol_name_len = take::<u32>(inp).map_err(|_| E::SymbolNameLen)?.to_le();
        let symbol_name_len = usize::try_from(symbol_name_len).map_err(|_| E::SymbolNameLenInt)?;
        let symbol_name = take_bytes(inp, symbol_name_len).map_err(|_| E::SymbolName)?;
        let symbol_name = std::str::from_utf8(symbol_name).map_err(|_| E::SymbolName)?;
        let symbol_name = Cow::from(symbol_name);
        let crc = take::<u32>(inp).map_err(|_| E::Crc)?.to_le();
        let files_count = take::<u32>(inp).map_err(|_| E::FileNamesCount)?.to_le();
        let files_count = usize::try_from(files_count).map_err(|_| E::FileNamesCountInt)?;
        let mut files = Vec::with_capacity(files_count);

        for _ in 0..files_count {
            let name_len = take::<u32>(inp).map_err(|_| E::FileNameLen)?;
            let name_len = usize::try_from(name_len).map_err(|_| E::FileNameLenInt)?;
            let name = take_bytes(inp, name_len).map_err(|_| E::FileName)?;
            let name = std::str::from_utf8(name).map_err(|_| E::FileName)?;
            let name = Cow::from(name);

            files.push(name);
        }

        let extras_len = take::<u32>(inp).map_err(|_| E::ExtrasLen)?;
        let extras_len = usize::try_from(extras_len).map_err(|_| E::ExtrasLenInt)?;
        let mut extras_bytes = take_bytes(inp, extras_len).map_err(|_| E::ExtrasBytes)?;

        let extras = match extras_len {
            0 => None,
            4 => Some(ExtraMetadata::Dialogue(
                DialogueMetadata::parse(&mut extras_bytes).map_err(E::DialogueExtra)?,
            )),
            24 => Some(ExtraMetadata::Animation(
                AnimationMetadata::parse(&mut extras_bytes).map_err(E::AnimationExtras)?,
            )),
            34 => Some(ExtraMetadata::Texture(
                TextureMetadata::parse(&mut extras_bytes).map_err(E::TextureExtras)?,
            )),
            x if x > 45 => Some(ExtraMetadata::Mesh(
                MeshMetadata::parse(&mut extras_bytes).map_err(E::MeshExtras)?,
            )),
            _ => Some(ExtraMetadata::Unknown(extras_bytes.to_vec())),
        };

        Ok(Self {
            magic,
            id,
            file_type,
            size,
            start,
            file_type_dev,
            symbol_name,
            crc,
            files,
            extras,
        })
    }

    pub fn into_owned(self) -> AssetMetadata {
        AssetMetadata {
            magic: self.magic,
            id: self.id,
            file_type: self.file_type,
            size: self.size,
            start: self.start,
            file_type_dev: self.file_type_dev,
            symbol_name: Cow::Owned(self.symbol_name.into_owned()),
            crc: self.crc,
            files: self
                .files
                .into_iter()
                .map(|x| Cow::Owned(x.into_owned()))
                .collect(),
            extras: self.extras,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraMetadata {
    Texture(TextureMetadata),
    Mesh(MeshMetadata),
    Animation(AnimationMetadata),
    Dialogue(DialogueMetadata),
    Unknown(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureMetadata {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub frame_width: u16,
    pub frame_height: u16,
    pub frame_count: u16,
    pub dxt_compression: u16,
    pub unknown1: u16,
    pub transparency: u8,
    pub mip_maps: u8,
    pub unknown2: u16,
    pub top_mip_map_size: u32,
    pub top_mip_map_compressed_size: u32,
    pub unknown3: u16,
    pub unknown4: u32,
}

#[derive(Display, Debug, Clone)]
pub enum TextureExtraError {
    Width,
    Height,
    Depth,
    FrameWidth,
    FrameHeight,
    FrameCount,
    DxtCompression,
    Unknown1,
    Transparency,
    MipMaps,
    Unknown2,
    TopMipMapSize,
    TopMipMapCompressedSize,
    Unknown3,
    Unknown4,
}

impl TextureMetadata {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, TextureExtraError> {
        use TextureExtraError as E;

        let width = take::<u16>(inp).map_err(|_| E::Width)?.to_le();
        let height = take::<u16>(inp).map_err(|_| E::Height)?.to_le();
        let depth = take::<u16>(inp).map_err(|_| E::Depth)?.to_le();
        let frame_width = take::<u16>(inp).map_err(|_| E::FrameWidth)?.to_le();
        let frame_height = take::<u16>(inp).map_err(|_| E::FrameHeight)?.to_le();
        let frame_count = take::<u16>(inp).map_err(|_| E::FrameCount)?.to_le();
        let dxt_compression = take::<u16>(inp).map_err(|_| E::DxtCompression)?.to_le();
        let unknown1 = take::<u16>(inp).map_err(|_| E::Unknown1)?.to_le();
        let transparency = take::<u8>(inp).map_err(|_| E::Transparency)?.to_le();
        let mip_maps = take::<u8>(inp).map_err(|_| E::MipMaps)?.to_le();
        let unknown2 = take::<u16>(inp).map_err(|_| E::Unknown2)?.to_le();
        let top_mip_map_size = take::<u32>(inp).map_err(|_| E::TopMipMapSize)?.to_le();
        let top_mip_map_compressed_size = take::<u32>(inp)
            .map_err(|_| E::TopMipMapCompressedSize)?
            .to_le();
        let unknown3 = take::<u16>(inp).map_err(|_| E::Unknown3)?.to_le();
        let unknown4 = take::<u32>(inp).map_err(|_| E::Unknown4)?.to_le();

        Ok(Self {
            width,
            height,
            depth,
            frame_width,
            frame_height,
            frame_count,
            dxt_compression,
            unknown1,
            transparency,
            mip_maps,
            unknown2,
            top_mip_map_size,
            top_mip_map_compressed_size,
            unknown3,
            unknown4,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshMetadata {
    pub physics_mesh: u32,
    pub unknown1: [f32; 10],
    pub size_compressed_lod: Vec<u32>,
    pub padding: u32,
    pub unknown2: Vec<u32>,
    pub texture_ids: Vec<u32>,
}

#[derive(Debug, Display, Clone)]
pub enum MeshExtraError {
    PhysicsMesh,
    Unknown1,
    SizeCompressedLodCount,
    SizeCompressedLodCountInt,
    SizeCompressedLod,
    Padding,
    Unknown2,
    TextureIdsCount,
    TextureIdsCountInt,
    TextureIds,
}

impl MeshMetadata {
    pub fn parse(i: &mut &[u8]) -> Result<Self, MeshExtraError> {
        use MeshExtraError as E;

        let physics_mesh = take::<u32>(i).map_err(|_| E::PhysicsMesh)?.to_le();

        let unknown1 = take::<[f32; 10]>(i).map_err(|_| E::Unknown1)?;

        let size_compressed_lod_count = take::<u32>(i)
            .map_err(|_| E::SizeCompressedLodCount)?
            .to_le();
        let size_compressed_lod_count =
            usize::try_from(size_compressed_lod_count).map_err(|_| E::SizeCompressedLodCountInt)?;

        let mut size_compressed_lod = Vec::with_capacity(size_compressed_lod_count);

        for _ in 0..size_compressed_lod_count {
            let unknown = take::<u32>(i).map_err(|_| E::SizeCompressedLod)?.to_le();
            size_compressed_lod.push(unknown);
        }

        let padding = take::<u32>(i).map_err(|_| E::Padding)?.to_le();

        let unknown2_count = size_compressed_lod_count - 1;
        let mut unknown2 = Vec::with_capacity(unknown2_count);

        for _ in 0..unknown2_count {
            let unknown = take::<u32>(i).map_err(|_| E::Unknown2)?.to_le();
            unknown2.push(unknown);
        }

        let texture_ids_count = take::<u32>(i).map_err(|_| E::TextureIdsCount)?.to_le();
        let texture_ids_count =
            usize::try_from(texture_ids_count).map_err(|_| E::TextureIdsCountInt)?;
        let mut texture_ids = Vec::with_capacity(texture_ids_count);

        for _ in 0..texture_ids_count {
            let texture_id = take::<u32>(i).map_err(|_| E::TextureIds)?.to_le();
            texture_ids.push(texture_id);
        }

        Ok(Self {
            physics_mesh,
            unknown1,
            size_compressed_lod,
            padding,
            unknown2,
            texture_ids,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationMetadata {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}

#[derive(Debug, Display, Clone)]
pub enum AnimationExtraError {
    Unknown1,
    Unknown2,
    Unknown3,
}

impl AnimationMetadata {
    pub fn parse(i: &mut &[u8]) -> Result<Self, AnimationExtraError> {
        use AnimationExtraError as E;

        let unknown1 = take::<f32>(i).map_err(|_| E::Unknown1)?;
        let unknown2 = take::<f32>(i).map_err(|_| E::Unknown2)?;
        let unknown3 = i.to_vec();

        Ok(Self {
            unknown1,
            unknown2,
            unknown3,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogueMetadata {
    pub unknown1: u32,
}

#[derive(Debug, Display, Clone)]
pub enum DialogueExtraError {
    Unknown1,
}

impl DialogueMetadata {
    pub fn parse(i: &mut &[u8]) -> Result<Self, DialogueExtraError> {
        use DialogueExtraError as E;

        let unknown1 = take::<u32>(i).map_err(|_| E::Unknown1)?;

        Ok(Self { unknown1 })
    }
}
