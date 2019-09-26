use nom::IResult;
use nom::character::complete::{alphanumeric1,digit1,line_ending};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::bytes::complete::{tag,take_while1};
use nom::branch::alt;
use nom::combinator::peek;
use nom::multi::{many1};

pub type Instr = (InstrKey, InstrValue);

#[derive(Debug,PartialEq)]
pub enum InstrKey {
    Regular(String),
    Object(Vec<String>),
}

#[derive(Debug,PartialEq)]
pub enum InstrValue {
    None,
    String(String),
    Name(String),
    I32(i32),
    U64(u64),
    F32(f32),
    Bool(bool),
    C3DCoordF((f32, f32, f32)),
}

pub fn parse_instr_key(input: &[u8]) -> IResult<&[u8], InstrKey> {
    let (input, key) = alphanumeric1(input)?;

    let key = match String::from_utf8(key.to_vec()) {
        Ok(key) => key,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    // TODO: Object keys.
    //
    // For example
    //
    // KeyCameras[0].Position C3DCoordF(2.009033,-2.85498,0.508656);
    // KeyCameras[0].LookDirection C3DCoordF(0.259062,0.961777,-0.088723);
    // KeyCameras[0].FOV 0.111111;
    // KeyCameras[0].ShuttleSpeed 1.0;

    Ok(
        (
            input,
            InstrKey::Regular(key)
        )
    )
}

pub fn parse_instr_value(input: &[u8]) -> IResult<&[u8], InstrValue> {
    alt((
        parse_instr_value_none,
        parse_instr_value_bool,
        parse_instr_value_f32,
        parse_instr_value_i32,
        parse_instr_value_u64,
    ))(input)
}

pub fn parse_instr_value_bool(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, value) = alt((tag("TRUE"), tag("FALSE")))(input)?;
    let value = match value {
        b"TRUE" => true,
        b"FALSE" => false,
        _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)))
    };
    Ok((maybe_input, InstrValue::Bool(value)))
}

pub fn parse_instr_value_none(input: &[u8]) -> IResult<&[u8], InstrValue> {
    match peek(tag(";"))(input) {
        Ok((_input, _tag)) => Ok((input, InstrValue::None)),
        Err(error) => Err(error)
    }
}

pub fn parse_instr_value_f32(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (input, negative) = opt(tag("-"))(input)?;
    let (maybe_input, value) = take_while1(|x| is_digit(x) || x == 0x2e)(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = match value.parse::<f32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = if negative.is_none() { value } else { -value };

    Ok((maybe_input, InstrValue::F32(value)))
}

pub fn parse_instr_value_i32(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (input, negative) = opt(tag("-"))(input)?;
    let (maybe_input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = match value.parse::<i32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = if negative.is_none() { value } else { -value };

   Ok((maybe_input, InstrValue::I32(value)))
}

pub fn parse_instr_value_u64(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = match value.parse::<u64>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

   Ok((maybe_input, InstrValue::U64(value)))
}

pub fn parse_instr(input: &[u8]) -> IResult<&[u8], Instr> {
    let (input, key) = parse_instr_key(input)?;
    let (input, _space) = opt(tag(" "))(input)?;
    let (input, value) = parse_instr_value(input)?;
    let (input, _semicolon) = tag(";")(input)?;
    let (input, _line_ending) = many1(line_ending)(input)?;

    Ok((input, (key, value)))
}

pub fn parse_instr_tag(name: String) -> impl Fn(&[u8]) -> IResult<&[u8], Instr> {
    move |input: &[u8]| {
        let (input, (key, value)) = parse_instr(input)?;

        if key != InstrKey::Regular(name.clone()) {
            return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)));
        }

        Ok((input, (key, value)))
    }
}