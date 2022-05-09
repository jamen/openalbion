mod error;
// mod script;

pub use error::*;
// pub use script::*;

use nom::{
    bytes::{
        complete::{take, take_till},
        streaming::tag,
    },
    combinator::all_consuming,
    number::complete::le_u32,
    IResult,
};

// Generic parsers

pub fn decode_rle_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, size) = le_u32(input)?;
    let (input, string) = take(size as usize)(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8_string)(&string)?;
    Ok((input, string))
}

pub fn decode_null_terminated_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, string) = take_till(|c| c == b'\0')(input)?;
    let (input, _nul) = tag(b"\0")(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8_string)(&string)?;
    Ok((input, string))
}

pub fn decode_bytes_as_utf8_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    match String::from_utf8(input.to_vec()) {
        Ok(string) => Ok((b"", string)),
        Err(_error) => Err(nom::Err::Error(Error::Utf8Error)),
    }
}
