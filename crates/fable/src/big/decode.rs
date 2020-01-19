// use std::fs::File;
// use std::io::{Read,Seek};
// use std::io::SeekFrom;
// use nom::IResult;
// use nom::number::complete::{le_u8,le_u32,le_u16,le_f32};
// use nom::bytes::complete::{tag,take,is_not};
// use nom::multi::count;
// use nom::combinator::all_consuming;
// use nom::sequence::tuple;
// use crate::shared::string::{decode_rle_string};

// use super::Big;

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

// pub fn decode_header(input: &[u8]) -> IResult<&[u8], BigHeader> {
//     let (input, _magic_number) = tag("BIGB")(input)?;
//     let (input, version) = le_u32(input)?;
//     let (input, bank_address) = le_u32(input)?;
//     let (input, _unknown_1) = le_u32(input)?;

//     Ok(
//         (
//             input,
//             BigHeader {
//                 version: version,
//                 bank_address: bank_address,
//             }
//         )
//     )
// }

// pub fn decode_bank_index(input: &[u8]) -> IResult<&[u8], BigBankIndex> {
//     let (input, _banks_count) = le_u32(input)?;
//     let (input, name) = is_not("\0")(input)?;
//     let (input, _zero) = tag("\0")(input)?;
//     let (input, bank_id) = le_u32(input)?;

//     let name = match String::from_utf8(name.to_vec()) {
//         Ok(name) => name,
//         Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
//     };

//     let (input, bank_entries_count) = le_u32(input)?;
//     let (input, index_start) = le_u32(input)?;
//     let (input, index_size) = le_u32(input)?;
//     let (input, block_size) = le_u32(input)?;

//     Ok(
//         (
//             input,
//             BigBankIndex {
//                 name: name,
//                 bank_id: bank_id,
//                 bank_entries_count: bank_entries_count,
//                 index_start: index_start,
//                 index_size: index_size,
//                 block_size: block_size,
//             }
//         )
//     )
// }

// pub fn decode_file_index(input: &[u8]) -> IResult<&[u8], BigFileIndex> {
//     let (input, file_types_count) = le_u32(input)?;
//     let (input, _file_type) = le_u32(input)?;
//     let (input, entries_count) = le_u32(input)?;

//     // println!("file_types_count {:?}", file_types_count);
//     // println!("file_type {:?}", file_type);
//     // println!("entries_count {:?}", entries_count);
//     // let entries_count = 10;

//     // println!("{:?}", &input[..60]);

//     // Lots of integers not documented in fabletlcmod.com
//     // let (input, _unknown_1) = take(56usize)(input)?;

//     let (input, unknown_types_map) = count(tuple((le_u32, le_u32)), (file_types_count - 1) as usize)(input)?;

//     let (input, entries) = count(decode_file_index_entry, entries_count as usize)(input)?;

//     Ok(
//         (
//             input,
//             BigFileIndex {
//                 // file_types_count: file_types_count,
//                 // file_type: file_type,
//                 unknown_types_map: unknown_types_map,
//                 entries: entries,
//                 // entries_count: entries_count,
//             }
//         )
//     )
// }

// pub fn decode_file_index_entry(input: &[u8]) -> IResult<&[u8], BigFileEntry> {
//     let (input, magic_number) = le_u32(input)?;
//     let (input, id) = le_u32(input)?;
//     let (input, file_type) = le_u32(input)?;
//     let (input, size) = le_u32(input)?;
//     let (input, start) = le_u32(input)?;
//     let (input, file_type_dev) = le_u32(input)?;

//     // println!("magic_number {:?}", magic_number);
//     // println!("id {:?}", id);
//     // println!("file_type {:?}", file_type);
//     // println!("size {:?}", size);
//     // println!("start {:?}", start);
//     // println!("file_type_dev {:?}", file_type_dev);

//     let (input, symbol_name_length) = le_u32(input)?;
//     let (input, symbol_name) = take(symbol_name_length as usize)(input)?;

//     let symbol_name = match String::from_utf8(symbol_name.to_vec()) {
//         Ok(name) => name,
//         Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::IsNot)))
//     };

//     let (input, crc) = le_u32(input)?;

//     let (input, files_count) = le_u32(input)?;
//     let (input, files) = count(decode_rle_string, files_count as usize)(input)?;

//     let (input, sub_header_size) = le_u32(input)?;
//     let (input, sub_header) = take(sub_header_size as usize)(input)?;

//     // println!("\"{:?} {:?}\" -> {:?}", symbol_name, file_type);
//     println!("file_type {:?} file_type_dev {:?} symbol_name {:?}", file_type, file_type_dev, symbol_name);

//     let (_, sub_header) = all_consuming(decode_big_sub_header(file_type))(sub_header)?;

//     println!("sub_header {:?}", sub_header);

//     Ok(
//         (
//             input,
//             BigFileEntry {
//                 magic_number: magic_number,
//                 id: id,
//                 file_type: file_type,
//                 size: size,
//                 start: start,
//                 files: files,
//                 file_type_dev: file_type_dev,
//                 symbol_name: symbol_name,
//                 crc: crc,
//                 sub_header: sub_header,
//             }
//         )
//     )
// }

// pub fn decode_big_sub_header(file_type: u32) -> impl Fn(&[u8]) -> IResult<&[u8], BigSubHeader> {
//     move |input: &[u8]| {
//         match file_type {
//             0 =>
//                 decode_big_sub_header_texture(input),
//             1 | 2 | 4 | 5 =>
//                 decode_big_sub_header_mesh(input),
//             // ? =>
//             //     decode_big_sub_header_anim(input),
//             // ? =>
//             //     Ok((b"", BigSubHeader::None)),
//             _ =>
//                 Ok((b"", BigSubHeader::Unknown(input.to_vec()))),
//         }
//     }
// }

// pub fn decode_big_sub_header_texture(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
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

// pub fn decode_big_sub_header_mesh(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
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

// pub fn decode_big_sub_header_anim(input: &[u8]) -> IResult<&[u8], BigSubHeader> {
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