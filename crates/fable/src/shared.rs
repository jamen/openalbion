use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::{take,take_till};
use nom::combinator::all_consuming;

use super::Error;

pub fn decode_rle_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, size) = le_u32(input)?;
    let (input, string) = take(size as usize)(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8)(&string)?;
    Ok((input, string))
}

pub fn decode_null_terminated_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, string) = take_till(|c| c == b'\0')(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8)(&string)?;
    Ok((input, string))
}

pub fn decode_bytes_as_utf8(input: &[u8]) -> IResult<&[u8], String, Error> {
    match String::from_utf8(input.to_vec()) {
        Ok(string) => Ok((b"", string)),
        Err(_error) => Err(nom::Err::Error(Error::Utf8Error))
    }
}