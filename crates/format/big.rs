use crate::{put, take, take_null_terminated, util::take_run_length_le_u32};

#[derive(Debug, PartialEq)]
pub struct BigHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub bank_address: u32,
    pub unknown_1: u32,
}

#[derive(Debug)]
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

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigHeaderPart> {
        use BigHeaderPart::*;

        let &magic = take::<[u8; 4]>(i).ok_or(Magic)?;
        let version = take::<u32>(i).ok_or(Version)?.to_le();
        let bank_address = take::<u32>(i).ok_or(BankAddress)?.to_le();
        let unknown_1 = take::<u32>(i).ok_or(Unknown1)?.to_le();

        Ok(BigHeader {
            magic,
            version,
            bank_address,
            unknown_1,
        })
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigHeaderPart> {
        use BigHeaderPart::*;

        put(out, &self.magic).ok_or(Magic)?;
        put(out, &self.version.to_le()).ok_or(Version)?;
        put(out, &self.bank_address.to_le()).ok_or(BankAddress)?;
        put(out, &self.unknown_1.to_le()).ok_or(Unknown1)?;

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

#[derive(Debug)]
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
        todo!()
    }

    pub fn parse(i: &mut &'a [u8]) -> Result<Self, BigBankIndexPart> {
        use BigBankIndexPart::*;

        let banks_count = take::<u32>(i).ok_or(BanksCount)?.to_le();
        let name = take_null_terminated(i).ok_or(Name)?;
        let bank_id = take::<u32>(i).ok_or(BankId)?.to_le();
        let bank_entries_count = take::<u32>(i).ok_or(BankEntriesCount)?.to_le();
        let index_start = take::<u32>(i).ok_or(IndexStart)?.to_le();
        let index_size = take::<u32>(i).ok_or(IndexSize)?.to_le();
        let block_size = take::<u32>(i).ok_or(BlockSize)?.to_le();

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
pub struct BigFileIndex<'a, 'b> {
    pub file_type: u32,
    pub types_map: Vec<[u32; 2]>,
    pub entries: Vec<BigFileEntry<'a, 'b>>,
}

#[derive(Debug)]
pub enum BigFileIndexPart {
    TypesCount,
    FileType,
    EntriesCount,
    TypesMap,
    Entry(BigFileEntryPart),
}

impl BigFileIndex<'_, '_> {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigFileIndexPart> {
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
pub struct BigFileEntry<'a, 'b> {
    pub magic: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: &'a [u8],
    pub crc: u32,
    pub files: Vec<&'b [u8]>,
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

impl BigFileEntry<'_, '_> {
    pub fn byte_size(&self) -> usize {
        todo!()
    }

    pub fn parse(i: &mut &[u8]) -> Result<Self, BigFileEntryPart> {
        use BigFileEntryPart::*;

        let magic = take::<u32>(i).ok_or(Magic)?.to_le();
        let id = take::<u32>(i).ok_or(Id)?.to_le();
        let file_type = take::<u32>(i).ok_or(FileType)?.to_le();
        let size = take::<u32>(i).ok_or(Size)?.to_le();
        let start = take::<u32>(i).ok_or(Start)?.to_le();
        let file_type_dev = take::<u32>(i).ok_or(FileTypeDev)?.to_le();

        let symbol_name = take_run_length_le_u32(i).ok_or(SymbolName)?;

        let crc = take::<u32>(i).ok_or(Crc)?.to_le();

        let files_count = take::<u32>(i).ok_or(FilesCount)?.to_le();
        let files_count = usize::try_from(files_count).or(Err(FilesCount))?;
        let files = Vec::with_capacity(files_count);

        for _ in 0..files_count {
            let name = take_run_length_le_u32(i).ok_or(Files)?;
        }

        let mut sub_header_bytes = take_run_length_le_u32(i).ok_or(SubHeaderSize)?;
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

        put(out, &sub_header_size.to_le()).ok_or(SubHeaderSize);

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
        match self {
            Self::None => Ok(()),
            Self::Texture(x) => x.compile(out).map_err(Texture),
            Self::Mesh(x) => x.compile(out).map_err(Mesh),
            Self::Animation(x) => x.compile(out).map_err(Animation),
            Self::Unknown(x) => {
                out.take_mut(..x.len()).ok_or(Unknown)?.copy_from_slice(x);
                Ok(())
            }
        }
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
        todo!()
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderTexturePart> {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderMesh {
    pub physics_mesh: u32,
    pub unknown1: Vec<f32>,
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
        todo!()
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderMeshPart> {
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
        todo!()
    }

    pub fn compile(&self, out: &mut &mut [u8]) -> Result<(), BigSubHeaderAnimationPart> {
        todo!()
    }
}

//     // pub fn decode_big_sub_header(file_type: u32) -> impl Fn(&[u8]) -> IResult<&[u8], BigSubHeader, Error> {
//     //     move |input: &[u8]| {
//     //         match file_type {
//     //             0 =>
//     //                 Self::decode_big_sub_header_texture(input),
//     //             1 | 2 | 4 | 5 =>
//     //                 Self::decode_big_sub_header_mesh(input),
//     //             // ? =>
//     //             //     decode_big_sub_header_anim(input),
//     //             // ? =>
//     //             //     Ok((b"", BigSubHeader::None)),
//     //             _ =>
//     //                 Ok((b"", BigSubHeader::Unknown(input.to_vec()))),
//     //         }
//     //     }
//     // }

//     // pub fn decode_big_sub_header_texture(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
//     //     let (input, width) = le_u16(input)?;
//     //     let (input, height) = le_u16(input)?;
//     //     let (input, depth) = le_u16(input)?;
//     //     let (input, frame_width) = le_u16(input)?;
//     //     let (input, frame_height) = le_u16(input)?;
//     //     let (input, frame_count) = le_u16(input)?;
//     //     let (input, dxt_compression) = le_u16(input)?;
//     //     let (input, unknown1) = le_u16(input)?;
//     //     let (input, transparency) = le_u8(input)?;
//     //     let (input, mip_maps) = le_u8(input)?;
//     //     let (input, unknown2) = le_u16(input)?;
//     //     let (input, top_mip_map_size) = le_u32(input)?;
//     //     let (input, top_mip_map_compressed_size) = le_u32(input)?;
//     //     let (input, unknown3) = le_u16(input)?;
//     //     let (input, unknown4) = le_u32(input)?;

//     //     Ok(
//     //         (
//     //             input,
//     //             BigSubHeader::Texture(
//     //                 BigSubHeaderTexture {
//     //                     width: width,
//     //                     height: height,
//     //                     depth: depth,
//     //                     frame_width: frame_width,
//     //                     frame_height: frame_height,
//     //                     frame_count: frame_count,
//     //                     dxt_compression: dxt_compression,
//     //                     unknown1: unknown1,
//     //                     transparency: transparency,
//     //                     mip_maps: mip_maps,
//     //                     unknown2: unknown2,
//     //                     top_mip_map_size: top_mip_map_size,
//     //                     top_mip_map_compressed_size: top_mip_map_compressed_size,
//     //                     unknown3: unknown3,
//     //                     unknown4: unknown4,
//     //                 }
//     //             )
//     //         )
//     //     )
//     // }

//     // pub fn decode_big_sub_header_mesh(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
//     //     // Check if this entry has no subheader.
//     //     if input.len() == 0 {
//     //         return Ok((b"", BigSubHeader::None))
//     //     }

//     //     let (input, physics_mesh) = le_u32(input)?;

//     //     // let (input, unknown1) = count(float, 10)(input)?;
//     //     // let (input, unknown1) = take(40usize)(input)?;
//     //     let (input, unknown1) = count(le_f32, 10usize)(input)?;

//     //     let (input, size_compressed_lod_count) = le_u32(input)?;
//     //     let (input, size_compressed_lod) = count(le_u32, size_compressed_lod_count as usize)(input)?;

//     //     let (input, padding) = le_u32(input)?;

//     //     let (input, unknown2) = count(le_u32, (size_compressed_lod_count - 1) as usize)(input)?;

//     //     let (input, texture_ids_count) = le_u32(input)?;
//     //     let (input, texture_ids) = count(le_u32, texture_ids_count as usize)(input)?;

//     //     Ok(
//     //         (
//     //             input,
//     //             BigSubHeader::Mesh(
//     //                 BigSubHeaderMesh {
//     //                     physics_mesh: physics_mesh,
//     //                     unknown1: unknown1.to_vec(),
//     //                     size_compressed_lod: size_compressed_lod,
//     //                     padding: padding,
//     //                     unknown2: unknown2,
//     //                     texture_ids: texture_ids,
//     //                 }
//     //             )
//     //         )
//     //     )
//     // }

//     // pub fn decode_big_sub_header_anim(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
//     //     let (input, unknown1) = le_f32(input)?;
//     //     let (input, unknown2) = le_f32(input)?;
//     //     let unknown3 = input.to_vec();
//     //     Ok(
//     //         (
//     //             b"",
//     //             BigSubHeader::Animation(
//     //                 BigSubHeaderAnimation {
//     //                     unknown1: unknown1,
//     //                     unknown2: unknown2,
//     //                     unknown3: unknown3,
//     //                 }
//     //             )
//     //         )
//     //     )
//     // }
// }
