#![feature(slice_take)]

mod compiler;
mod parser;

pub use compiler::*;
pub use parser::*;

#[derive(Debug, PartialEq)]
pub struct Big {
    pub header: BigHeader,
    pub bank: BigBankIndex,
    pub entries: BigFileIndex,
}

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
    pub const TOTAL_BYTE_SIZE: usize = 16;
}

#[derive(Debug, PartialEq)]
pub struct BigBankIndex {
    pub name: String,
    pub bank_id: u32,
    pub bank_entries_count: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
}

pub enum BigBankIndexPart {
    BanksCount,
    NameLen,
    Name,
    BankId,
    BankEntriesCount,
    IndexStart,
    IndexSize,
    BlockSize,
}

#[derive(Debug, PartialEq)]
pub struct BigFileIndex {
    // pub file_types_count: u32,
    // pub file_type: u32,
    // pub entries_count: u32,
    pub unknown_types_map: Vec<(u32, u32)>,
    pub entries: Vec<BigFileEntry>,
}

#[derive(Debug, PartialEq)]
pub struct BigFileEntry {
    pub magic_number: u32,
    pub id: u32,
    pub file_type: u32,
    pub size: u32,
    pub start: u32,
    pub file_type_dev: u32,
    pub symbol_name: String,
    pub crc: u32,
    pub files: Vec<String>,
    // pub sub_header: BigSubHeader,
    pub sub_header: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum BigSubHeader {
    None,
    Texture(BigSubHeaderTexture),
    Mesh(BigSubHeaderMesh),
    Animation(BigSubHeaderAnimation),
    Unknown(Vec<u8>),
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

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderMesh {
    pub physics_mesh: u32,
    pub unknown1: Vec<f32>,

    pub size_compressed_lod: Vec<u32>,
    pub padding: u32,
    pub unknown2: Vec<u32>,
    pub texture_ids: Vec<u32>,
}

#[derive(Debug, PartialEq)]
pub struct BigSubHeaderAnimation {
    pub unknown1: f32,
    pub unknown2: f32,
    pub unknown3: Vec<u8>,
}
