use crate::BytesExt;

use views::{View,Bytes,OutOfBounds};

#[derive(Debug)]
pub struct Big {
    pub magic_number: String,
    pub version: u32,
    pub bank_address: u32,
    pub banks_count: u32,
    pub banks: Vec<BigBank>,

}

#[derive(Debug)]
pub struct BigBank {
    pub path: String,
    pub unknown_1: u32,
    pub entries_total: u32,
    pub index_start: u32,
    pub index_size: u32,
    pub block_size: u32,
    pub index: BigIndex
}

#[derive(Debug)]
pub struct BigIndex {
    pub file_type_count: u32,
    pub file_types: Vec<(u32,u32)>,
    pub entries: Vec<BigEntry>,
}

#[derive(Debug)]
pub struct BigEntry {
    pub magic_number: u32,
    pub id: u32,
    pub kind: u32,
    pub data_size: u32,
    pub data_start: u32,
    pub data: Vec<u8>,
    pub kind_2: u32,
    pub symbol: String,
    pub crc: u32,
    pub source_file_count: u32,
    pub source_file_paths: Vec<String>,
    // TODO: Figure this out
    pub sub_header_size: u32,
    pub sub_header: Vec<u8>,
}

impl Big {
    pub fn decode<'a>(source: &[u8]) -> Result<Big, OutOfBounds> {
        let mut header = source.clone();

        let magic_number = header.take_as_str(4)?.to_owned();
        let version = header.take_u32_le()?;
        let bank_address = header.take_u32_le()?;

        let mut bank_source = source.get(bank_address as usize..).ok_or(OutOfBounds)?;
        let banks_count = bank_source.take_u32_le()?;
        let mut banks = Vec::new();

        while banks.len() < banks_count as usize {
            let path = std::str::from_utf8(bank_source.take_until_nul()?).map_err(|_| OutOfBounds)?.to_owned();
            let unknown_1 = bank_source.take_u32_le()?;
            let entries_total = bank_source.take_u32_le()?;
            let index_start = bank_source.take_u32_le()?;
            let index_size = bank_source.take_u32_le()?;
            let block_size = bank_source.take_u32_le()?;

            // println!("path {:?}", path);
            // println!("bank_entries_count {:?}", bank_entries_count);
            // println!("index_start {:?}", index_start);
            // println!("index_size {:?}", index_size);
            // println!("block_size {:?}", block_size);
            // println!("unknown_1 {:?}", unknown_1);
            // println!("entries_total {:?}", entries_total);

            let mut index_source = source
                .get(index_start as usize .. index_start as usize + index_size as usize)
                .ok_or(OutOfBounds)?;

            let file_type_count = index_source.take_u32_le()?;
            // let file_type = index_source.take_u32_le()?;
            // println!("file_type_count {:?}", file_type_count);
            // println!("file_type {:?}", file_type);
            let mut file_types = Vec::new();

            while file_types.len() < file_type_count as usize {
                file_types.push((
                    index_source.take_u32_le()?,
                    index_source.take_u32_le()?,
                ));
            }

            let mut entries = Vec::new();

            while entries.len() < entries_total as usize {
                let magic_number = index_source.take_u32_le()?;
                let id = index_source.take_u32_le()?;
                let kind = index_source.take_u32_le()?;
                let data_size = index_source.take_u32_le()?;
                let data_start = index_source.take_u32_le()?;
                let data = source
                    .get(data_start as usize .. data_start as usize + data_size as usize)
                    .ok_or(OutOfBounds)?
                    .to_owned();
                let kind_2 = index_source.take_u32_le()?;
                let symbol = index_source.take_as_str_with_u32_le_prefix()?.to_owned();
                let crc = index_source.take_u32_le()?;

                let source_file_count = index_source.take_u32_le()?;
                let mut source_file_paths = Vec::new();

                while source_file_paths.len() < source_file_count as usize {
                    source_file_paths.push(index_source.take_as_str_with_u32_le_prefix()?.to_owned());
                }

                let sub_header_size = index_source.take_u32_le()?;
                let sub_header = index_source.take(sub_header_size as usize)?.to_owned();

                entries.push(BigEntry {
                    magic_number,
                    id,
                    kind,
                    data_size,
                    data_start,
                    data,
                    kind_2,
                    symbol,
                    crc,
                    source_file_count,
                    source_file_paths,
                    sub_header_size,
                    sub_header,
                });
            }

            let index = BigIndex {
                file_type_count,
                file_types,
                entries,
            };

            banks.push(BigBank {
                path,
                unknown_1,
                entries_total,
                index_start,
                index_size,
                block_size,
                index,
            });
        }

        Ok(
            Big {
                magic_number,
                version,
                bank_address,
                banks_count,
                banks,
            }
        )
    }
}

// use std::io::{Read,Seek,SeekFrom};

// use super::{
//     Error,
//     Decode,
//     Entry,
//     IResult,
//     // all_consuming,
//     count,
//     decode_bytes_as_utf8_string,
//     decode_rle_string,
//     is_not,
//     // le_f32,
//     // le_u16,
//     le_u32,
//     // le_u8,
//     tag,
//     take,
//     tuple,
// };

// #[derive(Debug,PartialEq)]
// pub struct Big {
//     pub header: BigHeader,
//     pub bank: BigBankIndex,
//     pub entries: BigIndex,
// }

// #[derive(Debug,PartialEq)]
// pub struct BigHeader {
//     pub version: u32,
//     pub bank_address: u32,
// }

// #[derive(Debug,PartialEq)]
// pub struct BigBankIndex {
//     pub name: String,
//     pub bank_id: u32,
//     pub bank_entries_count: u32,
//     pub index_start: u32,
//     pub index_size: u32,
//     pub block_size: u32,
// }

// #[derive(Debug,PartialEq)]
// pub struct BigIndex {
//     // pub file_types_count: u32,
//     // pub file_type: u32,
//     // pub entries_count: u32,
//     pub unknown_types_map: Vec<(u32, u32)>,
//     pub entries: Vec<BigEntry>,
// }

// #[derive(Debug,PartialEq)]
// pub struct BigEntry {
//     pub magic_number: u32,
//     pub id: u32,
//     pub file_type: u32,
//     pub size: u32,
//     pub start: u32,
//     pub file_type_dev: u32,
//     pub symbol_name: String,
//     pub crc: u32,
//     pub files: Vec<String>,
//     // pub sub_header: BigSubHeader,
//     pub sub_header: Vec<u8>,
// }

// #[derive(Debug,PartialEq)]
// pub enum BigSubHeader {
//     None,
//     Texture(BigSubHeaderTexture),
//     Mesh(BigSubHeaderMesh),
//     Animation(BigSubHeaderAnimation),
//     Unknown(Vec<u8>),
// }

// #[derive(Debug,PartialEq)]
// pub struct BigSubHeaderTexture {
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

// #[derive(Debug,PartialEq)]
// pub struct BigSubHeaderMesh {
//     pub physics_mesh: u32,
//     pub unknown1: Vec<f32>,

//     pub size_compressed_lod: Vec<u32>,
//     pub padding: u32,
//     pub unknown2: Vec<u32>,
//     pub texture_ids: Vec<u32>,
// }

// #[derive(Debug,PartialEq)]
// pub struct BigSubHeaderAnimation {
//     pub unknown1: f32,
//     pub unknown2: f32,
//     pub unknown3: Vec<u8>
// }

// impl Decode for Big {
//     type Error = Error;

//     fn decode<S: Read + Seek>(source: &mut S) -> Result<Big, Error> {
//         let mut header: [u8; 16] = [0; 16];

//         source.read(&mut header)?;

//         let (_, header) = Big::decode_header(&header[..])?;

//         let mut bank_index: Vec<u8> = Vec::new();
//         source.seek(SeekFrom::Start(header.bank_address as u64))?;
//         source.read_to_end(&mut bank_index)?;

//         let (_, bank) = Big::decode_bank_index(&bank_index)?;

//         let mut file_index: Vec<u8> = Vec::new();
//         source.seek(SeekFrom::Start(bank.index_start as u64))?;
//         source.take(bank.index_size as u64).read_to_end(&mut file_index)?;

//         let (_, entries) = Big::decode_file_index(&file_index)?;

//         Ok(
//             Big {
//                 header: header,
//                 bank: bank,
//                 entries: entries,
//             }
//         )
//     }
// }

// impl Big {
//     pub fn decode_header(input: &[u8]) -> IResult<&[u8], BigHeader, Error> {
//         let (input, _magic_number) = tag("BIGB")(input)?;
//         let (input, version) = le_u32(input)?;
//         let (input, bank_address) = le_u32(input)?;
//         let (input, _unknown_1) = le_u32(input)?;

//         Ok(
//             (
//                 input,
//                 BigHeader {
//                     version: version,
//                     bank_address: bank_address,
//                 }
//             )
//         )
//     }

//     pub fn decode_bank_index(input: &[u8]) -> IResult<&[u8], BigBankIndex, Error> {
//         let (input, _banks_count) = le_u32(input)?;
//         let (input, name) = is_not("\0")(input)?;
//         let (input, _zero) = tag("\0")(input)?;
//         let (input, bank_id) = le_u32(input)?;

//         let (_, name) = decode_bytes_as_utf8_string(&name)?;

//         let (input, bank_entries_count) = le_u32(input)?;
//         let (input, index_start) = le_u32(input)?;
//         let (input, index_size) = le_u32(input)?;
//         let (input, block_size) = le_u32(input)?;

//         Ok(
//             (
//                 input,
//                 BigBankIndex {
//                     name: name,
//                     bank_id: bank_id,
//                     bank_entries_count: bank_entries_count,
//                     index_start: index_start,
//                     index_size: index_size,
//                     block_size: block_size,
//                 }
//             )
//         )
//     }

//     pub fn decode_file_index(input: &[u8]) -> IResult<&[u8], BigIndex, Error> {
//         let (input, file_types_count) = le_u32(input)?;
//         let (input, _file_type) = le_u32(input)?;
//         let (input, entries_count) = le_u32(input)?;

//         // println!("file_types_count {:?}", file_types_count);
//         // println!("file_type {:?}", file_type);
//         // println!("entries_count {:?}", entries_count);
//         // let entries_count = 10;

//         // println!("{:?}", &input[..60]);

//         // Lots of integers not documented in fabletlcmod.com
//         // let (input, _unknown_1) = take(56usize)(input)?;

//         let (input, unknown_types_map) = count(tuple((le_u32, le_u32)), (file_types_count - 1) as usize)(input)?;

//         let (input, entries) = count(Self::decode_file_index_entry, entries_count as usize)(input)?;

//         Ok(
//             (
//                 input,
//                 BigIndex {
//                     // file_types_count: file_types_count,
//                     // file_type: file_type,
//                     unknown_types_map: unknown_types_map,
//                     entries: entries,
//                     // entries_count: entries_count,
//                 }
//             )
//         )
//     }

//     pub fn decode_file_index_entry(input: &[u8]) -> IResult<&[u8], BigEntry, Error> {
//         let (input, magic_number) = le_u32(input)?;
//         let (input, id) = le_u32(input)?;
//         let (input, file_type) = le_u32(input)?;
//         let (input, size) = le_u32(input)?;
//         let (input, start) = le_u32(input)?;
//         let (input, file_type_dev) = le_u32(input)?;

//         // println!("magic_number {:?}", magic_number);
//         // println!("id {:?}", id);
//         // println!("file_type {:?}", file_type);
//         // println!("size {:?}", size);
//         // println!("start {:?}", start);
//         // println!("file_type_dev {:?}", file_type_dev);

//         let (input, symbol_name_length) = le_u32(input)?;
//         let (input, symbol_name) = take(symbol_name_length as usize)(input)?;

//         let (_, symbol_name) = decode_bytes_as_utf8_string(&symbol_name)?;

//         let (input, crc) = le_u32(input)?;

//         let (input, files_count) = le_u32(input)?;
//         let (input, files) = count(decode_rle_string, files_count as usize)(input)?;

//         let (input, sub_header_size) = le_u32(input)?;
//         let (input, sub_header) = take(sub_header_size as usize)(input)?;

//         // let (_, sub_header) = all_consuming(Self::decode_big_sub_header(file_type))(sub_header)?;

//         Ok(
//             (
//                 input,
//                 BigEntry {
//                     magic_number: magic_number,
//                     id: id,
//                     file_type: file_type,
//                     size: size,
//                     start: start,
//                     files: files,
//                     file_type_dev: file_type_dev,
//                     symbol_name: symbol_name,
//                     crc: crc,
//                     sub_header: sub_header.to_vec(),
//                 }
//             )
//         )
//     }

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

// impl BigEntry {
//     pub fn source<S: Read + Seek>(&self, source: S) -> Result<Entry<S>, Error> {
//         Ok(Entry::new(source, self.start as u64, self.start as u64 + self.size as u64)?)
//     }
// }