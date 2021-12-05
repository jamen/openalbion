use alloc::string::String;
use alloc::vec::Vec;

use crate::Bytes;

/// The kind of big file.
///
/// It seems that not all big files can be parsed the same. Specifically, the `BigInfo` seemingly
/// can't be parsed without knowing what kind of big file it is.
///
/// This might not be needed if something can be found inside the big file that helps parse it
/// correctly.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BigKind {
    Text,
    Dialogue,
    Fonts,
    Graphics,
    Textures,
    Frontend,
    Shaders,
    Effects,
    Fmp,
    Other,
}

impl BigKind {
    pub fn guess_from_file_name<T: AsRef<str>>(file_name: T) -> Self {
        match file_name.as_ref().to_lowercase().split_once(".") {
            Some(("text", "big")) => Self::Text,
            Some(("dialogue", "big")) => Self::Dialogue,
            Some(("fonts", "big")) => Self::Fonts,
            Some(("graphics", "big")) => Self::Graphics,
            Some(("textures", "big")) => Self::Textures,
            Some(("frontend", "big")) => Self::Frontend,
            Some(("shaders", "big")) => Self::Shaders,
            Some(("effects", "big")) => Self::Effects,
            Some((_, "fmp")) => Self::Fmp,
            None | Some(_) => Self::Other,
        }
    }
}

#[derive(Debug)]
pub struct BigHeader {
    pub magic_number: [u8; 4],
    pub version: u32,
    pub banks_start: u32,
}

impl BigHeader {
    pub fn parse<T: AsRef<[u8]>>(source: T) -> Option<Self> {
        let mut source = source.as_ref();

        let magic_number = source.advance(4)?.try_into().ok()?;
        let version = source.parse_u32_le()?;
        let banks_start = source.parse_u32_le()?;

        Some(BigHeader {
            magic_number,
            version,
            banks_start,
        })
    }

    // fn decode_entry_kind(big_kind, group_id) -> BigEntryKind {
    //     match (big_kind) {
    //         ()
    //         _ => BigEntryKind::Unknown,
    //     }
    // }

    // pub fn read_entry<T: Read + Seek>(
    //     mut source: T,
    //     entry: &BigEntry,
    //     buf: &mut [u8],
    // ) -> Result<(), IoError> {
    //     let max_len = buf.len();
    //     let read_buf = &mut buf[..(entry.data_size as usize).min(max_len)];
    //     source.seek(SeekFrom::Start(entry.data_start as u64))?;
    //     source.read_exact(read_buf)?;
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct BigBank {
    pub name: String,
    pub unknown_1: u32,
    pub entries_count: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
    pub file_type_counts: Vec<(u32, u32)>,
}

impl BigBank {
    pub fn parse(source: &mut &[u8]) -> Option<Vec<Self>> {
        let mut source = source.as_ref();

        let banks_count = source.parse_u32_le()?;

        let mut banks = Vec::new();

        for _ in 0..banks_count {
            let name = core::str::from_utf8(source.parse_until_nul()?)
                .ok()?
                .to_owned();
            let unknown_1 = source.parse_u32_le()?;
            let entries_count = source.parse_u32_le()?;
            let index_start = source.parse_u32_le()?;
            let index_size = source.parse_u32_le()?;
            let block_size = source.parse_u32_le()?;

            let file_types_count = source.parse_u32_le()?;
            let mut file_type_counts = Vec::new();

            while file_type_counts.len() < file_types_count as usize {
                let a = source.parse_u32_le()?;
                let b = source.parse_u32_le()?;
                file_type_counts.push((a, b));
            }

            banks.push(BigBank {
                name,
                unknown_1,
                entries_count,
                index_start,
                index_size,
                block_size,
                file_type_counts,
            });
        }

        Some(banks)
    }
}

#[derive(Debug)]
pub struct BigEntry {
    pub unknown_1: u32,
    pub id: u32,
    pub group: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub unknown_2: u32,
    pub name: String,
    pub crc: u32,
    pub sources: Vec<String>,
    pub info: BigInfo,
}

impl BigEntry {
    pub fn parse(data: &mut &[u8], kind: BigKind, bank: &BigBank) -> Option<Vec<Self>> {
        let mut entries = Vec::new();

        while entries.len() < bank.entries_count as usize {
            let unknown_1 = data.parse_u32_le()?;
            let id = data.parse_u32_le()?;
            let group = data.parse_u32_le()?;
            let data_size = data.parse_u32_le()?;
            let data_start = data.parse_u32_le()?;
            let unknown_2 = data.parse_u32_le()?;
            let name = data.parse_str_with_u32_le_prefix()?.to_owned();
            let crc = data.parse_u32_le()?;

            let sources_count = data.parse_u32_le()?;
            let mut sources = Vec::new();

            while sources.len() < sources_count as usize {
                sources.push(data.parse_str_with_u32_le_prefix()?.to_owned());
            }

            let info_size = data.parse_u32_le()?;
            let mut info_data = data.advance(info_size as usize)?;
            let info = BigInfo::parse(&mut info_data, group, kind)?;

            entries.push(Self {
                unknown_1,
                id,
                group,
                data_size,
                data_start,
                unknown_2,
                name,
                crc,
                sources,
                info,
            });
        }

        Some(entries)
    }
}

#[derive(Debug)]
pub struct BigMeshInfo {
    pub physics_mesh: u32,
    pub unknown_1: Vec<f32>, // 10 floats that are also found in the mesh.
    pub compressed_lod_sizes: Vec<u32>, // length prefixed list
    // pub unknown_2: u32,
    pub texture_ids: Vec<u32>, // length prefixed list
}

impl BigMeshInfo {
    fn parse(data: &mut &[u8]) -> Option<Self> {
        // println!("{:?}", data);

        let physics_mesh = data.parse_u32_le()?;

        let mut unknown_1 = Vec::new();
        for _ in 0..10 {
            unknown_1.push(data.parse_f32_le()?)
        }

        let compressed_lod_sizes_count = data.parse_u32_le()?;

        let mut compressed_lod_sizes = Vec::new();
        for _ in 0..compressed_lod_sizes_count as usize {
            compressed_lod_sizes.push(data.parse_u32_le()?);
        }

        // let unknown_2 = info_data.parse_u32_le()?;

        // println!("unknown_2 {:?}", unknown_2);

        let texture_ids_count = data.parse_u32_le()?;

        let mut texture_ids = Vec::new();
        for _ in 0..texture_ids_count as usize {
            texture_ids.push(data.parse_u32_le()?);
        }

        // println!("phyiscs_mesh {:?}", physics_mesh);
        // println!("unknown_1 {:?}", unknown_1);
        // println!("compressed_lod_sizes_count {:?}", compressed_lod_sizes_count);
        // println!("compressed_lod_sizes {:?}", compressed_lod_sizes);
        // println!("texture_ids_count {:?}", texture_ids_count);
        // println!("texture_ids {:?}", texture_ids);

        Some(Self {
            physics_mesh,
            unknown_1,
            compressed_lod_sizes,
            // unknown_2,
            texture_ids,
        })
    }
}

#[derive(Debug)]
pub struct BigTextureInfo {
    pub width: u16,
    pub height: u16,
    pub depth: u16,
    pub frame_width: u16,
    pub frame_height: u16,
    pub frame_count: u16,
    pub dxt_compression: u16,
    pub unknown_1: u16,
    pub alpha_channel_count: u8,
    pub mipmaps: u8,
    pub unknown_2: u16,
    pub first_mipmap_size: u32,
    pub first_mipmap_compressed_size: u32,
    pub unknown_3: u16,
    pub unknown_4: u32,
}

impl BigTextureInfo {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        let width = data.parse_u16_le()?;
        let height = data.parse_u16_le()?;
        let depth = data.parse_u16_le()?;
        let frame_width = data.parse_u16_le()?;
        let frame_height = data.parse_u16_le()?;
        let frame_count = data.parse_u16_le()?;
        let dxt_compression = data.parse_u16_le()?;
        let unknown_1 = data.parse_u16_le()?;
        let alpha_channel_count = data.parse_u8()?;
        let mipmaps = data.parse_u8()?;
        let unknown_2 = data.parse_u16_le()?;
        let first_mipmap_size = data.parse_u32_le()?;
        let first_mipmap_compressed_size = data.parse_u32_le()?;
        let unknown_3 = data.parse_u16_le()?;
        let unknown_4 = data.parse_u32_le()?;

        Some(Self {
            width,
            height,
            depth,
            frame_width,
            frame_height,
            frame_count,
            dxt_compression,
            unknown_1,
            alpha_channel_count,
            mipmaps,
            unknown_2,
            first_mipmap_size,
            first_mipmap_compressed_size,
            unknown_3,
            unknown_4,
        })
    }
}

#[derive(Debug)]
pub struct BigAnimInfo {
    unknown: Vec<u8>,
}

impl BigAnimInfo {
    pub fn parse(data: &mut &[u8]) -> Option<Self> {
        Some(Self {
            unknown: data.to_vec(),
        })
    }
}

#[derive(Debug)]
pub enum BigInfo {
    Mesh(BigMeshInfo),
    Texture(BigTextureInfo),
    Animation(BigAnimInfo),
    Unknown(Vec<u8>),
    None,
}

impl BigInfo {
    pub fn parse(data: &mut &[u8], group: u32, kind: BigKind) -> Option<Self> {
        // TODO: This seems brute forced. Figure out a better way.
        Some(match (kind, group) {
            // (BigKind::Graphics, 0) => BigInfo::Unknown,
            // Normal mesh?
            (BigKind::Graphics, 1) => BigInfo::Mesh(BigMeshInfo::parse(data)?),
            // Flora mesh?
            (BigKind::Graphics, 2) => BigInfo::Mesh(BigMeshInfo::parse(data)?),
            // Physics mesh
            (BigKind::Graphics, 3) => BigInfo::Unknown(data.to_owned()),
            (BigKind::Graphics, 4) => BigInfo::Mesh(BigMeshInfo::parse(data)?),
            (BigKind::Graphics, 5) => BigInfo::Mesh(BigMeshInfo::parse(data)?),
            // Normal animation?
            (BigKind::Graphics, 6) => BigInfo::Animation(BigAnimInfo::parse(data)?),
            (BigKind::Graphics, 7) => BigInfo::Animation(BigAnimInfo::parse(data)?),
            // (BigKind::Graphics, 8) => BigInfo::Unknown,
            (BigKind::Graphics, 9) => BigInfo::Animation(BigAnimInfo::parse(data)?),

            (BigKind::Textures, 0) => BigInfo::Texture(BigTextureInfo::parse(data)?),
            (BigKind::Textures, 1) => BigInfo::Texture(BigTextureInfo::parse(data)?),
            // Bump maps
            (BigKind::Textures, 2) => BigInfo::Texture(BigTextureInfo::parse(data)?),
            // (BigKind::Textures, 3) => BigInfo::Unknown,
            (BigKind::Textures, 3) => BigInfo::Texture(BigTextureInfo::parse(data)?),
            (BigKind::Textures, 4) => BigInfo::Texture(BigTextureInfo::parse(data)?),
            (BigKind::Textures, 5) => BigInfo::Texture(BigTextureInfo::parse(data)?),

            // (BigKind::Text, 0) => BigInfo::Unknown,
            (_, _) => BigInfo::Unknown(data.to_owned()),
        })
    }
}
