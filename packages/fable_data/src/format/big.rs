use crate::{
    TakeError, UnexpectedEnd,
    bytes::{put_bytes, take_bytes, take_bytes_nul_terminated},
    format::bytes::{put, take},
};
use derive_more::Display;
use std::{borrow::Cow, collections::BTreeMap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub banks_position: u32,
    pub unknown_1: u32,
}

#[derive(Display, Copy, Clone, Debug)]
pub enum BigHeaderSection {
    Magic,
    Version,
    BanksPosition,
    Unknown1,
}

impl BigHeader {
    pub const BYTE_SIZE: usize = 16;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigHeaderSection> {
        use BigHeaderSection as E;

        let magic = take::<[u8; 4]>(inp).map_err(|_| E::Magic)?;
        let version = take::<u32>(inp).map_err(|_| E::Version)?.to_le();
        let banks_position = take::<u32>(inp).map_err(|_| E::BanksPosition)?.to_le();
        let unknown_1 = take::<u32>(inp).map_err(|_| E::Unknown1)?.to_le();

        Ok(BigHeader {
            magic,
            version,
            banks_position,
            unknown_1,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigHeaderSection> {
        use BigHeaderSection as E;

        put(out, &self.magic).map_err(|_| E::Magic)?;
        put(out, &self.version.to_le()).map_err(|_| E::Version)?;
        put(out, &self.banks_position.to_le()).map_err(|_| E::BanksPosition)?;
        put(out, &self.unknown_1.to_le()).map_err(|_| E::Unknown1)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigBankInfoCount {
    pub bank_info_count: u32,
}

impl BigBankInfoCount {
    pub const BYTE_SIZE: usize = 4;

    pub fn parse(inp: &mut &[u8]) -> Result<Self, TakeError> {
        let bank_info_count = take::<u32>(inp)?.to_le();
        Ok(Self { bank_info_count })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &self.bank_info_count.to_le())?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigBankInfo<'a> {
    pub name: Cow<'a, str>,
    pub id: u32,
    pub asset_count: u32,
    pub position: u32,
    pub length: u32,
    pub block_size: u32,
}

pub type BigBankInfoOwned = BigBankInfo<'static>;

#[derive(Display, Clone, Debug)]
pub enum BigBankInfoSection {
    NameBytes,
    Name,
    Id,
    AssetCount,
    Position,
    Length,
    BlockSize,
}

impl<'a> BigBankInfo<'a> {
    pub fn parse(input: &mut &'a [u8]) -> Result<Self, BigBankInfoSection> {
        use BigBankInfoSection as E;

        let name_bytes = take_bytes_nul_terminated(input).map_err(|_| E::NameBytes)?;
        let name = str::from_utf8(name_bytes).map_err(|_| E::Name)?;
        let name = Cow::from(name);

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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigBankInfoSection> {
        use BigBankInfoSection as E;

        put_bytes(out, &self.name.as_bytes()).map_err(|_| E::NameBytes)?;
        put(out, b"\0").map_err(|_| E::NameBytes)?;

        put(out, &self.id.to_le()).map_err(|_| E::Id)?;
        put(out, &self.asset_count.to_le()).map_err(|_| E::AssetCount)?;
        put(out, &self.position.to_le()).map_err(|_| E::Position)?;
        put(out, &self.length.to_le()).map_err(|_| E::Length)?;
        put(out, &self.block_size.to_le()).map_err(|_| E::BlockSize)?;

        Ok(())
    }

    pub fn into_owned(self) -> BigBankInfoOwned {
        BigBankInfoOwned {
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
pub struct BigAssetInfoHeader {
    pub file_type: u32,
    pub type_map: BTreeMap<u32, u32>,
}

#[derive(Display, Clone, Debug)]
pub enum BigAssetInfoHeaderSection {
    FileType,
    TypeMapCount,
    TypeMapCountInt,
    TypeMap,
    AssetCount,
    AssetCountInt,
}

impl BigAssetInfoHeader {
    pub fn parse(i: &mut &[u8]) -> Result<BigAssetInfoHeader, BigAssetInfoHeaderSection> {
        use BigAssetInfoHeaderSection as E;

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
pub struct BigAssetInfo<'a> {
    pub magic: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: Cow<'a, str>,
    pub crc: u32,
    pub files: Vec<Cow<'a, str>>,
    pub extras: Option<BigAssetExtras>,
}

pub type BigAssetInfoOwned = BigAssetInfo<'static>;

#[derive(Debug, Display, Clone)]
pub enum BigAssetInfoSection {
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
    DialogueExtras(BigDialogueExtrasSection),
    AnimationExtras(BigAnimationExtrasSection),
    TextureExtras(BigTextureExtrasSection),
    MeshExtras(BigMeshExtrasSection),
}

impl<'a> BigAssetInfo<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<Self, BigAssetInfoSection> {
        use BigAssetInfoSection as E;

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
            4 => Some(BigAssetExtras::Dialogue(
                BigDialogueExtras::parse(&mut extras_bytes).map_err(E::DialogueExtras)?,
            )),
            24 => Some(BigAssetExtras::Animation(
                BigAnimationExtras::parse(&mut extras_bytes).map_err(E::AnimationExtras)?,
            )),
            34 => Some(BigAssetExtras::Texture(
                BigTextureExtras::parse(&mut extras_bytes).map_err(E::TextureExtras)?,
            )),
            x if x > 45 => Some(BigAssetExtras::Mesh(
                BigMeshExtras::parse(&mut extras_bytes).map_err(E::MeshExtras)?,
            )),
            _ => Some(BigAssetExtras::Unknown(extras_bytes.to_vec())),
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

    pub fn into_owned(self) -> BigAssetInfoOwned {
        BigAssetInfoOwned {
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
pub enum BigAssetExtras {
    Texture(BigTextureExtras),
    Mesh(BigMeshExtras),
    Animation(BigAnimationExtras),
    Dialogue(BigDialogueExtras),
    Unknown(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigTextureExtras {
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
pub enum BigTextureExtrasSection {
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

impl BigTextureExtras {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigTextureExtrasSection> {
        use BigTextureExtrasSection as E;

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
pub struct BigMeshExtras {
    pub physics_mesh: u32,
    pub unknown1: [f32; 10],
    pub size_compressed_lod: Vec<u32>,
    pub padding: u32,
    pub unknown2: Vec<u32>,
    pub texture_ids: Vec<u32>,
}

#[derive(Debug, Display, Clone)]
pub enum BigMeshExtrasSection {
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

impl BigMeshExtras {
    pub fn parse(i: &mut &[u8]) -> Result<Self, BigMeshExtrasSection> {
        use BigMeshExtrasSection as E;

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
pub struct BigAnimationExtras {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}

#[derive(Debug, Display, Clone)]
pub enum BigAnimationExtrasSection {
    Unknown1,
    Unknown2,
    Unknown3,
}

impl BigAnimationExtras {
    pub fn parse(i: &mut &[u8]) -> Result<Self, BigAnimationExtrasSection> {
        use BigAnimationExtrasSection as E;

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
pub struct BigDialogueExtras {
    pub unknown1: u32,
}

#[derive(Debug, Display, Clone)]
pub enum BigDialogueExtrasSection {
    Unknown1,
}

impl BigDialogueExtras {
    pub fn parse(i: &mut &[u8]) -> Result<Self, BigDialogueExtrasSection> {
        use BigDialogueExtrasSection as E;

        let unknown1 = take::<u32>(i).map_err(|_| E::Unknown1)?;

        Ok(Self { unknown1 })
    }
}

// #[derive(Debug, PartialEq)]
// pub struct BigBankEntries<'a> {
//     pub file_type: u32,
//     pub types_map: BTreeMap<u32, u32>,
//     pub entry_count: u32,
//     pub entry_bytes: Cow<'a, [u8]>,
// }

// #[derive(Error, Display, Debug)]
// pub enum BigBankError {
//     TypesCount,
//     TypesCountInt,
//     FileType,
//     EntryCount,
//     EntriesCountInt,
//     TypesMap,
//     Entry(BigEntryError),
// }

// impl<'a> BigBankEntries<'a> {
//     pub fn parse(i: &mut &'a [u8]) -> Result<Self, BigBankError> {
//         use BigBankError as E;

//         let types_count = take::<u32>(i).map_err(|_| E::TypesCount)?.to_le();
//         let file_type = take::<u32>(i).map_err(|_| E::FileType)?.to_le();
//         let entry_count = take::<u32>(i).map_err(|_| E::EntryCount)?.to_le();

//         let mut types_map = BTreeMap::new();

//         let types_map_count = types_count.saturating_sub(1);

//         for _ in 0..types_map_count {
//             let v1 = take::<u32>(i).map_err(|_| E::TypesMap)?.to_le();
//             let v2 = take::<u32>(i).map_err(|_| E::TypesMap)?.to_le();
//             types_map.insert(v1, v2);
//         }

//         let entry_bytes = Cow::from(*i);

//         Ok(BigBankEntries {
//             file_type,
//             types_map,
//             entry_count,
//             entry_bytes,
//         })
//     }

//     // pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigBankError> {
//     //     use BigBankError as E;

//     //     let types_count = u32::try_from(self.types_map.len()).map_err(|_| E::TypesCountInt)?;
//     //     let types_count = types_count + 1;

//     //     put(out, &types_count.to_le()).map_err(|_| E::TypesCount)?;
//     //     put(out, &self.file_type.to_le()).map_err(|_| E::FileType)?;

//     //     let entries_count = u32::try_from(self.entries.len()).map_err(|_| E::EntriesCountInt)?;

//     //     put(out, &entries_count.to_le()).map_err(|_| E::EntryCount)?;

//     //     for (&k, &v) in &self.types_map {
//     //         put(out, &[k, v]).map_err(|_| E::TypesMap)?;
//     //     }

//     //     for entries in &self.entries {
//     //         entries.serialize(out).map_err(E::Entry)?;
//     //     }

//     //     Ok(())
//     // }

//     // pub fn byte_size(&self) -> usize {
//     //     todo!()
//     // }

//     pub fn into_owned(self) -> BigBankEntries<'static> {
//         BigBankEntries {
//             file_type: self.file_type,
//             types_map: self.types_map,
//             entry_count: self.entry_count,
//             entry_bytes: Cow::Owned(self.entry_bytes.into_owned()),
//         }
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct BigEntry {
//     pub magic: u32,
//     pub id: u32,
//     pub file_type: u32,
//     pub size: u32,
//     pub start: u32,
//     pub file_type_dev: u32,
//     pub symbol_name: String,
//     pub crc: u32,
//     pub files: Vec<String>,
//     pub sub_header: BigSubheader,
// }

// #[derive(Debug, Display, Error)]
// pub enum BigEntryError {
//     Magic,
//     Id,
//     FileType,
//     Size,
//     Start,
//     FileTypeDev,
//     SymbolNameLen,
//     SymbolNameLenInt,
//     SymbolName,
//     Crc,
//     FileNamesCount,
//     FileNamesCountInt,
//     FileNameLen,
//     FileNameLenInt,
//     FileName,
//     SubHeaderLen,
//     SubHeaderLenInt,
//     SubHeaderBytes,
//     SubHeader(BigSubheaderError),
// }

// impl BigEntry {
//     pub fn parse(inp: &mut &[u8]) -> Result<Self, BigEntryError> {
//         use BigEntryError as E;

//         let magic = take::<u32>(inp).map_err(|_| E::Magic)?.to_le();
//         let id = take::<u32>(inp).map_err(|_| E::Id)?.to_le();
//         let file_type = take::<u32>(inp).map_err(|_| E::FileType)?.to_le();
//         let size = take::<u32>(inp).map_err(|_| E::Size)?.to_le();
//         let start = take::<u32>(inp).map_err(|_| E::Start)?.to_le();
//         let file_type_dev = take::<u32>(inp).map_err(|_| E::FileTypeDev)?.to_le();

//         let symbol_name_len = take::<u32>(inp).map_err(|_| E::SymbolNameLen)?.to_le();
//         let symbol_name_len = usize::try_from(symbol_name_len).map_err(|_| E::SymbolNameLenInt)?;
//         let symbol_name = take_bytes(inp, symbol_name_len).map_err(|_| E::SymbolName)?;
//         let symbol_name = String::from_utf8(symbol_name.to_vec()).map_err(|_| E::SymbolName)?;

//         let crc = take::<u32>(inp).map_err(|_| E::Crc)?.to_le();

//         let files_count = take::<u32>(inp).map_err(|_| E::FileNamesCount)?.to_le();
//         let files_count = usize::try_from(files_count).map_err(|_| E::FileNamesCountInt)?;

//         let mut files = Vec::with_capacity(files_count);

//         for _ in 0..files_count {
//             let name_len = take::<u32>(inp).map_err(|_| E::FileNameLen)?;
//             let name_len = usize::try_from(name_len).map_err(|_| E::FileNameLenInt)?;
//             let name = take_bytes(inp, name_len).map_err(|_| E::FileName)?;
//             let name = String::from_utf8(name.to_vec()).map_err(|_| E::FileName)?;

//             files.push(name);
//         }

//         let sub_header_len = take::<u32>(inp).map_err(|_| E::SubHeaderLen)?;
//         let sub_header_len = usize::try_from(sub_header_len).map_err(|_| E::SubHeaderLenInt)?;

//         let sub_header = if sub_header_len == 0 {
//             BigSubheader::None
//         } else {
//             let mut sub_header_bytes =
//                 take_bytes(inp, sub_header_len).map_err(|_| E::SubHeaderBytes)?;

//             BigSubheader::parse(&mut sub_header_bytes, sub_header_len).map_err(E::SubHeader)?
//         };

//         Ok(Self {
//             magic,
//             id,
//             file_type,
//             size,
//             start,
//             file_type_dev,
//             symbol_name,
//             crc,
//             files,
//             sub_header,
//         })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigEntryError> {
//         use BigEntryError as E;

//         put(out, &self.magic.to_le()).map_err(|_| E::Magic)?;
//         put(out, &self.id.to_le()).map_err(|_| E::Id)?;
//         put(out, &self.file_type.to_le()).map_err(|_| E::FileType)?;
//         put(out, &self.size.to_le()).map_err(|_| E::Size)?;
//         put(out, &self.start.to_le()).map_err(|_| E::Start)?;
//         put(out, &self.file_type_dev.to_le()).map_err(|_| E::FileTypeDev)?;

//         let symbol_name_len =
//             u32::try_from(self.symbol_name.len()).map_err(|_| E::SymbolNameLenInt)?;
//         put(out, &symbol_name_len.to_le()).map_err(|_| E::SymbolNameLen)?;
//         put_bytes(out, &self.symbol_name.as_bytes()).map_err(|_| E::SymbolName)?;

//         put(out, &self.crc.to_le()).map_err(|_| E::Crc)?;

//         let file_names_count = u32::try_from(self.files.len()).map_err(|_| E::FileNamesCountInt)?;
//         put(out, &file_names_count.to_le()).map_err(|_| E::FileNamesCount)?;

//         for name in &self.files {
//             let name_size = u32::try_from(name.len()).map_err(|_| E::FileNameLenInt)?;
//             put(out, &name_size.to_le()).map_err(|_| E::FileNameLen)?;
//             put_bytes(out, name.as_bytes()).map_err(|_| E::FileName)?;
//         }

//         let sub_header_len =
//             u32::try_from(self.sub_header.byte_size()).map_err(|_| E::SubHeaderLenInt)?;
//         put(out, &sub_header_len.to_le()).map_err(|_| E::SubHeaderLen)?;
//         self.sub_header.serialize(out).map_err(E::SubHeader)?;

//         Ok(())
//     }

//     pub fn kind(&self) -> BigEntryKind {
//         // TODO: Find a better way. Some entries lack a sub-header but we can still figure out what
//         // kind of entry it is.
//         match &self.sub_header {
//             BigSubheader::None => BigEntryKind::Unknown,
//             BigSubheader::Texture(_) => BigEntryKind::Texture,
//             BigSubheader::Mesh(_) => BigEntryKind::Mesh,
//             BigSubheader::Animation(_) => BigEntryKind::Animation,
//             BigSubheader::Dialogue(_) => BigEntryKind::Dialogue,
//             BigSubheader::Unknown(_) => BigEntryKind::Unknown,
//         }
//     }

//     pub fn byte_size(&self) -> usize {
//         todo!()
//     }
// }

// #[derive(Copy, Clone, PartialEq, Eq)]
// pub enum BigEntryKind {
//     Texture,
//     Mesh,
//     Animation,
//     Dialogue,
//     Unknown,
// }

// #[derive(Debug, PartialEq)]
// pub enum BigSubheader {
//     None,
//     Texture(BigSubheaderTexture),
//     Mesh(BigSubheaderMesh),
//     Animation(BigSubheaderAnimation),
//     Dialogue(BigSubheaderDialogue),
//     Unknown(Vec<u8>),
// }

// #[derive(Debug, Display, Error)]
// pub enum BigSubheaderError {
//     Texture(BigSubheaderTextureError),
//     Mesh(BigSubheaderMeshError),
//     Animation(BigSubheaderAnimationError),
//     Dialogue(BigSubheaderDialogueError),
//     Unknown,
// }

// impl BigSubheader {
//     pub fn byte_size(&self) -> usize {
//         match self {
//             Self::None => 0,
//             Self::Texture(x) => x.byte_size(),
//             Self::Animation(x) => x.byte_size(),
//             Self::Mesh(x) => x.byte_size(),
//             Self::Dialogue(x) => x.byte_size(),
//             Self::Unknown(x) => x.len(),
//         }
//     }

//     pub fn parse(inp: &mut &[u8], sub_header_len: usize) -> Result<Self, BigSubheaderError> {
//         use BigSubheaderError as E;
//         Ok(match sub_header_len {
//             0 => Self::None,
//             4 => Self::Dialogue(BigSubheaderDialogue::parse(inp).map_err(E::Dialogue)?),
//             24 => Self::Animation(BigSubheaderAnimation::parse(inp).map_err(E::Animation)?),
//             34 => Self::Texture(BigSubheaderTexture::parse(inp).map_err(E::Texture)?),
//             x if x > 45 => Self::Mesh(BigSubheaderMesh::parse(inp).map_err(E::Mesh)?),
//             _ => Self::Unknown(inp.to_vec()),
//         })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderError> {
//         use BigSubheaderError as E;

//         Ok(match self {
//             Self::None => {}
//             Self::Texture(subheader) => {
//                 subheader.serialize(out).map_err(E::Texture)?;
//             }
//             Self::Mesh(subheader) => {
//                 subheader.serialize(out).map_err(E::Mesh)?;
//             }
//             Self::Animation(subheader) => {
//                 subheader.serialize(out).map_err(E::Animation)?;
//             }
//             Self::Dialogue(subheader) => {
//                 subheader.serialize(out).map_err(E::Dialogue)?;
//             }
//             Self::Unknown(subheader) => {
//                 put_bytes(out, subheader).map_err(|_| E::Unknown)?;
//             }
//         })
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct BigSubheaderTexture {
//     pub width: u16,
//     pub height: u16,
//     pub depth: u16,
//     pub frame_width: u16,
//     pub frame_height: u16,
//     pub frame_count: u16,
//     pub dxt_compression: u16,
//     pub unknown1: u16,
//     pub transparency: u8,
//     pub mip_maps: u8,
//     pub unknown2: u16,
//     pub top_mip_map_size: u32,
//     pub top_mip_map_compressed_size: u32,
//     pub unknown3: u16,
//     pub unknown4: u32,
// }

// #[derive(Debug, Display, Error)]
// pub enum BigSubheaderTextureError {
//     Width,
//     Height,
//     Depth,
//     FrameWidth,
//     FrameHeight,
//     FrameCount,
//     DxtCompression,
//     Unknown1,
//     Transparency,
//     MipMaps,
//     Unknown2,
//     TopMipMapSize,
//     TopMipMapCompressedSize,
//     Unknown3,
//     Unknown4,
// }

// impl BigSubheaderTexture {
//     pub fn parse(inp: &mut &[u8]) -> Result<Self, BigSubheaderTextureError> {
//         use BigSubheaderTextureError as E;

//         let width = take::<u16>(inp).map_err(|_| E::Width)?.to_le();
//         let height = take::<u16>(inp).map_err(|_| E::Height)?.to_le();
//         let depth = take::<u16>(inp).map_err(|_| E::Depth)?.to_le();
//         let frame_width = take::<u16>(inp).map_err(|_| E::FrameWidth)?.to_le();
//         let frame_height = take::<u16>(inp).map_err(|_| E::FrameHeight)?.to_le();
//         let frame_count = take::<u16>(inp).map_err(|_| E::FrameCount)?.to_le();
//         let dxt_compression = take::<u16>(inp).map_err(|_| E::DxtCompression)?.to_le();
//         let unknown1 = take::<u16>(inp).map_err(|_| E::Unknown1)?.to_le();
//         let transparency = take::<u8>(inp).map_err(|_| E::Transparency)?.to_le();
//         let mip_maps = take::<u8>(inp).map_err(|_| E::MipMaps)?.to_le();
//         let unknown2 = take::<u16>(inp).map_err(|_| E::Unknown2)?.to_le();
//         let top_mip_map_size = take::<u32>(inp).map_err(|_| E::TopMipMapSize)?.to_le();
//         let top_mip_map_compressed_size = take::<u32>(inp)
//             .map_err(|_| E::TopMipMapCompressedSize)?
//             .to_le();
//         let unknown3 = take::<u16>(inp).map_err(|_| E::Unknown3)?.to_le();
//         let unknown4 = take::<u32>(inp).map_err(|_| E::Unknown4)?.to_le();

//         Ok(Self {
//             width,
//             height,
//             depth,
//             frame_width,
//             frame_height,
//             frame_count,
//             dxt_compression,
//             unknown1,
//             transparency,
//             mip_maps,
//             unknown2,
//             top_mip_map_size,
//             top_mip_map_compressed_size,
//             unknown3,
//             unknown4,
//         })
//     }

//     pub fn serialize(&self, _out: &mut &mut [u8]) -> Result<(), BigSubheaderTextureError> {
//         todo!()
//     }

//     pub fn byte_size(&self) -> usize {
//         todo!()
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct BigSubheaderMesh {
//     pub physics_mesh: u32,
//     pub unknown1: [f32; 10],
//     pub size_compressed_lod: Vec<u32>,
//     pub padding: u32,
//     pub unknown2: Vec<u32>,
//     pub texture_ids: Vec<u32>,
// }

// #[derive(Debug, Display, Error)]
// pub enum BigSubheaderMeshError {
//     PhysicsMesh,
//     Unknown1,
//     SizeCompressedLodCount,
//     SizeCompressedLodCountInt,
//     SizeCompressedLod,
//     Padding,
//     Unknown2,
//     TextureIdsCount,
//     TextureIdsCountInt,
//     TextureIds,
// }

// impl BigSubheaderMesh {
//     pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderMeshError> {
//         use BigSubheaderMeshError as E;

//         let physics_mesh = take::<u32>(i).map_err(|_| E::PhysicsMesh)?.to_le();

//         let unknown1 = take::<[f32; 10]>(i).map_err(|_| E::Unknown1)?;

//         let size_compressed_lod_count = take::<u32>(i)
//             .map_err(|_| E::SizeCompressedLodCount)?
//             .to_le();
//         let size_compressed_lod_count =
//             usize::try_from(size_compressed_lod_count).map_err(|_| E::SizeCompressedLodCountInt)?;

//         let mut size_compressed_lod = Vec::with_capacity(size_compressed_lod_count);

//         for _ in 0..size_compressed_lod_count {
//             let unknown = take::<u32>(i).map_err(|_| E::SizeCompressedLod)?.to_le();
//             size_compressed_lod.push(unknown);
//         }

//         let padding = take::<u32>(i).map_err(|_| E::Padding)?.to_le();

//         let unknown2_count = size_compressed_lod_count - 1;
//         let mut unknown2 = Vec::with_capacity(unknown2_count);

//         for _ in 0..unknown2_count {
//             let unknown = take::<u32>(i).map_err(|_| E::Unknown2)?.to_le();
//             unknown2.push(unknown);
//         }

//         let texture_ids_count = take::<u32>(i).map_err(|_| E::TextureIdsCount)?.to_le();
//         let texture_ids_count =
//             usize::try_from(texture_ids_count).map_err(|_| E::TextureIdsCountInt)?;
//         let mut texture_ids = Vec::with_capacity(texture_ids_count);

//         for _ in 0..texture_ids_count {
//             let texture_id = take::<u32>(i).map_err(|_| E::TextureIds)?.to_le();
//             texture_ids.push(texture_id);
//         }

//         Ok(Self {
//             physics_mesh,
//             unknown1,
//             size_compressed_lod,
//             padding,
//             unknown2,
//             texture_ids,
//         })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderMeshError> {
//         use BigSubheaderMeshError as E;

//         put(out, &self.physics_mesh.to_le()).map_err(|_| E::PhysicsMesh)?;
//         put(out, &self.unknown1).map_err(|_| E::Unknown1)?;

//         let size_compressed_lod_count = u32::try_from(self.size_compressed_lod.len())
//             .map_err(|_| E::SizeCompressedLodCountInt)?;

//         put(out, &size_compressed_lod_count).map_err(|_| E::SizeCompressedLodCount)?;

//         for size_compressed_lod in &self.size_compressed_lod {
//             put(out, &size_compressed_lod.to_le()).map_err(|_| E::SizeCompressedLod)?;
//         }

//         put(out, &self.padding).map_err(|_| E::Padding)?;

//         // let unknown2_count = self.size_compressed_lod.len() - 1;

//         for v in &self.unknown2 {
//             put(out, &v.to_le()).map_err(|_| E::Unknown2)?;
//         }

//         let texture_ids_count =
//             u32::try_from(self.texture_ids.len()).map_err(|_| E::TextureIdsCountInt)?;

//         put(out, &texture_ids_count).map_err(|_| E::TextureIdsCount)?;

//         for texture_id in &self.texture_ids {
//             put(out, &texture_id.to_le()).map_err(|_| E::TextureIds)?;
//         }

//         Ok(())
//     }

//     pub fn byte_size(&self) -> usize {
//         todo!()
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct BigSubheaderAnimation {
//     pub unknown1: f32,
//     pub unknown2: f32,
//     pub unknown3: Vec<u8>,
// }

// #[derive(Debug, Display, Error)]
// pub enum BigSubheaderAnimationError {
//     Unknown1,
//     Unknown2,
//     Unknown3,
// }

// impl BigSubheaderAnimation {
//     pub fn byte_size(&self) -> usize {
//         todo!()
//     }

//     pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderAnimationError> {
//         use BigSubheaderAnimationError as E;

//         let unknown1 = take::<f32>(i).map_err(|_| E::Unknown1)?;
//         let unknown2 = take::<f32>(i).map_err(|_| E::Unknown2)?;
//         let unknown3 = i.to_vec();

//         Ok(Self {
//             unknown1,
//             unknown2,
//             unknown3,
//         })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderAnimationError> {
//         use BigSubheaderAnimationError as E;

//         put(out, &self.unknown1).map_err(|_| E::Unknown1)?;
//         put(out, &self.unknown2).map_err(|_| E::Unknown2)?;
//         put_bytes(out, &self.unknown3).map_err(|_| E::Unknown3)?;

//         Ok(())
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct BigSubheaderDialogue {
//     unknown1: u32,
// }

// #[derive(Debug, Display, Error)]
// pub enum BigSubheaderDialogueError {
//     Unknown1,
// }

// impl BigSubheaderDialogue {
//     pub fn byte_size(&self) -> usize {
//         todo!()
//     }

//     pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubheaderDialogueError> {
//         use BigSubheaderDialogueError as E;

//         let unknown1 = take::<u32>(i).map_err(|_| E::Unknown1)?;

//         Ok(Self { unknown1 })
//     }

//     pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubheaderDialogueError> {
//         use BigSubheaderDialogueError as E;

//         put(out, &self.unknown1.to_le()).map_err(|_| E::Unknown1)?;

//         Ok(())
//     }
// }

// pub struct BigEntryReader<'a> {
//     bytes: Cow<'a, [u8]>,
//     position: usize,
//     entries_left: usize,
// }

// impl<'a> BigEntryReader<'a> {
//     pub fn new(bytes: Cow<'a, [u8]>, entry_count: usize) -> Self {
//         Self {
//             bytes,
//             position: 0,
//             entries_left: entry_count,
//         }
//     }

//     pub fn into_owned(self) -> BigEntryReader<'static> {
//         BigEntryReader {
//             bytes: Cow::Owned(self.bytes.into_owned()),
//             position: self.position,
//             entries_left: self.entries_left,
//         }
//     }

//     pub fn into_iterator(self) -> fallible_iterator::Iterator<Self> {
//         self.iterator()
//     }
// }

// impl<'a> FallibleIterator for BigEntryReader<'a> {
//     type Item = BigEntry;
//     type Error = BigEntryError;

//     fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
//         if self.entries_left > 0 {
//             let mut entry_bytes = &self.bytes[self.position..];
//             let entry = BigEntry::parse(&mut entry_bytes)?;
//             self.position += entry.byte_size();
//             self.entries_left -= 1;
//             Ok(Some(entry))
//         } else {
//             Ok(None)
//         }
//     }
// }
