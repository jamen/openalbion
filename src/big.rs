// From fabletlcmod.com:
//
// Main header
//
// Char [4] 'BIGB'
// Char [4] Version
// Int [4] Bank address
// Int [4] unknown
//
// Bank Index
//
// Int [4] Number of Banks
// Char [x] NULL terminted string
// Int [4] Bank ID
//
// Bank
//
// Int [4] Number of Entries in Bank
// Int [4] Index Start
// Int [4] Index Size
// Int [4] block size
//
// FileIndex
// Header
//
// Int [4] NumberFileTypes
// Int [4] FileType
// Int [4] NumberFiles
// Index
//
// Int [4] Magic Number
// Int [4] File ID
// Int [4] File Type
// Int [4] File Size
// Int [4] File Start
// –[dev info]–
// Int [4] File Type
// Int [4] StringLength;
// Int [StringLength] SymbolName
// Int [4] File CRC
// Int [4] Number of Source Files
// Int [4] StringLength;
// Int [StringLength] Source File Name
// –[/dev info]–
// Int [4] SubHeaderSize
// Int [SubHeaderSize] SubHeader
//
// Texture sub-header:
//
// Int [2] Width; Texture size…
// Int [2] Height;
// Int [2] Depth; …and depth for vol. textures
// Int [2] FrameWidth; Actual image size (may be smaller)
// Int [2] FrameHeight;
// Int [2] FrameCount;
// Int [2] DXT Compression;
// Int [2] Unkown;
// Int [1] Transparency; Number of alpha channels
// Int [1] MipMaps; Number of MipMaps
// Int [2] Unkown;
// Int [4] TopMipmapSize;
// Int [4] TopMipmapCompressedSize;
// Int [2] Unkown; DXT compression again?
// Int [4] Unkown;
//
// The first mipmap of each texture is compressed using lzo1x and the rest are uncompressed.
//
// Mesh Sub-header:
//
// Dword Physics_Mesh;
// Float [10]; matches first 40 bytes of bbm. possibly origin
// Dword Number_LOD's;
// Dword [Number_LOD's] Size_compressed_LOD;
// Dword padding;
// Dword Number_Textures_Assigned; 1 per surface within model
// Dword [Number_Textures_Assigned] Texture_ID; Texture ID as used in the textures.big
//

use nom::IResult;
use nom::number::complete::{le_u8,le_u32,le_u16,le_f32};
use nom::bytes::complete::{tag,take,is_not};
use nom::multi::count;
use nom::combinator::all_consuming;
use nom::sequence::tuple;
use crate::shared::string::{parse_rle_string};

#[derive(Debug,PartialEq)]
pub struct BigHeader {
    version: u32,
    bank_address: u32,
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], BigHeader> {
    let (input, _magic_number) = tag("BIGB")(input)?;
    let (input, version) = le_u32(input)?;
    let (input, bank_address) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?;

    Ok(
        (
            input,
            BigHeader {
                version: version,
                bank_address: bank_address,
            }
        )
    )
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

pub fn parse_bank_index(input: &[u8]) -> IResult<&[u8], BigBankIndex> {
    let (input, _banks_count) = le_u32(input)?;
    let (input, name) = is_not("\0")(input)?;
    let (input, _zero) = tag("\0")(input)?;
    let (input, bank_id) = le_u32(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    };

    let (input, bank_entries_count) = le_u32(input)?;
    let (input, index_start) = le_u32(input)?;
    let (input, index_size) = le_u32(input)?;
    let (input, block_size) = le_u32(input)?;

    Ok(
        (
            input,
            BigBankIndex {
                name: name,
                bank_id: bank_id,
                bank_entries_count: bank_entries_count,
                index_start: index_start,
                index_size: index_size,
                block_size: block_size,
            }
        )
    )
}

#[derive(Debug,PartialEq)]
pub struct BigFileIndex {
    // file_types_count: u32,
    // file_type: u32,
    // entries_count: u32,
    unknown_types_map: Vec<(u32, u32)>,
    entries: Vec<BigFileEntry>,
}

pub fn parse_file_index(input: &[u8]) -> IResult<&[u8], BigFileIndex> {
    let (input, file_types_count) = le_u32(input)?;
    let (input, _file_type) = le_u32(input)?;
    let (input, entries_count) = le_u32(input)?;

    // println!("file_types_count {:?}", file_types_count);
    // println!("file_type {:?}", file_type);
    // println!("entries_count {:?}", entries_count);
    // let entries_count = 10;

    // println!("{:?}", &input[..60]);

    // Lots of integers not documented in fabletlcmod.com
    // let (input, _unknown_1) = take(56usize)(input)?;

    let (input, unknown_types_map) = count(tuple((le_u32, le_u32)), (file_types_count - 1) as usize)(input)?;

    let (input, entries) = count(parse_file_index_entry, entries_count as usize)(input)?;

    Ok(
        (
            input,
            BigFileIndex {
                // file_types_count: file_types_count,
                // file_type: file_type,
                unknown_types_map: unknown_types_map,
                entries: entries,
                // entries_count: entries_count,
            }
        )
    )
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

pub fn parse_file_index_entry(input: &[u8]) -> IResult<&[u8], BigFileEntry> {
    let (input, magic_number) = le_u32(input)?;
    let (input, id) = le_u32(input)?;
    let (input, file_type) = le_u32(input)?;
    let (input, size) = le_u32(input)?;
    let (input, start) = le_u32(input)?;
    let (input, file_type_dev) = le_u32(input)?;

    // println!("magic_number {:?}", magic_number);
    // println!("id {:?}", id);
    // println!("file_type {:?}", file_type);
    // println!("size {:?}", size);
    // println!("start {:?}", start);
    // println!("file_type_dev {:?}", file_type_dev);

    let (input, symbol_name_length) = le_u32(input)?;
    let (input, symbol_name) = take(symbol_name_length as usize)(input)?;

    let symbol_name = match String::from_utf8(symbol_name.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
    };

    let (input, crc) = le_u32(input)?;

    let (input, files_count) = le_u32(input)?;
    let (input, files) = count(parse_rle_string, files_count as usize)(input)?;

    let (input, sub_header_size) = le_u32(input)?;
    let (input, sub_header) = take(sub_header_size as usize)(input)?;

    // println!("\"{:?} {:?}\" -> {:?}", file_type_dev, symbol_name, file_type);
    println!("file_type {:?} file_type_dev {:?} crc {:?} symbol_name {:?}", file_type, file_type_dev, crc, symbol_name);

    let (_, sub_header) = all_consuming(parse_big_sub_header(file_type))(sub_header)?;

    // println!("sub_header {:?}", sub_header);

    Ok(
        (
            input,
            BigFileEntry {
                magic_number: magic_number,
                id: id,
                file_type: file_type,
                size: size,
                start: start,
                files: files,
                file_type_dev: file_type_dev,
                symbol_name: symbol_name,
                crc: crc,
                sub_header: sub_header,
            }
        )
    )
}

#[derive(Debug,PartialEq)]
pub enum BigSubHeader {
    None,
    Texture(BigSubHeaderTexture),
    Mesh(BigSubHeaderMesh),
    Animation(BigSubHeaderAnimation),
    Unknown(Vec<u8>),
}

// This is pretty disgusting :D
// I looked at how the different entry properties are associated to different symbols, but I couldn't find something
// consistent enough for parsing this nicely.

pub fn parse_big_sub_header(file_type: u32) -> impl Fn(&[u8]) -> IResult<&[u8], BigSubHeader> {
    move |input: &[u8]| {
        match file_type {
            1339597992 |
            1310379679 =>
            // 0 =>
                parse_big_sub_header_texture(input),
            69563933 |

            161007522 |
            296707119 |
            321290918 |
            409261624 |
            470996368 |
            496338967 |
            636771168 |
            662552041 |
            813582674 |
            855093211 |
            863040092 |
            914467955 |

            1036253421 |
            1270726879 |
            1112739598 |
            1174434144 |
            1338753911 |
            1401578322 |
            1514262853 |
            1580716781 |
            1605406570 |
            1700313748 |
            1741773853 |
            1901458721 |
            1918586415 |
            1956948750 |

            2023638838 |
            2048369087 |
            2154789095 |
            2309278448 |
            2379401560 |
            2514840277 |
            2555960170 |
            2581563117 |
            2713605530 |
            2738336531 |
            2781369906 |
            2798054716 |
            2806100155 |
            2926088593 |
            2999689865 |

            3074096294 |
            3414281613 |
            3489123877 |
            3619712424 |
            3665681431 |
            3776099438 |
            3818607335 |
            3848583110 |

            4042171892 |
            4103644764 |
            4112514011 |
            4128325845 =>
            // 1 | 2 | 4 | 5 =>
                parse_big_sub_header_mesh(input),
            2188482983 |
            1940626445 =>
                parse_big_sub_header_anim(input),
            0 |
            3099354981 =>
                Ok((b"", BigSubHeader::None)),
            _ =>
                Ok((b"", BigSubHeader::Unknown(input.to_vec()))),
        }
    }
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

pub fn parse_big_sub_header_texture(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
    let (input, width) = le_u16(input)?;
    let (input, height) = le_u16(input)?;
    let (input, depth) = le_u16(input)?;
    let (input, frame_width) = le_u16(input)?;
    let (input, frame_height) = le_u16(input)?;
    let (input, frame_count) = le_u16(input)?;
    let (input, dxt_compression) = le_u16(input)?;
    let (input, unknown1) = le_u16(input)?;
    let (input, transparency) = le_u8(input)?;
    let (input, mip_maps) = le_u8(input)?;
    let (input, unknown2) = le_u16(input)?;
    let (input, top_mip_map_size) = le_u32(input)?;
    let (input, top_mip_map_compressed_size) = le_u32(input)?;
    let (input, unknown3) = le_u16(input)?;
    let (input, unknown4) = le_u32(input)?;

    Ok(
        (
            input,
            BigSubHeader::Texture(
                BigSubHeaderTexture {
                    width: width,
                    height: height,
                    depth: depth,
                    frame_width: frame_width,
                    frame_height: frame_height,
                    frame_count: frame_count,
                    dxt_compression: dxt_compression,
                    unknown1: unknown1,
                    transparency: transparency,
                    mip_maps: mip_maps,
                    unknown2: unknown2,
                    top_mip_map_size: top_mip_map_size,
                    top_mip_map_compressed_size: top_mip_map_compressed_size,
                    unknown3: unknown3,
                    unknown4: unknown4,
                }
            )
        )
    )
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

pub fn parse_big_sub_header_mesh(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
    // Check if this entry has no subheader.
    if input.len() == 0 {
        return Ok((b"", BigSubHeader::None))
    }

    let (input, physics_mesh) = le_u32(input)?;

    // let (input, unknown1) = count(float, 10)(input)?;
    // let (input, unknown1) = take(40usize)(input)?;
    let (input, unknown1) = count(le_f32, 10usize)(input)?;

    let (input, size_compressed_lod_count) = le_u32(input)?;
    let (input, size_compressed_lod) = count(le_u32, size_compressed_lod_count as usize)(input)?;

    let (input, padding) = le_u32(input)?;

    let (input, unknown2) = count(le_u32, (size_compressed_lod_count - 1) as usize)(input)?;

    let (input, texture_ids_count) = le_u32(input)?;
    let (input, texture_ids) = count(le_u32, texture_ids_count as usize)(input)?;

    Ok(
        (
            input,
            BigSubHeader::Mesh(
                BigSubHeaderMesh {
                    physics_mesh: physics_mesh,
                    unknown1: unknown1.to_vec(),
                    size_compressed_lod: size_compressed_lod,
                    padding: padding,
                    unknown2: unknown2,
                    texture_ids: texture_ids,
                }
            )
        )
    )
}

#[derive(Debug,PartialEq)]
pub struct BigSubHeaderAnimation {
    unknown1: f32,
    unknown2: f32,
    unknown3: Vec<u8>
}

pub fn parse_big_sub_header_anim(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
    let (input, unknown1) = le_f32(input)?;
    let (input, unknown2) = le_f32(input)?;
    let unknown3 = input.to_vec();
    Ok(
        (
            b"",
            BigSubHeader::Animation(
                BigSubHeaderAnimation {
                    unknown1: unknown1,
                    unknown2: unknown2,
                    unknown3: unknown3,
                }
            )
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read,Seek};
    use std::io::SeekFrom;

    #[test]
    fn test_big() {
        // let mut file = File::open(concat!(env!("FABLE"), "/data/graphics/graphics.big")).expect("failed to open file.");
        // let mut file = File::open(concat!(env!("FABLE"), "/data/graphics/pc/textures.big")).expect("failed to open file.");
        // let mut file = File::open(concat!(env!("FABLE"), "/data/shaders/pc/shaders.big")).expect("failed to open file.");
        let mut file = File::open(env!("FABLE")).expect("failed to open file.");

        let mut header: [u8; 16] = [0; 16];

        file.read(&mut header).expect("Failed to read file.");

        let (_, big_header) = parse_header(&header[..]).expect("Failed to parse header.");

        // println!("{:?}", big_header);

        let mut bank_index: Vec<u8> = Vec::new();
        file.seek(SeekFrom::Start(big_header.bank_address as u64)).expect("Failed to seek file.");
        file.read_to_end(&mut bank_index).expect("Failed to read file.");

        let (_, big_bank_index) = parse_bank_index(&bank_index).expect("Failed to parse bank index.");

        // println!("{:?}", big_bank_index);

        let mut file_index: Vec<u8> = Vec::new();
        file.seek(SeekFrom::Start(big_bank_index.index_start as u64)).expect("Failed to seek file.");
        file.take(big_bank_index.index_size as u64).read_to_end(&mut file_index).expect("Failed to read file.");
        // file.read_to_end(&mut file_index).expect("Failed to read file.");

        // println!("digraph {{");

        let (_, big_file_index) = match parse_file_index(&file_index) {
            Ok(value) => value,
            Err(nom::Err::Error((_, error))) => return println!("Error {:?}", error),
            Err(nom::Err::Failure((_, error))) => return println!("Error {:?}", error),
            Err(_) => return println!("Error"),
        };

        // println!("}}");

        // println!("{:#?}", big_file_index);
    }
}