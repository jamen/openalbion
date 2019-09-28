use nom::IResult;
use nom::number::complete::le_u32;
use chrono::naive::{NaiveDateTime,NaiveDate,NaiveTime};

pub fn parse_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime> {
    let (input, year) = le_u32(input)?;
    let (input, month) = le_u32(input)?;
    let (input, day) = le_u32(input)?;
    let (input, hour) = le_u32(input)?;
    let (input, minute) = le_u32(input)?;
    let (input, second) = le_u32(input)?;
    let (input, millisecond) = le_u32(input)?;

    let ymd = NaiveDate::from_ymd(year as i32, month, day);
    let hms = NaiveTime::from_hms_milli(hour, minute, second, millisecond);
    let date_time = NaiveDateTime::new(ymd, hms);

    Ok((input, date_time))
}

pub fn parse_short_timestamp(input: &[u8]) -> IResult<&[u8], NaiveDateTime> {
    let (input, year) = le_u32(input)?;
    let (input, month) = le_u32(input)?;
    let (input, day) = le_u32(input)?;
    let (input, hour) = le_u32(input)?;
    let (input, minute) = le_u32(input)?;

    let ymd = NaiveDate::from_ymd(year as i32, month, day);
    let hms = NaiveTime::from_hms(hour, minute, 0);
    let date_time = NaiveDateTime::new(ymd, hms);

    Ok((input, date_time))
}