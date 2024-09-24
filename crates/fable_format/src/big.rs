use crate::util::binary::{
    BinaryParser, BinaryParserError, BinarySerializer, BinarySerializerError,
};

#[derive(Debug, PartialEq)]
pub struct BigHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub bank_address: u32,
    pub unknown_1: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum BigHeaderPart {
    Magic,
    Version,
    BankAddress,
    Unknown1,
}

impl BigHeader {
    pub fn byte_size(&self) -> usize {
        16
    }

    pub fn parse(p: &mut BinaryParser) -> Result<Self, BinaryParserError<BigHeaderPart>> {
        use BigHeaderPart::*;

        let magic = p.take::<[u8; 4], _>(Magic)?;
        let version = p.take::<u32, _>(Version)?.to_le();
        let bank_address = p.take::<u32, _>(BankAddress)?.to_le();
        let unknown_1 = p.take::<u32, _>(Unknown1)?.to_le();

        Ok(BigHeader {
            magic,
            version,
            bank_address,
            unknown_1,
        })
    }

    pub fn compile(
        &self,
        s: &mut BinarySerializer,
    ) -> Result<(), BinarySerializerError<BigHeaderPart>> {
        use BigHeaderPart::*;

        s.put(&self.magic, Magic)?;
        s.put(&self.version.to_le(), Version)?;
        s.put(&self.bank_address.to_le(), BankAddress)?;
        s.put(&self.unknown_1.to_le(), Unknown1)?;

        Ok(())
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
pub enum BigBankIndexPart {
    BanksCount,
    Name,
    BankId,
    BankEntriesCount,
    IndexStart,
    IndexSize,
    BlockSize,
}

impl<'a> BigBankIndex<'a> {
    pub fn byte_size(&self) -> usize {
        25 + self.name.len()
    }

    pub fn parse(p: &mut BinaryParser) -> Result<Self, BinaryParserError<BigBankIndexPart>> {
        use BigBankIndexPart::*;

        let banks_count = p.take::<u32, _>(BanksCount)?.to_le();
        let name = null_terminated_buf(i).ok_or(Name)?;
        let bank_id = p.take::<u32, _>(BankId)?.to_le();
        let bank_entries_count = p.take::<u32, _>(BankEntriesCount)?.to_le();
        let index_start = p.take::<u32, _>(IndexStart)?.to_le();
        let index_size = p.take::<u32, _>(IndexSize)?.to_le();
        let block_size = p.take::<u32, _>(BlockSize)?.to_le();

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

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigBankIndexPart> {
        use BigBankIndexPart::*;

        put(out, &self.banks_count.to_le()).ok_or(BanksCount)?;
        out.take_mut(..self.name.len())
            .ok_or(Name)?
            .clone_from_slice(&self.name);
        put(out, b"\0").ok_or(Name)?;
        put(out, &self.bank_id.to_le()).ok_or(BankId)?;
        put(out, &self.bank_entries_count.to_le()).ok_or(BankEntriesCount)?;
        put(out, &self.index_start.to_le()).ok_or(IndexStart)?;
        put(out, &self.index_size.to_le()).ok_or(IndexSize)?;
        put(out, &self.block_size.to_le()).ok_or(BlockSize)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct BigFileIndex<'a> {
    pub file_type: u32,
    pub types_map: Vec<[u32; 2]>,
    pub entries: Vec<BigFileEntry<'a>>,
}

#[derive(Debug)]
pub enum BigFileIndexPart {
    TypesCount,
    FileType,
    EntriesCount,
    TypesMap,
    Entry(BigFileEntryPart),
}

impl<'a> BigFileIndex<'a> {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &'a [u8]) -> Result<Self, BigFileIndexPart> {
        use BigFileIndexPart::*;

        let types_count = take::<u32>(i).ok_or(TypesCount)?.to_le();
        let file_type = take::<u32>(i).ok_or(FileType)?.to_le();
        let entries_count = take::<u32>(i).ok_or(EntriesCount)?.to_le();

        let types_map_count = (types_count - 1) as usize;

        let mut types_map = Vec::with_capacity(types_map_count);

        for _ in 0..types_map_count {
            let v1 = take::<u32>(i).ok_or(TypesMap)?.to_le();
            let v2 = take::<u32>(i).ok_or(TypesMap)?.to_le();
            types_map.push([v1, v2]);
        }

        let mut entries = Vec::with_capacity(entries_count.try_into().unwrap());

        for _ in 0..entries_count {
            let entry = BigFileEntry::parse(i).map_err(BigFileIndexPart::Entry)?;
            entries.push(entry);
        }

        Ok(BigFileIndex {
            file_type,
            types_map,
            entries,
        })
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigFileIndexPart> {
        use BigFileIndexPart::*;

        let types_count = u32::try_from(self.types_map.len()).or(Err(TypesCount))?;
        let types_count = types_count + 1;

        put(out, &types_count.to_le()).ok_or(TypesCount)?;
        put(out, &self.file_type.to_le()).ok_or(FileType)?;

        let entries_count = u32::try_from(self.entries.len()).or(Err(EntriesCount))?;

        put(out, &entries_count.to_le()).ok_or(EntriesCount)?;

        for entry in &self.types_map {
            put(out, entry).ok_or(TypesMap)?;
        }

        for entries in &self.entries {
            entries.compile(out).map_err(Entry)?;
        }

        Ok(())
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
pub enum BigFileEntryPart {
    Magic,
    Id,
    FileType,
    Size,
    Start,
    FileTypeDev,
    SymbolName,
    Crc,
    FilesCount,
    Files,
    SubHeaderSize,
    SubHeader(BigSubHeaderPart),
}

impl<'a> BigFileEntry<'a> {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &'a [u8]) -> Result<Self, BigFileEntryPart> {
        use BigFileEntryPart::*;

        let magic = take::<u32>(i).ok_or(Magic)?.to_le();
        let id = take::<u32>(i).ok_or(Id)?.to_le();
        let file_type = take::<u32>(i).ok_or(FileType)?.to_le();
        let size = take::<u32>(i).ok_or(Size)?.to_le();
        let start = take::<u32>(i).ok_or(Start)?.to_le();
        let file_type_dev = take::<u32>(i).ok_or(FileTypeDev)?.to_le();

        let symbol_name = run_le_u32_buf(i).ok_or(SymbolName)?;

        let crc = take::<u32>(i).ok_or(Crc)?.to_le();

        let files_count = take::<u32>(i).ok_or(FilesCount)?.to_le();
        let files_count = usize::try_from(files_count).or(Err(FilesCount))?;
        let mut files = Vec::with_capacity(files_count);

        for _ in 0..files_count {
            let name = run_le_u32_buf(i).ok_or(Files)?;
            files.push(name);
        }

        let mut sub_header_bytes = run_le_u32_buf(i).ok_or(SubHeaderSize)?;
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

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigFileEntryPart> {
        use BigFileEntryPart::*;

        put(out, &self.magic.to_le()).ok_or(Magic)?;
        put(out, &self.id.to_le()).ok_or(Id)?;
        put(out, &self.file_type.to_le()).ok_or(FileType)?;
        put(out, &self.size.to_le()).ok_or(Size)?;
        put(out, &self.start.to_le()).ok_or(Start)?;
        put(out, &self.file_type_dev.to_le()).ok_or(FileTypeDev)?;

        let symbol_name_size = u32::try_from(self.symbol_name.len()).or(Err(SymbolName))?;

        put(out, &symbol_name_size.to_le()).ok_or(SymbolName)?;

        out.take_mut(..self.symbol_name.len())
            .ok_or(SymbolName)?
            .copy_from_slice(&self.symbol_name);

        put(out, &self.crc.to_le()).ok_or(Crc)?;

        let files_count = u32::try_from(self.files.len()).or(Err(Files))?;

        put(out, &files_count.to_le()).ok_or(Files)?;

        for name in &self.files {
            let name_size = u32::try_from(name.len()).or(Err(Files))?;

            put(out, &name_size.to_le()).ok_or(Files)?;

            out.take_mut(..name.len())
                .ok_or(Files)?
                .copy_from_slice(&name)
        }

        let sub_header_size = self.sub_header.byte_size();

        put(out, &sub_header_size.to_le()).ok_or(SubHeaderSize)?;

        self.sub_header
            .compile(out)
            .map_err(BigFileEntryPart::SubHeader)?;

        Ok(())
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
pub enum BigSubHeaderPart {
    Texture(BigSubHeaderTexturePart),
    Mesh(BigSubHeaderMeshPart),
    Animation(BigSubHeaderAnimationPart),
    Unknown,
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

    pub fn parse(i: &mut &[u8], file_type: u32) -> Result<Self, BigSubHeaderPart> {
        use BigSubHeaderPart::*;

        Ok(match file_type {
            0 => Self::Texture(BigSubHeaderTexture::parse(i).map_err(Texture)?),
            1 | 2 | 4 | 5 => Self::Mesh(BigSubHeaderMesh::parse(i).map_err(Mesh)?),
            // ? =>
            //     decode_big_sub_header_anim(input),
            // ? =>
            //     Ok((b"", BigSubHeader::None)),
            _ => Self::Unknown(i.to_vec()),
        })
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderPart> {
        use BigSubHeaderPart::*;
        Ok(match self {
            Self::None => {}
            Self::Texture(x) => {
                x.compile(out).map_err(Texture)?;
            }
            Self::Mesh(x) => {
                x.compile(out).map_err(Mesh)?;
            }
            Self::Animation(x) => {
                x.compile(out).map_err(Animation)?;
            }
            Self::Unknown(x) => {
                out.take_mut(..x.len()).ok_or(Unknown)?.copy_from_slice(x);
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
pub enum BigSubHeaderTexturePart {
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

impl BigSubHeaderTexture {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubHeaderTexturePart> {
        use BigSubHeaderTexturePart::*;

        let width = take::<u16>(i).ok_or(Width)?.to_le();
        let height = take::<u16>(i).ok_or(Height)?.to_le();
        let depth = take::<u16>(i).ok_or(Depth)?.to_le();
        let frame_width = take::<u16>(i).ok_or(FrameWidth)?.to_le();
        let frame_height = take::<u16>(i).ok_or(FrameHeight)?.to_le();
        let frame_count = take::<u16>(i).ok_or(FrameCount)?.to_le();
        let dxt_compression = take::<u16>(i).ok_or(DxtCompression)?.to_le();
        let unknown1 = take::<u16>(i).ok_or(Unknown1)?.to_le();
        let transparency = take::<u8>(i).ok_or(Transparency)?.to_le();
        let mip_maps = take::<u8>(i).ok_or(MipMaps)?.to_le();
        let unknown2 = take::<u16>(i).ok_or(Unknown2)?.to_le();
        let top_mip_map_size = take::<u32>(i).ok_or(TopMipMapSize)?.to_le();
        let top_mip_map_compressed_size = take::<u32>(i).ok_or(TopMipMapCompressedSize)?.to_le();
        let unknown3 = take::<u16>(i).ok_or(Unknown3)?.to_le();
        let unknown4 = take::<u32>(i).ok_or(Unknown4)?.to_le();

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

    pub fn compile(&self, _out: &mut &mut [u8]) -> Result<(), BigSubHeaderTexturePart> {
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
pub enum BigSubHeaderMeshPart {
    PhysicsMesh,
    Unknown1,
    SizeCompressedLod,
    Padding,
    Unknown2,
    TextureIds,
}

impl BigSubHeaderMesh {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubHeaderMeshPart> {
        use BigSubHeaderMeshPart::*;

        let physics_mesh = take::<u32>(i).ok_or(PhysicsMesh)?.to_le();

        let unknown1 = take::<[f32; 10]>(i).ok_or(Unknown1)?;

        let size_compressed_lod_count = take::<u32>(i).ok_or(SizeCompressedLod)?.to_le();
        let size_compressed_lod_count =
            usize::try_from(size_compressed_lod_count).or(Err(SizeCompressedLod))?;
        let mut size_compressed_lod = Vec::with_capacity(size_compressed_lod_count);

        for _ in 0..size_compressed_lod_count {
            let unknown = take::<u32>(i).ok_or(SizeCompressedLod)?.to_le();
            size_compressed_lod.push(unknown);
        }

        let padding = take::<u32>(i).ok_or(Padding)?.to_le();

        let unknown2_count = size_compressed_lod_count - 1;
        let mut unknown2 = Vec::with_capacity(unknown2_count);

        for _ in 0..unknown2_count {
            let unknown = take::<u32>(i).ok_or(Unknown2)?.to_le();
            unknown2.push(unknown);
        }

        let texture_ids_count = take::<u32>(i).ok_or(TextureIds)?.to_le();
        let texture_ids_count = usize::try_from(texture_ids_count).or(Err(TextureIds))?;
        let mut texture_ids = Vec::with_capacity(texture_ids_count);

        for _ in 0..texture_ids_count {
            let texture_id = take::<u32>(i).ok_or(TextureIds)?.to_le();
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

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderMeshPart> {
        use BigSubHeaderMeshPart::*;

        put(out, &self.physics_mesh.to_le()).ok_or(PhysicsMesh)?;
        put(out, &self.unknown1).ok_or(Unknown1)?;

        let size_compressed_lod_count =
            u32::try_from(self.size_compressed_lod.len()).or(Err(SizeCompressedLod))?;

        put(out, &size_compressed_lod_count).ok_or(SizeCompressedLod)?;

        for size_compressed_lod in &self.size_compressed_lod {
            put(out, &size_compressed_lod.to_le()).ok_or(SizeCompressedLod)?;
        }

        put(out, &self.padding).ok_or(Padding)?;

        // let unknown2_count = self.size_compressed_lod.len() - 1;

        for v in &self.unknown2 {
            put(out, &v.to_le()).ok_or(Unknown2)?;
        }

        let texture_ids_count = u32::try_from(self.texture_ids.len()).or(Err(TextureIds))?;

        put(out, &texture_ids_count).ok_or(TextureIds)?;

        for texture_id in &self.texture_ids {
            put(out, &texture_id.to_le()).ok_or(TextureIds)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderAnimation {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}

#[derive(Debug)]
pub enum BigSubHeaderAnimationPart {
    Unknown1,
    Unknown2,
    Unknown3,
}

impl BigSubHeaderAnimation {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigSubHeaderAnimationPart> {
        use BigSubHeaderAnimationPart::*;

        let unknown1 = take::<f32>(i).ok_or(Unknown1)?;
        let unknown2 = take::<f32>(i).ok_or(Unknown2)?;
        let unknown3 = i.to_vec();

        Ok(Self {
            unknown1,
            unknown2,
            unknown3,
        })
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderAnimationPart> {
        use BigSubHeaderAnimationPart::*;

        put(out, &self.unknown1).ok_or(Unknown1)?;
        put(out, &self.unknown2).ok_or(Unknown2)?;

        out.take_mut(..self.unknown3.len())
            .ok_or(Unknown3)?
            .copy_from_slice(&self.unknown3);

        Ok(())
    }
}
