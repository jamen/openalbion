use fable_base::nom::IResult;
use fable_base::nom::number::complete::le_u32;
use fable_base::nom::bytes::complete::tag;
use fable_base::string::decode_rle_string;

use crate::stb::{
    StbHeader,
    StbDevHeader,
};

pub fn decode_header(input: &[u8]) -> IResult<&[u8], StbHeader> {
    let (input, _magic_number) = tag("BBBB")(input)?;
    let (input, version) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?;
    let (input, _unknown_2) = le_u32(input)?;
    let (input, header_size) = le_u32(input)?;
    let (input, files_count) = le_u32(input)?;
    let (input, levels_count) = le_u32(input)?;
    let (input, developer_listings) = le_u32(input)?;

    Ok(
        (
            input,
            StbHeader {
                version: version,
                header_size: header_size,
                files_count: files_count,
                levels_count: levels_count,
                developer_listings: developer_listings,
            }
        )
    )
}

pub fn decode_developer_header(input: &[u8]) -> IResult<&[u8], StbDevHeader> {
    let (input, listing_start) = le_u32(input)?;
    let (input, file_id) = le_u32(input)?;
    let (input, _null) = le_u32(input)?;
    let (input, file_size) = le_u32(input)?;
    let (input, offset) = le_u32(input)?;
    let (input, _null) = le_u32(input)?;

    println!("listing_start {:?}", listing_start);
    println!("file_id {:?}", file_id);
    println!("file_size {:?}", file_size);
    println!("offset {:?}", offset);

    let (input, file_name) = decode_rle_string(input)?;

    let (input, _null) = le_u32(input)?;
    let (input, _unknown_1) = le_u32(input)?;

    let (input, file_name_2) = decode_rle_string(input)?;

    let (input, bytes_left) = le_u32(input)?;

    let (input, _unknown_2) = le_u32(input)?;
    let (input, _unknown_3) = le_u32(input)?;
    let (input, _null) = le_u32(input)?;
    let (input, _unknown_4) = le_u32(input)?;

    Ok(
        (
            input,
            StbDevHeader {
                listing_start: listing_start,
                file_id: file_id,
                file_size: file_size,
                offset: offset,
                file_name: file_name,
                file_name_2: file_name_2,
                bytes_left: bytes_left,
            }
        )
    )
}