use nom::IResult;
use nom::number::complete::le_u32;
use nom::bytes::complete::take;
use chrono::naive::{NaiveDateTime,NaiveDate,NaiveTime};

pub mod script;

pub fn parse_rle_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, string) = le_u32(input)?;
    let (input, string) = take(string as usize)(input)?;

    let string = match String::from_utf8(string.to_vec()) {
        Ok(name) => name,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    Ok((input, string))
}

pub fn parse_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime> {
    let (input, year) = le_u32(input)?;
    let (input, month) = le_u32(input)?;
    let (input, day) = le_u32(input)?;
    let (input, hour) = le_u32(input)?;
    let (input, minute) = le_u32(input)?;
    let (input, second) = le_u32(input)?;
    let (input, millisecond) = le_u32(input)?;

    Ok(
        (
            input,
            NaiveDateTime::new(
                NaiveDate::from_ymd(year as i32, month, day),
                NaiveTime::from_hms_milli(hour, minute, second, millisecond)
            )
        )
    )
}

pub fn parse_short_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime> {
    let (input, year) = le_u32(input)?;
    let (input, month) = le_u32(input)?;
    let (input, day) = le_u32(input)?;
    let (input, hour) = le_u32(input)?;
    let (input, minute) = le_u32(input)?;

    Ok(
        (
            input,
            NaiveDateTime::new(
                NaiveDate::from_ymd(year as i32, month, day),
                NaiveTime::from_hms(hour, minute, 0)
            )
        )
    )
}