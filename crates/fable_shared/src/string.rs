use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{take,take_till};

pub fn decode_rle_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, string) = le_u32(input)?;
    let (input, string) = take(string as usize)(input)?;

    let string = match String::from_utf8(string.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    Ok((input, string))
}

pub fn decode_null_terminated_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, string) = take_till(|c| c == b'\0')(input)?;

    let string = match String::from_utf8(string.to_vec()) {
        Ok(string) => string,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)))
    };

    Ok((input, string))
}