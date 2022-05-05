use nom::{
    bytes::complete::{is_not, tag, take},
    multi::count,
    number::complete::le_u32,
    sequence::tuple,
    IResult,
};

use crate::{decode_bytes_as_utf8_string, decode_rle_string, Error};

#[derive(Debug, PartialEq)]
pub struct Big {
    pub header: BigHeader,
    pub bank: BigBankIndex,
    pub entries: BigFileIndex,
}

#[derive(Debug, PartialEq)]
pub struct BigHeader {
    pub version: u32,
    pub bank_address: u32,
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

impl Big {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self, Error> {
        let (_, header) = BigHeader::parse(input)?;

        let bank_input = &input[header.bank_address as usize..];

        let (_, bank) = BigBankIndex::parse(bank_input)?;

        let index_input = &input[bank.index_start as usize..][..bank.index_size as usize];

        let (_, entries) = BigFileIndex::parse(index_input)?;

        Ok((
            &[],
            Big {
                header,
                bank,
                entries,
            },
        ))
    }
}

impl BigHeader {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self, Error> {
        let (input, _magic_number) = tag("BIGB")(input)?;
        let (input, version) = le_u32(input)?;
        let (input, bank_address) = le_u32(input)?;
        let (input, _unknown_1) = le_u32(input)?;

        Ok((
            input,
            BigHeader {
                version,
                bank_address,
            },
        ))
    }
}

impl BigBankIndex {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self, Error> {
        let (input, _banks_count) = le_u32(input)?;
        let (input, name) = is_not("\0")(input)?;
        let (input, _zero) = tag("\0")(input)?;
        let (input, bank_id) = le_u32(input)?;

        let (_, name) = decode_bytes_as_utf8_string(name)?;

        let (input, bank_entries_count) = le_u32(input)?;
        let (input, index_start) = le_u32(input)?;
        let (input, index_size) = le_u32(input)?;
        let (input, block_size) = le_u32(input)?;

        Ok((
            input,
            BigBankIndex {
                name,
                bank_id,
                bank_entries_count,
                index_start,
                index_size,
                block_size,
            },
        ))
    }
}

impl BigFileIndex {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self, Error> {
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

        let (input, unknown_types_map) =
            count(tuple((le_u32, le_u32)), (file_types_count - 1) as usize)(input)?;

        let (input, entries) = count(Self::decode_file_index_entry, entries_count as usize)(input)?;

        Ok((
            input,
            BigFileIndex {
                // file_types_count: file_types_count,
                // file_type: file_type,
                unknown_types_map,
                entries,
                // entries_count: entries_count,
            },
        ))
    }

    pub fn decode_file_index_entry(input: &[u8]) -> IResult<&[u8], BigFileEntry, Error> {
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

        let (_, symbol_name) = decode_bytes_as_utf8_string(symbol_name)?;

        let (input, crc) = le_u32(input)?;

        let (input, files_count) = le_u32(input)?;
        let (input, files) = count(decode_rle_string, files_count as usize)(input)?;

        let (input, sub_header_size) = le_u32(input)?;
        let (input, sub_header) = take(sub_header_size as usize)(input)?;

        // let (_, sub_header) = all_consuming(Self::decode_big_sub_header(file_type))(sub_header)?;

        Ok((
            input,
            BigFileEntry {
                magic_number,
                id,
                file_type,
                size,
                start,
                files,
                file_type_dev,
                symbol_name,
                crc,
                sub_header: sub_header.to_vec(),
            },
        ))
    }

    // pub fn decode_big_sub_header(file_type: u32) -> impl Fn(&[u8]) -> IResult<&[u8], BigSubHeader, Error> {
    //     move |input: &[u8]| {
    //         match file_type {
    //             0 =>
    //                 Self::decode_big_sub_header_texture(input),
    //             1 | 2 | 4 | 5 =>
    //                 Self::decode_big_sub_header_mesh(input),
    //             // ? =>
    //             //     decode_big_sub_header_anim(input),
    //             // ? =>
    //             //     Ok((b"", BigSubHeader::None)),
    //             _ =>
    //                 Ok((b"", BigSubHeader::Unknown(input.to_vec()))),
    //         }
    //     }
    // }

    // pub fn decode_big_sub_header_texture(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
    //     let (input, width) = le_u16(input)?;
    //     let (input, height) = le_u16(input)?;
    //     let (input, depth) = le_u16(input)?;
    //     let (input, frame_width) = le_u16(input)?;
    //     let (input, frame_height) = le_u16(input)?;
    //     let (input, frame_count) = le_u16(input)?;
    //     let (input, dxt_compression) = le_u16(input)?;
    //     let (input, unknown1) = le_u16(input)?;
    //     let (input, transparency) = le_u8(input)?;
    //     let (input, mip_maps) = le_u8(input)?;
    //     let (input, unknown2) = le_u16(input)?;
    //     let (input, top_mip_map_size) = le_u32(input)?;
    //     let (input, top_mip_map_compressed_size) = le_u32(input)?;
    //     let (input, unknown3) = le_u16(input)?;
    //     let (input, unknown4) = le_u32(input)?;

    //     Ok(
    //         (
    //             input,
    //             BigSubHeader::Texture(
    //                 BigSubHeaderTexture {
    //                     width: width,
    //                     height: height,
    //                     depth: depth,
    //                     frame_width: frame_width,
    //                     frame_height: frame_height,
    //                     frame_count: frame_count,
    //                     dxt_compression: dxt_compression,
    //                     unknown1: unknown1,
    //                     transparency: transparency,
    //                     mip_maps: mip_maps,
    //                     unknown2: unknown2,
    //                     top_mip_map_size: top_mip_map_size,
    //                     top_mip_map_compressed_size: top_mip_map_compressed_size,
    //                     unknown3: unknown3,
    //                     unknown4: unknown4,
    //                 }
    //             )
    //         )
    //     )
    // }

    // pub fn decode_big_sub_header_mesh(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
    //     // Check if this entry has no subheader.
    //     if input.len() == 0 {
    //         return Ok((b"", BigSubHeader::None))
    //     }

    //     let (input, physics_mesh) = le_u32(input)?;

    //     // let (input, unknown1) = count(float, 10)(input)?;
    //     // let (input, unknown1) = take(40usize)(input)?;
    //     let (input, unknown1) = count(le_f32, 10usize)(input)?;

    //     let (input, size_compressed_lod_count) = le_u32(input)?;
    //     let (input, size_compressed_lod) = count(le_u32, size_compressed_lod_count as usize)(input)?;

    //     let (input, padding) = le_u32(input)?;

    //     let (input, unknown2) = count(le_u32, (size_compressed_lod_count - 1) as usize)(input)?;

    //     let (input, texture_ids_count) = le_u32(input)?;
    //     let (input, texture_ids) = count(le_u32, texture_ids_count as usize)(input)?;

    //     Ok(
    //         (
    //             input,
    //             BigSubHeader::Mesh(
    //                 BigSubHeaderMesh {
    //                     physics_mesh: physics_mesh,
    //                     unknown1: unknown1.to_vec(),
    //                     size_compressed_lod: size_compressed_lod,
    //                     padding: padding,
    //                     unknown2: unknown2,
    //                     texture_ids: texture_ids,
    //                 }
    //             )
    //         )
    //     )
    // }

    // pub fn decode_big_sub_header_anim(input: &[u8]) -> IResult<&[u8], BigSubHeader, Error> {
    //     let (input, unknown1) = le_f32(input)?;
    //     let (input, unknown2) = le_f32(input)?;
    //     let unknown3 = input.to_vec();
    //     Ok(
    //         (
    //             b"",
    //             BigSubHeader::Animation(
    //                 BigSubHeaderAnimation {
    //                     unknown1: unknown1,
    //                     unknown2: unknown2,
    //                     unknown3: unknown3,
    //                 }
    //             )
    //         )
    //     )
    // }
}
