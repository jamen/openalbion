use std::num::TryFromIntError;

use crate::common::bytes::{
    put, put_bytes, take, take_bytes, take_bytes_nul_terminated, TakeError, UnexpectedEnd,
};

#[derive(Debug, PartialEq)]
pub struct BigHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub bank_address: u32,
    pub unknown_1: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum BigHeaderError<E> {
    Magic(E),
    Version(E),
    BankAddress(E),
    Unknown1(E),
}

impl BigHeader {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigHeaderError<TakeError>> {
        use BigHeaderError::*;

        let magic = take::<[u8; 4]>(inp).map_err(Magic)?;
        let version = take::<u32>(inp).map_err(Version)?.to_le();
        let bank_address = take::<u32>(inp).map_err(BankAddress)?.to_le();
        let unknown_1 = take::<u32>(inp).map_err(Unknown1)?.to_le();

        Ok(BigHeader {
            magic,
            version,
            bank_address,
            unknown_1,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigHeaderError<UnexpectedEnd>> {
        use BigHeaderError::*;

        put(out, &self.magic).map_err(Magic)?;
        put(out, &self.version.to_le()).map_err(Version)?;
        put(out, &self.bank_address.to_le()).map_err(BankAddress)?;
        put(out, &self.unknown_1.to_le()).map_err(Unknown1)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        16
    }
}

#[derive(Debug, PartialEq)]
pub struct BigBankIndex<'a> {
    pub banks_count: u32,
    // Null-terminated string
    pub name: &'a [u8],
    pub bank_id: u32,
    pub bank_entries_count: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum BigBankIndexError<E> {
    BanksCount(E),
    Name(UnexpectedEnd),
    BankId(E),
    BankEntriesCount(E),
    IndexStart(E),
    IndexSize(E),
    BlockSize(E),
}

impl<'a> BigBankIndex<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<Self, BigBankIndexError<TakeError>> {
        use BigBankIndexError::*;

        let banks_count = take::<u32>(inp).map_err(BanksCount)?.to_le();
        let name = take_bytes_nul_terminated(inp).map_err(Name)?;
        let bank_id = take::<u32>(inp).map_err(BankId)?.to_le();
        let bank_entries_count = take::<u32>(inp).map_err(BankEntriesCount)?.to_le();
        let index_start = take::<u32>(inp).map_err(IndexStart)?.to_le();
        let index_size = take::<u32>(inp).map_err(IndexSize)?.to_le();
        let block_size = take::<u32>(inp).map_err(BlockSize)?.to_le();

        Ok(BigBankIndex {
            banks_count,
            name,
            bank_id,
            bank_entries_count,
            index_start,
            index_size,
            block_size,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigBankIndexError<UnexpectedEnd>> {
        use BigBankIndexError::*;

        put(out, &self.banks_count.to_le()).map_err(BanksCount)?;

        put_bytes(out, &self.name).map_err(Name)?;
        put(out, b"\0").map_err(Name)?;

        put(out, &self.bank_id.to_le()).map_err(BankId)?;
        put(out, &self.bank_entries_count.to_le()).map_err(BankEntriesCount)?;
        put(out, &self.index_start.to_le()).map_err(IndexStart)?;
        put(out, &self.index_size.to_le()).map_err(IndexSize)?;
        put(out, &self.block_size.to_le()).map_err(BlockSize)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        25 + self.name.len()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigFileIndex<'a> {
    pub file_type: u32,
    pub types_map: Vec<[u32; 2]>,
    pub entries: Vec<BigFileEntry<'a>>,
}

#[derive(Debug)]
pub enum BigFileIndexError<E> {
    TypesCount(E),
    TypesCountInt(TryFromIntError),
    FileType(E),
    EntriesCount(E),
    EntriesCountInt(TryFromIntError),
    TypesMap(E),
    Entry(BigFileEntryError<E>),
}

impl<'a> BigFileIndex<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<Self, BigFileIndexError<TakeError>> {
        use BigFileIndexError::*;

        let types_count = take::<u32>(inp).map_err(TypesCount)?.to_le();
        let file_type = take::<u32>(inp).map_err(FileType)?.to_le();
        let entries_count = take::<u32>(inp).map_err(EntriesCount)?.to_le();

        let types_map_count = usize::try_from(types_count - 1).map_err(TypesCountInt)?;

        let mut types_map = Vec::with_capacity(types_map_count);

        for _ in 0..types_map_count {
            let v1 = take::<u32>(inp).map_err(TypesMap)?.to_le();
            let v2 = take::<u32>(inp).map_err(TypesMap)?.to_le();
            types_map.push([v1, v2]);
        }

        let mut entries = Vec::with_capacity(entries_count.try_into().unwrap());

        for _ in 0..entries_count {
            let entry = BigFileEntry::parse(inp).map_err(BigFileIndexError::Entry)?;
            entries.push(entry);
        }

        Ok(BigFileIndex {
            file_type,
            types_map,
            entries,
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigFileIndexError<UnexpectedEnd>> {
        use BigFileIndexError::*;

        let types_count = u32::try_from(self.types_map.len()).map_err(TypesCountInt)?;
        let types_count = types_count + 1;

        put(out, &types_count.to_le()).map_err(TypesCount)?;
        put(out, &self.file_type.to_le()).map_err(FileType)?;

        let entries_count = u32::try_from(self.entries.len()).map_err(EntriesCountInt)?;

        put(out, &entries_count.to_le()).map_err(EntriesCount)?;

        for entry in &self.types_map {
            put(out, entry).map_err(TypesMap)?;
        }

        for entries in &self.entries {
            entries.serialize(out).map_err(Entry)?;
        }

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigFileEntry<'a> {
    pub magic: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: &'a [u8],
    pub crc: u32,
    pub files: Vec<&'a [u8]>,
    pub sub_header: BigSubHeader,
    // pub sub_header: Vec<u8>,
}

#[derive(Debug)]
pub enum BigFileEntryError<E> {
    Magic(E),
    Id(E),
    FileType(E),
    Size(E),
    Start(E),
    FileTypeDev(E),
    SymbolNameLen(E),
    SymbolNameLenInt(TryFromIntError),
    SymbolName(UnexpectedEnd),
    Crc(E),
    FileNamesCount(E),
    FileNamesCountInt(TryFromIntError),
    FileNameLen(E),
    FileNameLenInt(TryFromIntError),
    FileName(UnexpectedEnd),
    SubHeaderLen(E),
    SubHeaderLenInt(TryFromIntError),
    SubHeaderBytes(UnexpectedEnd),
    SubHeader(BigSubHeaderError<E>),
}

impl<'a> BigFileEntry<'a> {
    pub fn parse(inp: &mut &'a [u8]) -> Result<Self, BigFileEntryError<TakeError>> {
        use BigFileEntryError::*;

        let magic = take::<u32>(inp).map_err(Magic)?.to_le();
        let id = take::<u32>(inp).map_err(Id)?.to_le();
        let file_type = take::<u32>(inp).map_err(FileType)?.to_le();
        let size = take::<u32>(inp).map_err(Size)?.to_le();
        let start = take::<u32>(inp).map_err(Start)?.to_le();
        let file_type_dev = take::<u32>(inp).map_err(FileTypeDev)?.to_le();

        let symbol_name_len = take::<u32>(inp).map_err(SymbolNameLen)?.to_le();
        let symbol_name_len = usize::try_from(symbol_name_len).map_err(SymbolNameLenInt)?;
        let symbol_name = take_bytes(inp, symbol_name_len).map_err(SymbolName)?;

        let crc = take::<u32>(inp).map_err(Crc)?.to_le();

        let files_count = take::<u32>(inp).map_err(FileNamesCount)?.to_le();
        let files_count = usize::try_from(files_count).map_err(FileNamesCountInt)?;

        let mut files = Vec::with_capacity(files_count);

        for _ in 0..files_count {
            let name_len = take::<u32>(inp).map_err(FileNameLen)?;
            let name_len = usize::try_from(name_len).map_err(FileNameLenInt)?;
            let name = take_bytes(inp, name_len).map_err(FileName)?;

            files.push(name);
        }

        let sub_header_len = take::<u32>(inp).map_err(SubHeaderLen)?;
        let sub_header_len = usize::try_from(sub_header_len).map_err(SubHeaderLenInt)?;
        let mut sub_header_bytes = take_bytes(inp, sub_header_len).map_err(SubHeaderBytes)?;
        let sub_header =
            BigSubHeader::parse(&mut sub_header_bytes, file_type).map_err(SubHeader)?;

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

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigFileEntryError<UnexpectedEnd>> {
        use BigFileEntryError::*;

        put(out, &self.magic.to_le()).map_err(Magic)?;
        put(out, &self.id.to_le()).map_err(Id)?;
        put(out, &self.file_type.to_le()).map_err(FileType)?;
        put(out, &self.size.to_le()).map_err(Size)?;
        put(out, &self.start.to_le()).map_err(Start)?;
        put(out, &self.file_type_dev.to_le()).map_err(FileTypeDev)?;

        let symbol_name_len = u32::try_from(self.symbol_name.len()).map_err(SymbolNameLenInt)?;
        put(out, &symbol_name_len.to_le()).map_err(SymbolNameLen)?;
        put_bytes(out, &self.symbol_name).map_err(SymbolName)?;

        put(out, &self.crc.to_le()).map_err(Crc)?;

        let file_names_count = u32::try_from(self.files.len()).map_err(FileNamesCountInt)?;
        put(out, &file_names_count.to_le()).map_err(FileNamesCount)?;

        for name in &self.files {
            let name_size = u32::try_from(name.len()).map_err(FileNameLenInt)?;
            put(out, &name_size.to_le()).map_err(FileNameLen)?;
            put_bytes(out, name).map_err(FileName)?;
        }

        let sub_header_len = u32::try_from(self.sub_header.byte_size()).map_err(SubHeaderLenInt)?;
        put(out, &sub_header_len.to_le()).map_err(SubHeaderLen)?;
        self.sub_header.serialize(out).map_err(SubHeader)?;

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum BigSubHeader {
    None,
    Texture(BigSubHeaderTexture),
    Mesh(BigSubHeaderMesh),
    Animation(BigSubHeaderAnimation),
    Unknown(Vec<u8>),
}

#[derive(Debug)]
pub enum BigSubHeaderError<E> {
    Texture(BigSubHeaderTextureError<E>),
    Mesh(BigSubHeaderMeshError<E>),
    Animation(BigSubHeaderAnimationError<E>),
    Unknown(E),
}

impl BigSubHeader {
    pub fn byte_size(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Texture(x) => x.byte_size(),
            Self::Animation(x) => x.byte_size(),
            Self::Mesh(x) => x.byte_size(),
            Self::Unknown(x) => x.len(),
        }
    }

    pub fn parse(inp: &mut &[u8], file_type: u32) -> Result<Self, BigSubHeaderError<TakeError>> {
        use BigSubHeaderError::*;

        Ok(match file_type {
            0 => Self::Texture(BigSubHeaderTexture::parse(inp).map_err(Texture)?),
            1 | 2 | 4 | 5 => Self::Mesh(BigSubHeaderMesh::parse(inp).map_err(Mesh)?),
            // ? =>
            //     decode_big_sub_header_anim(input),
            // ? =>
            //     Ok((b"", BigSubHeader::None)),
            _ => Self::Unknown(inp.to_vec()),
        })
    }

    pub fn serialize(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderError<UnexpectedEnd>> {
        use BigSubHeaderError::*;

        Ok(match self {
            Self::None => {}
            Self::Texture(subheader) => {
                subheader.serialize(out).map_err(Texture)?;
            }
            Self::Mesh(subheader) => {
                subheader.serialize(out).map_err(Mesh)?;
            }
            Self::Animation(subheader) => {
                subheader.serialize(out).map_err(Animation)?;
            }
            Self::Unknown(subheader) => {
                put_bytes(out, subheader).map_err(Unknown)?;
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderTexture {
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

#[derive(Debug)]
pub enum BigSubHeaderTextureError<E> {
    Width(E),
    Height(E),
    Depth(E),
    FrameWidth(E),
    FrameHeight(E),
    FrameCount(E),
    DxtCompression(E),
    Unknown1(E),
    Transparency(E),
    MipMaps(E),
    Unknown2(E),
    TopMipMapSize(E),
    TopMipMapCompressedSize(E),
    Unknown3(E),
    Unknown4(E),
}

impl BigSubHeaderTexture {
    pub fn parse(inp: &mut &[u8]) -> Result<Self, BigSubHeaderTextureError<TakeError>> {
        use BigSubHeaderTextureError::*;

        let width = take::<u16>(inp).map_err(Width)?.to_le();
        let height = take::<u16>(inp).map_err(Height)?.to_le();
        let depth = take::<u16>(inp).map_err(Depth)?.to_le();
        let frame_width = take::<u16>(inp).map_err(FrameWidth)?.to_le();
        let frame_height = take::<u16>(inp).map_err(FrameHeight)?.to_le();
        let frame_count = take::<u16>(inp).map_err(FrameCount)?.to_le();
        let dxt_compression = take::<u16>(inp).map_err(DxtCompression)?.to_le();
        let unknown1 = take::<u16>(inp).map_err(Unknown1)?.to_le();
        let transparency = take::<u8>(inp).map_err(Transparency)?.to_le();
        let mip_maps = take::<u8>(inp).map_err(MipMaps)?.to_le();
        let unknown2 = take::<u16>(inp).map_err(Unknown2)?.to_le();
        let top_mip_map_size = take::<u32>(inp).map_err(TopMipMapSize)?.to_le();
        let top_mip_map_compressed_size =
            take::<u32>(inp).map_err(TopMipMapCompressedSize)?.to_le();
        let unknown3 = take::<u16>(inp).map_err(Unknown3)?.to_le();
        let unknown4 = take::<u32>(inp).map_err(Unknown4)?.to_le();

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

    pub fn serialize(
        &self,
        _out: &mut &mut [u8],
    ) -> Result<(), BigSubHeaderTextureError<UnexpectedEnd>> {
        todo!()
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderMesh {
    pub physics_mesh: u32,
    pub unknown1: [f32; 10],
    pub size_compressed_lod: Vec<u32>,
    pub padding: u32,
    pub unknown2: Vec<u32>,
    pub texture_ids: Vec<u32>,
}

#[derive(Debug)]
pub enum BigSubHeaderMeshError<E> {
    PhysicsMesh(E),
    Unknown1(E),
    SizeCompressedLodCount(E),
    SizeCompressedLodCountInt(TryFromIntError),
    SizeCompressedLod(E),
    Padding(E),
    Unknown2(E),
    TextureIdsCount(E),
    TextureIdsCountInt(TryFromIntError),
    TextureIds(E),
}

impl BigSubHeaderMesh {
    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubHeaderMeshError<TakeError>> {
        use BigSubHeaderMeshError::*;

        let physics_mesh = take::<u32>(i).map_err(PhysicsMesh)?.to_le();

        let unknown1 = take::<[f32; 10]>(i).map_err(Unknown1)?;

        let size_compressed_lod_count = take::<u32>(i).map_err(SizeCompressedLodCount)?.to_le();
        let size_compressed_lod_count =
            usize::try_from(size_compressed_lod_count).map_err(SizeCompressedLodCountInt)?;

        let mut size_compressed_lod = Vec::with_capacity(size_compressed_lod_count);

        for _ in 0..size_compressed_lod_count {
            let unknown = take::<u32>(i).map_err(SizeCompressedLod)?.to_le();
            size_compressed_lod.push(unknown);
        }

        let padding = take::<u32>(i).map_err(Padding)?.to_le();

        let unknown2_count = size_compressed_lod_count - 1;
        let mut unknown2 = Vec::with_capacity(unknown2_count);

        for _ in 0..unknown2_count {
            let unknown = take::<u32>(i).map_err(Unknown2)?.to_le();
            unknown2.push(unknown);
        }

        let texture_ids_count = take::<u32>(i).map_err(TextureIdsCount)?.to_le();
        let texture_ids_count = usize::try_from(texture_ids_count).map_err(TextureIdsCountInt)?;
        let mut texture_ids = Vec::with_capacity(texture_ids_count);

        for _ in 0..texture_ids_count {
            let texture_id = take::<u32>(i).map_err(TextureIds)?.to_le();
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

    pub fn serialize(
        &self,
        out: &mut &mut [u8],
    ) -> Result<(), BigSubHeaderMeshError<UnexpectedEnd>> {
        use BigSubHeaderMeshError::*;

        put(out, &self.physics_mesh.to_le()).map_err(PhysicsMesh)?;
        put(out, &self.unknown1).map_err(Unknown1)?;

        let size_compressed_lod_count =
            u32::try_from(self.size_compressed_lod.len()).map_err(SizeCompressedLodCountInt)?;

        put(out, &size_compressed_lod_count).map_err(SizeCompressedLodCount)?;

        for size_compressed_lod in &self.size_compressed_lod {
            put(out, &size_compressed_lod.to_le()).map_err(SizeCompressedLod)?;
        }

        put(out, &self.padding).map_err(Padding)?;

        // let unknown2_count = self.size_compressed_lod.len() - 1;

        for v in &self.unknown2 {
            put(out, &v.to_le()).map_err(Unknown2)?;
        }

        let texture_ids_count =
            u32::try_from(self.texture_ids.len()).map_err(TextureIdsCountInt)?;

        put(out, &texture_ids_count).map_err(TextureIdsCount)?;

        for texture_id in &self.texture_ids {
            put(out, &texture_id.to_le()).map_err(TextureIds)?;
        }

        Ok(())
    }

    pub fn byte_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderAnimation {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}

#[derive(Debug)]
pub enum BigSubHeaderAnimationError<E> {
    Unknown1(E),
    Unknown2(E),
    Unknown3(E),
}

impl BigSubHeaderAnimation {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubHeaderAnimationError<TakeError>> {
        use BigSubHeaderAnimationError::*;

        let unknown1 = take::<f32>(i).map_err(Unknown1)?;
        let unknown2 = take::<f32>(i).map_err(Unknown2)?;
        let unknown3 = i.to_vec();

        Ok(Self {
            unknown1,
            unknown2,
            unknown3,
        })
    }

    pub fn serialize(
        &self,
        out: &mut &mut [u8],
    ) -> Result<(), BigSubHeaderAnimationError<UnexpectedEnd>> {
        use BigSubHeaderAnimationError::*;

        put(out, &self.unknown1).map_err(Unknown1)?;
        put(out, &self.unknown2).map_err(Unknown2)?;
        put_bytes(out, &self.unknown3).map_err(Unknown3)?;

        Ok(())
    }
}
