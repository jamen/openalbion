use super::bytes::{put, put_bytes, take, take_bytes, take_bytes_nul_terminated};
use derive_more::derive::{Display, Error};
use fallible_iterator::FallibleIterator;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, Read, Seek, SeekFrom},
};

#[derive(Debug, PartialEq)]
pub struct BigHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub index_header_position: u32,
    pub unknown_1: u32,
}

#[derive(Error, Display, Copy, Clone, Debug)]
pub enum BigHeaderError {
    Magic,
    Version,
    IndexPosition,
    Unknown1,
}

impl BigHeader {
    pub const BYTE_SIZE: usize = 16;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigHeaderError> {
        use BigHeaderError as E;

        let magic = take::<[u8; 4]>(inp).map_err(|_| E::Magic)?;
        let version = take::<u32>(inp).map_err(|_| E::Version)?.to_le();
        let index_position = take::<u32>(inp).map_err(|_| E::IndexPosition)?.to_le();
        let unknown_1 = take::<u32>(inp).map_err(|_| E::Unknown1)?.to_le();

        Ok(BigHeader {
            magic,
            version,
            index_header_position: index_position,
            unknown_1,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigHeaderError> {
        use BigHeaderError as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.to_le()).map_err(|_| E::Version)?;
        put(out, &self.index_header_position.to_le()).map_err(|_| E::IndexPosition)?;
        put(out, &self.unknown_1.to_le()).map_err(|_| E::Unknown1)?;

        Ok(())
    }
}

pub struct BigBanks<'a> {
    count: u32,
    bytes: Cow<'a, [u8]>,
}

impl<'a> BigBanks<'a> {
    fn new(bytes: Cow<'a, [u8]>) -> Result<Self, BigBanksError> {
        use BigBanksError as E;

        let bytes_buf = &mut &bytes[..];

        let count = take::<u32>(bytes_buf).map_err(|_| E::EntryCount)?.to_le();

        Ok(Self { count, bytes })
    }

    fn into_owned(self) -> BigBanks<'static> {
        BigBanks {
            count: self.count,
            bytes: Cow::Owned(self.bytes.into_owned()),
        }
    }

    fn into_reader(self) -> BigBanksReader<'a> {
        BigBanksReader::new(self)
    }
}

#[derive(Error, Display, Clone, Debug)]
pub enum BigBanksError {
    EntryCount,
}

pub struct BigBanksReader<'a> {
    banks: BigBanks<'a>,
    position: usize,
}

impl<'a> BigBanksReader<'a> {
    pub fn new(banks: BigBanks<'a>) -> Self {
        Self {
            position: 4,
            banks,
        }
    }

    fn into_owned(self) -> BigBanksReader<'static> {
        BigBanksReader {
            position: self.position,
            banks: self.banks.into_owned(),
        }
    }
}

// #[derive(Debug, PartialEq)]
// pub struct BigIndex {
//     pub entries: Vec<BigBankEntry>,
// }

// #[derive(Error, Display, Clone, Debug)]
// pub enum BigIndexError {
//     EntryCount,
//     Entry(BigBankEntryError),
// }

// impl BigIndex {
//     pub fn parse(inp: &mut &[u8]) -> Result<Self, BigIndexError> {
//         use BigIndexError as E;

//         let entry_count = take::<u32>(inp).map_err(|_| E::EntryCount)?.to_le();

//         let mut entries = Vec::with_capacity(entry_count as usize);

//         for _ in 0..entry_count {
//             entries.push(BigBankEntry::parse(inp).map_err(E::Entry)?);
//         }

//         Ok(BigIndex { entries })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigIndexError> {
//         use BigIndexError as E;

//         let entry_count = u32::try_from(self.entries.len()).map_err(|_| E::EntryCount)?;

//         put(out, &entry_count.to_le()).map_err(|_| E::EntryCount)?;

//         for entry in &self.entries {
//             entry.serialize(out).map_err(E::Entry)?;
//         }

//         Ok(())
//     }
// }

#[derive(Debug, PartialEq)]
pub struct BigBankEntry {
    pub name: String,
    pub bank_id: u32,
    pub bank_entries_count: u32,
    pub bank_position: u32,
    pub bank_length: u32,
    pub block_size: u32,
}

#[derive(Error, Display, Clone, Debug)]
pub enum BigBankEntryError {
    NameBytes,
    Name,
    BankId,
    BankEntriesCount,
    BankPosition,
    BankLength,
    BlockSize,
}

impl BigBankEntry {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigBankEntryError> {
        use BigBankEntryError as E;

        let name_bytes = take_bytes_nul_terminated(inp).map_err(|_| E::NameBytes)?;
        let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| E::Name)?;

        let bank_id = take::<u32>(inp).map_err(|_| E::BankId)?.to_le();
        let bank_entries_count = take::<u32>(inp).map_err(|_| E::BankEntriesCount)?.to_le();
        let index_position = take::<u32>(inp).map_err(|_| E::BankPosition)?.to_le();
        let index_length = take::<u32>(inp).map_err(|_| E::BankLength)?.to_le();
        let block_size = take::<u32>(inp).map_err(|_| E::BlockSize)?.to_le();

        Ok(BigBankEntry {
            name,
            bank_id,
            bank_entries_count,
            bank_position: index_position,
            bank_length: index_length,
            block_size,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigBankEntryError> {
        use BigBankEntryError as E;

        put_bytes(out, &self.name.as_bytes()).map_err(|_| E::NameBytes)?;
        put(out, b"\0").map_err(|_| E::NameBytes)?;

        put(out, &self.bank_id.to_le()).map_err(|_| E::BankId)?;
        put(out, &self.bank_entries_count.to_le()).map_err(|_| E::BankEntriesCount)?;
        put(out, &self.bank_position.to_le()).map_err(|_| E::BankPosition)?;
        put(out, &self.bank_length.to_le()).map_err(|_| E::BankLength)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;

        Ok(())
    }

    pub const MIN_BYTE_SIZE: usize = 21;

    pub fn byte_size(&self) -> usize {
        Self::MIN_BYTE_SIZE + self.name.len()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigBankEntries<'a> {
    pub file_type: u32,
    pub types_map: BTreeMap<u32, u32>,
    pub entry_count: u32,
    pub entry_bytes: Cow<'a, [u8]>,
}

#[derive(Error, Display, Debug)]
pub enum BigBankError {
    TypesCount,
    TypesCountInt,
    FileType,
    EntryCount,
    EntriesCountInt,
    TypesMap,
    Entry(BigEntryError),
}

impl<'a> BigBankEntries<'a> {
    pub fn parse(i: &mut &'a [u8]) -> Result<Self, BigBankError> {
        use BigBankError as E;

        let types_count = take::<u32>(i).map_err(|_| E::TypesCount)?.to_le();
        let file_type = take::<u32>(i).map_err(|_| E::FileType)?.to_le();
        let entry_count = take::<u32>(i).map_err(|_| E::EntryCount)?.to_le();

        let mut types_map = BTreeMap::new();

        let types_map_count = types_count.saturating_sub(1);

        for _ in 0..types_map_count {
            let v1 = take::<u32>(i).map_err(|_| E::TypesMap)?.to_le();
            let v2 = take::<u32>(i).map_err(|_| E::TypesMap)?.to_le();
            types_map.insert(v1, v2);
        }

        let entry_bytes = Cow::from(*i);

        Ok(BigBankEntries {
            file_type,
            types_map,
            entry_count,
            entry_bytes,
        })
    }

    // pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigBankError> {
    //     use BigBankError as E;

    //     let types_count = u32::try_from(self.types_map.len()).map_err(|_| E::TypesCountInt)?;
    //     let types_count = types_count + 1;

    //     put(out, &types_count.to_le()).map_err(|_| E::TypesCount)?;
    //     put(out, &self.file_type.to_le()).map_err(|_| E::FileType)?;

    //     let entries_count = u32::try_from(self.entries.len()).map_err(|_| E::EntriesCountInt)?;

    //     put(out, &entries_count.to_le()).map_err(|_| E::EntryCount)?;

    //     for (&k, &v) in &self.types_map {
    //         put(out, &[k, v]).map_err(|_| E::TypesMap)?;
    //     }

    //     for entries in &self.entries {
    //         entries.serialize(out).map_err(E::Entry)?;
    //     }

    //     Ok(())
    // }

    // pub fn byte_size(&self) -> usize {
    //     todo!()
    // }

    pub fn into_owned(self) -> BigBankEntries<'static> {
        BigBankEntries {
            file_type: self.file_type,
            types_map: self.types_map,
            entry_count: self.entry_count,
            entry_bytes: Cow::Owned(self.entry_bytes.into_owned()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BigEntry {
    pub magic: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: String,
    pub crc: u32,
    pub files: Vec<String>,
    pub sub_header: BigSubheader,
}

#[derive(Debug, Display, Error)]
pub enum BigEntryError {
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
    SubHeaderLen,
    SubHeaderLenInt,
    SubHeaderBytes,
    SubHeader(BigSubheaderError),
}

impl BigEntry {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigEntryError> {
        use BigEntryError as E;

        let magic = take::<u32>(inp).map_err(|_| E::Magic)?.to_le();
        let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
        let file_type = take::<u32>(inp).map_err(|_| E::FileType)?.to_le();
        let size = take::<u32>(inp).map_err(|_| E::Size)?.to_le();
        let start = take::<u32>(inp).map_err(|_| E::Start)?.to_le();
        let file_type_dev = take::<u32>(inp).map_err(|_| E::FileTypeDev)?.to_le();

        let symbol_name_len = take::<u32>(inp).map_err(|_| E::SymbolNameLen)?.to_le();
        let symbol_name_len = usize::try_from(symbol_name_len).map_err(|_| E::SymbolNameLenInt)?;
        let symbol_name = take_bytes(inp, symbol_name_len).map_err(|_| E::SymbolName)?;
        let symbol_name = String::from_utf8(symbol_name.to_vec()).map_err(|_| E::SymbolName)?;

        let crc = take::<u32>(inp).map_err(|_| E::Crc)?.to_le();

        let files_count = take::<u32>(inp).map_err(|_| E::FileNamesCount)?.to_le();
        let files_count = usize::try_from(files_count).map_err(|_| E::FileNamesCountInt)?;

        let mut files = Vec::with_capacity(files_count);

        for _ in 0..files_count {
            let name_len = take::<u32>(inp).map_err(|_| E::FileNameLen)?;
            let name_len = usize::try_from(name_len).map_err(|_| E::FileNameLenInt)?;
            let name = take_bytes(inp, name_len).map_err(|_| E::FileName)?;
            let name = String::from_utf8(name.to_vec()).map_err(|_| E::FileName)?;

            files.push(name);
        }

        let sub_header_len = take::<u32>(inp).map_err(|_| E::SubHeaderLen)?;
        let sub_header_len = usize::try_from(sub_header_len).map_err(|_| E::SubHeaderLenInt)?;

        let sub_header = if sub_header_len == 0 {
            BigSubheader::None
        } else {
            let mut sub_header_bytes =
                take_bytes(inp, sub_header_len).map_err(|_| E::SubHeaderBytes)?;

            BigSubheader::parse(&mut sub_header_bytes, sub_header_len).map_err(E::SubHeader)?
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
            sub_header,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigEntryError> {
        use BigEntryError as E;

        put(out, &self.magic.to_le()).map_err(|_| E::Magic)?;
        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.file_type.to_le()).map_err(|_| E::FileType)?;
        put(out, &self.size.to_le()).map_err(|_| E::Size)?;
        put(out, &self.start.to_le()).map_err(|_| E::Start)?;
        put(out, &self.file_type_dev.to_le()).map_err(|_| E::FileTypeDev)?;

        let symbol_name_len =
            u32::try_from(self.symbol_name.len()).map_err(|_| E::SymbolNameLenInt)?;
        put(out, &symbol_name_len.to_le()).map_err(|_| E::SymbolNameLen)?;
        put_bytes(out, &self.symbol_name.as_bytes()).map_err(|_| E::SymbolName)?;

        put(out, &self.crc.to_le()).map_err(|_| E::Crc)?;

        let file_names_count = u32::try_from(self.files.len()).map_err(|_| E::FileNamesCountInt)?;
        put(out, &file_names_count.to_le()).map_err(|_| E::FileNamesCount)?;

        for name in &self.files {
            let name_size = u32::try_from(name.len()).map_err(|_| E::FileNameLenInt)?;
            put(out, &name_size.to_le()).map_err(|_| E::FileNameLen)?;
            put_bytes(out, name.as_bytes()).map_err(|_| E::FileName)?;
        }

        let sub_header_len =
            u32::try_from(self.sub_header.byte_size()).map_err(|_| E::SubHeaderLenInt)?;
        put(out, &sub_header_len.to_le()).map_err(|_| E::SubHeaderLen)?;
        self.sub_header.serialize(out).map_err(E::SubHeader)?;

        Ok(())
    }

    pub fn kind(&self) -> BigEntryKind {
        // TODO: Find a better way. Some entries lack a sub-header but we can still figure out what
        // kind of entry it is.
        match &self.sub_header {
            BigSubheader::None => BigEntryKind::Unknown,
            BigSubheader::Texture(_) => BigEntryKind::Texture,
            BigSubheader::Mesh(_) => BigEntryKind::Mesh,
            BigSubheader::Animation(_) => BigEntryKind::Animation,
            BigSubheader::Dialogue(_) => BigEntryKind::Dialogue,
            BigSubheader::Unknown(_) => BigEntryKind::Unknown,
        }
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BigEntryKind {
    Texture,
    Mesh,
    Animation,
    Dialogue,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum BigSubheader {
    None,
    Texture(BigSubheaderTexture),
    Mesh(BigSubheaderMesh),
    Animation(BigSubheaderAnimation),
    Dialogue(BigSubheaderDialogue),
    Unknown(Vec<u8>),
}

#[derive(Debug, Display, Error)]
pub enum BigSubheaderError {
    Texture(BigSubheaderTextureError),
    Mesh(BigSubheaderMeshError),
    Animation(BigSubheaderAnimationError),
    Dialogue(BigSubheaderDialogueError),
    Unknown,
}

impl BigSubheader {
    pub fn byte_size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Texture(x) => x.byte_size(),
            Self::Animation(x) => x.byte_size(),
            Self::Mesh(x) => x.byte_size(),
            Self::Dialogue(x) => x.byte_size(),
            Self::Unknown(x) => x.len(),
        }
    }

    pub fn parse(inp: &mut &[u8], sub_header_len: usize) -> Result<Self, BigSubheaderError> {
        use BigSubheaderError as E;
        Ok(match sub_header_len {
            0 => Self::None,
            4 => Self::Dialogue(BigSubheaderDialogue::parse(inp).map_err(E::Dialogue)?),
            24 => Self::Animation(BigSubheaderAnimation::parse(inp).map_err(E::Animation)?),
            34 => Self::Texture(BigSubheaderTexture::parse(inp).map_err(E::Texture)?),
            x if x > 45 => Self::Mesh(BigSubheaderMesh::parse(inp).map_err(E::Mesh)?),
            _ => Self::Unknown(inp.to_vec()),
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderError> {
        use BigSubheaderError as E;

        Ok(match self {
            Self::None => {}
            Self::Texture(subheader) => {
                subheader.serialize(out).map_err(E::Texture)?;
            }
            Self::Mesh(subheader) => {
                subheader.serialize(out).map_err(E::Mesh)?;
            }
            Self::Animation(subheader) => {
                subheader.serialize(out).map_err(E::Animation)?;
            }
            Self::Dialogue(subheader) => {
                subheader.serialize(out).map_err(E::Dialogue)?;
            }
            Self::Unknown(subheader) => {
                put_bytes(out, subheader).map_err(|_| E::Unknown)?;
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubheaderTexture {
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

#[derive(Debug, Display, Error)]
pub enum BigSubheaderTextureError {
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

impl BigSubheaderTexture {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigSubheaderTextureError> {
        use BigSubheaderTextureError as E;

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

    pub fn serialize(&self, _out: &mut &mut [u8]) -> Result<(), BigSubheaderTextureError> {
        todo!()
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubheaderMesh {
    pub physics_mesh: u32,
    pub unknown1: [f32; 10],
    pub size_compressed_lod: Vec<u32>,
    pub padding: u32,
    pub unknown2: Vec<u32>,
    pub texture_ids: Vec<u32>,
}

#[derive(Debug, Display, Error)]
pub enum BigSubheaderMeshError {
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

impl BigSubheaderMesh {
    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderMeshError> {
        use BigSubheaderMeshError as E;

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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderMeshError> {
        use BigSubheaderMeshError as E;

        put(out, &self.physics_mesh.to_le()).map_err(|_| E::PhysicsMesh)?;
        put(out, &self.unknown1).map_err(|_| E::Unknown1)?;

        let size_compressed_lod_count = u32::try_from(self.size_compressed_lod.len())
            .map_err(|_| E::SizeCompressedLodCountInt)?;

        put(out, &size_compressed_lod_count).map_err(|_| E::SizeCompressedLodCount)?;

        for size_compressed_lod in &self.size_compressed_lod {
            put(out, &size_compressed_lod.to_le()).map_err(|_| E::SizeCompressedLod)?;
        }

        put(out, &self.padding).map_err(|_| E::Padding)?;

        // let unknown2_count = self.size_compressed_lod.len() - 1;

        for v in &self.unknown2 {
            put(out, &v.to_le()).map_err(|_| E::Unknown2)?;
        }

        let texture_ids_count =
            u32::try_from(self.texture_ids.len()).map_err(|_| E::TextureIdsCountInt)?;

        put(out, &texture_ids_count).map_err(|_| E::TextureIdsCount)?;

        for texture_id in &self.texture_ids {
            put(out, &texture_id.to_le()).map_err(|_| E::TextureIds)?;
        }

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubheaderAnimation {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}

#[derive(Debug, Display, Error)]
pub enum BigSubheaderAnimationError {
    Unknown1,
    Unknown2,
    Unknown3,
}

impl BigSubheaderAnimation {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderAnimationError> {
        use BigSubheaderAnimationError as E;

        let unknown1 = take::<f32>(i).map_err(|_| E::Unknown1)?;
        let unknown2 = take::<f32>(i).map_err(|_| E::Unknown2)?;
        let unknown3 = i.to_vec();

        Ok(Self {
            unknown1,
            unknown2,
            unknown3,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderAnimationError> {
        use BigSubheaderAnimationError as E;

        put(out, &self.unknown1).map_err(|_| E::Unknown1)?;
        put(out, &self.unknown2).map_err(|_| E::Unknown2)?;
        put_bytes(out, &self.unknown3).map_err(|_| E::Unknown3)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubheaderDialogue {
    unknown1: u32,
}

#[derive(Debug, Display, Error)]
pub enum BigSubheaderDialogueError {
    Unknown1,
}

impl BigSubheaderDialogue {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderDialogueError> {
        use BigSubheaderDialogueError as E;

        let unknown1 = take::<u32>(i).map_err(|_| E::Unknown1)?;

        Ok(Self { unknown1 })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderDialogueError> {
        use BigSubheaderDialogueError as E;

        put(out, &self.unknown1.to_le()).map_err(|_| E::Unknown1)?;

        Ok(())
    }
}

pub struct BigReader<Source: Read + Seek> {
    source: Source,
}

impl<Source: Read + Seek> BigReader<Source> {
    pub fn new(source: Source) -> Self {
        Self { source }
    }

    pub fn read_header(&mut self) -> Result<BigHeader, BigReaderError> {
        use BigReaderError as E;

        let mut header_bytes = [0; BigHeader::BYTE_SIZE];

        self.source
            .seek(SeekFrom::Start(0))
            .map_err(E::SeekHeader)?;

        self.source
            .read_exact(&mut header_bytes)
            .map_err(E::ReadHeader)?;

        BigHeader::parse(&mut &header_bytes[..]).map_err(E::ParseHeader)
    }

    pub fn read_index(&mut self) -> Result<BigIndex, BigReaderError> {
        use BigReaderError as E;

        let header = self.read_header()?;

        let index_header_position = u64::try_from(header.index_header_position)
            .map_err(|_| E::ParseHeader(BigHeaderError::IndexPosition))?;

        let mut index_header_bytes = Vec::new();

        self.source
            .seek(SeekFrom::Start(index_header_position))
            .map_err(E::SeekIndex)?;

        self.source
            .read_to_end(&mut index_header_bytes)
            .map_err(E::ReadIndex)?;

        BigIndex::parse(&mut &index_header_bytes[..]).map_err(E::ParseIndex)
    }

    pub fn read_index_entry(
        &mut self,
        index_entry: &BigBankEntry,
    ) -> Result<BigBankEntries, BigReaderError> {
        use BigReaderError as E;

        let bank_position = u64::try_from(index_entry.bank_position)
            .map_err(|_| E::ParseIndex(BigIndexError::Entry(BigBankEntryError::BankPosition)))?;

        let bank_length = usize::try_from(index_entry.bank_length)
            .map_err(|_| E::ParseIndex(BigIndexError::Entry(BigBankEntryError::BankLength)))?;

        let mut bank_bytes = vec![0; bank_length];

        self.source
            .seek(SeekFrom::Start(bank_position))
            .map_err(E::SeekBank)?;

        self.source
            .read_exact(&mut bank_bytes)
            .map_err(E::ReadBank)?;

        BigBankEntries::parse(&mut &bank_bytes[..]).map_err(E::ParseBank)
    }

    fn read_entries()
}

#[derive(Error, Display, Debug)]
pub enum BigReaderError {
    SeekHeader(io::Error),
    ReadHeader(io::Error),
    ParseHeader(BigHeaderError),
    SeekIndex(io::Error),
    ReadIndex(io::Error),
    ParseIndex(BigIndexError),
    SeekBank(io::Error),
    ReadBank(io::Error),
    ParseBank(BigBankError),
}

pub struct BigEntryReader<'a> {
    bytes: Cow<'a, [u8]>,
    position: usize,
    entries_left: usize,
}

impl<'a> BigEntryReader<'a> {
    pub fn new(bytes: Cow<'a, [u8]>, entry_count: usize) -> Self {
        Self {
            bytes,
            position: 0,
            entries_left: entry_count,
        }
    }

    pub fn into_owned(self) -> BigEntryReader<'static> {
        BigEntryReader {
            bytes: Cow::Owned(self.bytes.into_owned()),
            position: self.position,
            entries_left: self.entries_left,
        }
    }

    pub fn into_iterator(self) -> fallible_iterator::Iterator<Self> {
        self.iterator()
    }
}

impl<'a> FallibleIterator for BigEntryReader<'a> {
    type Item = BigEntry;
    type Error = BigEntryError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.entries_left > 0 {
            let mut entry_bytes = &self.bytes[self.position..];
            let entry = BigEntry::parse(&mut entry_bytes)?;
            self.position += entry.byte_size();
            self.entries_left -= 1;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
}
