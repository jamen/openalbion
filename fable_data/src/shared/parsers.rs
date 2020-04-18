use crate::nom::{
    IResult,
    ParseError,
    all_consuming,
    le_u32,
    take,
    take_till,
};

use super::Error;

pub fn decode_rle_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, size) = le_u32(input)?;
    let (input, string) = take(size as usize)(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8_string)(&string)?;
    Ok((input, string))
}

pub fn decode_null_terminated_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, string) = take_till(|c| c == b'\0')(input)?;
    let (_, string) = all_consuming(decode_bytes_as_utf8_string)(&string)?;
    Ok((input, string))
}

pub fn decode_bytes_as_utf8_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    match String::from_utf8(input.to_vec()) {
        Ok(string) => Ok((b"", string)),
        Err(_error) => Err(nom::Err::Error(Error::Utf8Error))
    }
}

pub fn recover<'a, I: Clone, O: Sized, E: ParseError<I>>(
    parser: impl Fn(I) -> IResult<I, O, E>,
    original: I,
) -> impl Fn(I) -> IResult<I, O, E> {
    move |input: I| {
        match parser(input) {
            Ok(x) => Ok(x),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(E::append(original.clone(), nom::error::ErrorKind::Alt, e))),
            Err(x) => Err(x)
        }
    }
}