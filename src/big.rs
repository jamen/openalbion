pub mod decode;
pub mod encode;

#[derive(Debug,PartialEq)]
pub struct Big {
    header: BigHeader,
    bank: BigBankIndex,
    entries: BigFileIndex
}

#[derive(Debug,PartialEq)]
pub struct BigHeader {
    version: u32,
    bank_address: u32,
}

#[derive(Debug,PartialEq)]
pub struct BigBankIndex {
    name: String,
    bank_id: u32,
    bank_entries_count: u32,
    index_start: u32,
    index_size: u32,
    block_size: u32,
}

#[derive(Debug,PartialEq)]
pub struct BigFileIndex {
    // file_types_count: u32,
    // file_type: u32,
    // entries_count: u32,
    unknown_types_map: Vec<(u32, u32)>,
    entries: Vec<BigFileEntry>,
}

#[derive(Debug,PartialEq)]
pub struct BigFileEntry {
    magic_number: u32,
    id: u32,
    file_type: u32,
    size: u32,
    start: u32,
    file_type_dev: u32,
    symbol_name: String,
    crc: u32,
    files: Vec<String>,
    sub_header: BigSubHeader,
    // sub_header: Vec<u8>,
}

#[derive(Debug,PartialEq)]
pub enum BigSubHeader {
    None,
    Texture(BigSubHeaderTexture),
    Mesh(BigSubHeaderMesh),
    Animation(BigSubHeaderAnimation),
    Unknown(Vec<u8>),
}

#[derive(Debug,PartialEq)]
pub struct BigSubHeaderTexture {
    width: u16,
    height: u16,
    depth: u16,
    frame_width: u16,
    frame_height: u16,
    frame_count: u16,
    dxt_compression: u16,
    unknown1: u16,
    transparency: u8,
    mip_maps: u8,
    unknown2: u16,
    top_mip_map_size: u32,
    top_mip_map_compressed_size: u32,
    unknown3: u16,
    unknown4: u32,
}

#[derive(Debug,PartialEq)]
pub struct BigSubHeaderMesh {
    physics_mesh: u32,
    // unknown1: Vec<u8>,
    // unknown1: Vec<u32>,
    unknown1: Vec<f32>,
    size_compressed_lod: Vec<u32>,
    padding: u32,
    unknown2: Vec<u32>,
    texture_ids: Vec<u32>,
}

#[derive(Debug,PartialEq)]
pub struct BigSubHeaderAnimation {
    unknown1: f32,
    unknown2: f32,
    unknown3: Vec<u8>
}

// impl Big {
//     fn from(file: &mut File) -> Option<Big> {
//         let mut header: [u8; 16] = [0; 16];

//         file.read(&mut header).expect("Failed to read file.");

//         let (_, header) = decode_header(&header[..]).expect("Failed to parse header.");

//         let mut bank_index: Vec<u8> = Vec::new();
//         file.seek(SeekFrom::Start(header.bank_address as u64)).expect("Failed to seek file.");
//         file.read_to_end(&mut bank_index).expect("Failed to read file.");

//         let (_, bank) = decode_bank_index(&bank_index).expect("Failed to parse bank index.");

//         let mut file_index: Vec<u8> = Vec::new();
//         file.seek(SeekFrom::Start(bank.index_start as u64)).expect("Failed to seek file.");
//         file.take(bank.index_size as u64).read_to_end(&mut file_index).expect("Failed to read file.");

//         let (_, entries) = decode_file_index(&file_index);

//         Big {
//             header: header,
//             bank: bank,
//             entries: entries,
//         }
//     }
// }