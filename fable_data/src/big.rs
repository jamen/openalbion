use std::collections::HashMap;
use std::io::{Error as IoError, Read, Seek, SeekFrom};
use std::path::Path;

use crate::Bytes;

#[derive(Debug)]
pub struct Big {
    pub kind: BigKind,
    pub magic_number: String,
    pub version: u32,
    pub banks_start: u32,
    pub banks: Vec<BigBank>,
    pub entries: Vec<BigEntry>,
    pub entries_by_name: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct BigBank {
    pub name: String,
    pub unknown_1: u32,
    pub entries_count: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
    pub file_type_counts: HashMap<u32, u32>,
}

#[derive(Debug)]
pub struct BigEntry {
    pub bank_id: u32,
    pub unknown_1: u32,
    pub id: u32,
    pub group: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub unknown_2: u32,
    pub crc: u32,
    pub sources: Vec<String>,
    pub info: BigInfo,
}

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
    Unknown,
}

#[derive(Debug)]
pub enum BigInfo {
    Mesh(BigMeshInfo),
    Texture(BigTextureInfo),
    Animation(BigAnimationInfo),
    Unknown(Vec<u8>),
    None,
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

#[derive(Debug)]
pub struct BigMeshInfo {
    pub physics_mesh: u32,
    pub unknown_1: Vec<f32>, // 10 floats that are also found in the mesh.
    pub compressed_lod_sizes: Vec<u32>, // length prefixed list
    // pub unknown_2: u32,
    pub texture_ids: Vec<u32>, // length prefixed list
}

#[derive(Debug)]
pub struct BigAnimationInfo {
    unknown: Vec<u8>,
}

impl Big {
    pub fn decode_reader_with_path<T: Read + Seek, P: AsRef<Path>>(
        source: T,
        path: P,
    ) -> Option<Self> {
        let path = path.as_ref();

        let file_name = path
            .file_name()
            .map(|x| x.to_str().map(|x| x.to_lowercase()))
            .flatten();

        let kind = if path.extension().map(|x| x.to_str()) == Some(Some(".fmp")) {
            BigKind::Fmp
        } else {
            match file_name.as_deref() {
                Some("text.big") => BigKind::Text,
                Some("dialogue.big") => BigKind::Dialogue,
                Some("fonts.big") => BigKind::Fonts,
                Some("graphics.big") => BigKind::Graphics,
                Some("textures.big") => BigKind::Textures,
                Some("frontend.big") => BigKind::Frontend,
                Some("shaders.big") => BigKind::Shaders,
                Some("effects.big") => BigKind::Effects,
                _ => BigKind::Unknown,
            }
        };

        Self::decode_reader(source, kind)
    }

    pub fn decode_reader<T: Read + Seek>(mut source: T, kind: BigKind) -> Option<Self> {
        let mut header = &mut [0; 12][..];

        source.read_exact(&mut header).ok()?;

        let magic_number = header.parse_str(4)?.to_owned();

        let version = header.parse_u32_le()?;
        let banks_start = header.parse_u32_le()?;

        // let banks = Self::decode_banks(source, banks_start, kind)?;

        let mut banks_source = Vec::new();

        source.seek(SeekFrom::Start(banks_start as u64)).ok()?;
        source.read_to_end(&mut banks_source).ok()?;

        let mut banks_source = &banks_source[..];
        let banks_count = banks_source.parse_u32_le()?;
        let mut banks = Vec::new();
        let mut entries = Vec::new();
        let mut entries_by_name = HashMap::new();

        for bank_id in 0..banks_count {
            let name = std::str::from_utf8(banks_source.parse_until_nul()?)
                .ok()?
                .to_owned();
            let unknown_1 = banks_source.parse_u32_le()?;
            let entries_count = banks_source.parse_u32_le()?;
            let index_start = banks_source.parse_u32_le()?;
            let index_size = banks_source.parse_u32_le()?;
            let block_size = banks_source.parse_u32_le()?;

            let mut index_source = &mut vec![0; index_size as usize][..];

            source.seek(SeekFrom::Start(index_start as u64)).ok()?;
            source.read_exact(&mut index_source).ok()?;

            let file_types_count = index_source.parse_u32_le()?;
            let mut file_type_counts = HashMap::new();

            while file_type_counts.len() < file_types_count as usize {
                let a = index_source.parse_u32_le()?;
                let b = index_source.parse_u32_le()?;
                file_type_counts.insert(a, b);
            }

            while entries.len() < entries_count as usize {
                let unknown_1 = index_source.parse_u32_le()?;
                let id = index_source.parse_u32_le()?;
                let group = index_source.parse_u32_le()?;
                let data_size = index_source.parse_u32_le()?;
                let data_start = index_source.parse_u32_le()?;
                let unknown_2 = index_source.parse_u32_le()?;
                let name = index_source.parse_str_with_u32_le_prefix()?.to_owned();
                let crc = index_source.parse_u32_le()?;

                // println!("{:?} {:?}", name, group);

                let sources_count = index_source.parse_u32_le()?;
                let mut sources = Vec::new();

                while sources.len() < sources_count as usize {
                    sources.push(index_source.parse_str_with_u32_le_prefix()?.to_owned());
                }

                // TODO: This seems brute forced. Figure out a better way to handle this.

                let info_size = index_source.parse_u32_le()?;
                let info_data = index_source.advance(info_size as usize)?;
                let info = match (kind, group) {
                    // (BigKind::Graphics, 0) => BigInfo::Unknown,
                    // Normal mesh?
                    (BigKind::Graphics, 1) => BigInfo::Mesh(Self::decode_mesh_info(info_data)?),
                    // Flora mesh?
                    (BigKind::Graphics, 2) => BigInfo::Mesh(Self::decode_mesh_info(info_data)?),
                    // Physics mesh
                    (BigKind::Graphics, 3) => BigInfo::Unknown(info_data.to_owned()),
                    (BigKind::Graphics, 4) => BigInfo::Mesh(Self::decode_mesh_info(info_data)?),
                    (BigKind::Graphics, 5) => BigInfo::Mesh(Self::decode_mesh_info(info_data)?),
                    // Normal animation?
                    (BigKind::Graphics, 6) => {
                        BigInfo::Animation(Self::decode_animation_info(info_data)?)
                    }
                    (BigKind::Graphics, 7) => {
                        BigInfo::Animation(Self::decode_animation_info(info_data)?)
                    }
                    // (BigKind::Graphics, 8) => BigInfo::Unknown,
                    (BigKind::Graphics, 9) => {
                        BigInfo::Animation(Self::decode_animation_info(info_data)?)
                    }

                    (BigKind::Textures, 0) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }
                    (BigKind::Textures, 1) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }
                    // Bump maps
                    (BigKind::Textures, 2) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }
                    // (BigKind::Textures, 3) => BigInfo::Unknown,
                    (BigKind::Textures, 3) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }
                    (BigKind::Textures, 4) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }
                    (BigKind::Textures, 5) => {
                        BigInfo::Texture(Self::decode_texture_info(info_data)?)
                    }

                    // (BigKind::Text, 0) => BigInfo::Unknown,
                    (_, _) => BigInfo::Unknown(info_data.to_owned()),
                };

                if entries_by_name.insert(name, entries.len()).is_some() {
                    return None;
                }

                entries.push(BigEntry {
                    bank_id,
                    unknown_1,
                    id,
                    group,
                    data_size,
                    data_start,
                    unknown_2,
                    crc,
                    sources,
                    info,
                });
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

        Some(Big {
            kind,
            magic_number,
            version,
            banks_start,
            banks,
            entries,
            entries_by_name,
        })
    }

    // fn decode_entry_kind(big_kind, group_id) -> BigEntryKind {
    //     match (big_kind) {
    //         ()
    //         _ => BigEntryKind::Unknown,
    //     }
    // }

    fn decode_mesh_info(mut data: &[u8]) -> Option<BigMeshInfo> {
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

        Some(BigMeshInfo {
            physics_mesh,
            unknown_1,
            compressed_lod_sizes,
            // unknown_2,
            texture_ids,
        })
    }

    fn decode_texture_info(mut data: &[u8]) -> Option<BigTextureInfo> {
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

        Some(BigTextureInfo {
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

    fn decode_animation_info(data: &[u8]) -> Option<BigAnimationInfo> {
        Some(BigAnimationInfo {
            unknown: data.to_vec(),
        })
    }

    pub fn read_entry<T: Read + Seek>(
        mut source: T,
        entry: &BigEntry,
        buf: &mut [u8],
    ) -> Result<(), IoError> {
        let max_len = buf.len();
        let read_buf = &mut buf[..(entry.data_size as usize).min(max_len)];
        source.seek(SeekFrom::Start(entry.data_start as u64))?;
        source.read_exact(read_buf)?;
        Ok(())
    }
}
