use nom::IResult;
use nom::character::complete::{alphanumeric1,digit1,line_ending};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::bytes::complete::{tag,take_while1};
use nom::branch::alt;
use nom::combinator::peek;
use nom::multi::{many1,many_till};

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: TngInstr,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: TngInstr,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: TngInstr,
    // TODO: Parse instrs more thoroughly into fields, like positions, names, nested sections, etc.
    pub instrs: Vec<TngInstr>
}

pub type TngInstr = (TngKey, TngValue);

#[derive(Debug,PartialEq)]
pub enum TngKey {
    Regular(String),
    Object(Vec<String>),
}

#[derive(Debug,PartialEq)]
pub enum TngValue {
    None,
    String(String),
    Name(String),
    I32(i32),
    U64(u64),
    F32(f32),
    Bool(bool),
    C3DCoordF((f32, f32, f32)),
}

pub fn parse_tng_key(input: &[u8]) -> IResult<&[u8], TngKey> {
    let (input, key) = alphanumeric1(input)?;

    let key = match String::from_utf8(key.to_vec()) {
        Ok(key) => key,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    // TODO: Object keys

    Ok(
        (
            input,
            TngKey::Regular(key)
        )
    )
}

pub fn parse_tng_value(input: &[u8]) -> IResult<&[u8], TngValue> {
    alt((
        parse_tng_value_none,
        parse_tng_value_bool,
        parse_tng_value_f32,
        parse_tng_value_i32,
        parse_tng_value_u64,
    ))(input)
}

pub fn parse_tng_value_bool(input: &[u8]) -> IResult<&[u8], TngValue> {
    let (maybe_input, value) = alt((tag("TRUE"), tag("FALSE")))(input)?;
    let value = match value {
        b"TRUE" => true,
        b"FALSE" => false,
        _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)))
    };
    Ok((maybe_input, TngValue::Bool(value)))
}

pub fn parse_tng_value_none(input: &[u8]) -> IResult<&[u8], TngValue> {
    match peek(tag(";"))(input) {
        Ok((_input, _tag)) => Ok((input, TngValue::None)),
        Err(error) => Err(error)
    }
}

pub fn parse_tng_value_f32(input: &[u8]) -> IResult<&[u8], TngValue> {
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

    Ok((maybe_input, TngValue::F32(value)))
}

pub fn parse_tng_value_i32(input: &[u8]) -> IResult<&[u8], TngValue> {
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

   Ok((maybe_input, TngValue::I32(value)))
}

pub fn parse_tng_value_u64(input: &[u8]) -> IResult<&[u8], TngValue> {
    let (maybe_input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = match value.parse::<u64>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

   Ok((maybe_input, TngValue::U64(value)))
}

pub fn parse_tng_instr(input: &[u8]) -> IResult<&[u8], TngInstr> {
    let (input, key) = parse_tng_key(input)?;
    let (input, _space) = opt(tag(" "))(input)?;
    let (input, value) = parse_tng_value(input)?;
    let (input, _semicolon) = tag(";")(input)?;
    let (input, _line_ending) = many1(line_ending)(input)?;

    Ok((input, (key, value)))
}

pub fn parse_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing> {
    let (input, new_thing) = parse_tng_instr_tag("NewThing".to_string())(input)?;
    let (input, (instrs, _end_instr)) = many_till(parse_tng_instr, parse_tng_instr_tag("EndThing".to_string()))(input)?;
    Ok(
        (
            input,
            TngThing {
                new_thing: new_thing,
                instrs: instrs,
            }
        )
    )
}

pub fn parse_tng_section(input: &[u8]) -> IResult<&[u8], TngSection> {
    let (input, section_start) = parse_tng_instr_tag("XXXSectionStart".to_string())(input)?;
    let (input, (things, _end_instr)) = many_till(parse_tng_thing, parse_tng_instr_tag("XXXSectionEnd".to_string()))(input)?;
    Ok(
        (
            input,
            TngSection {
                section_start: section_start,
                things: things,
            }
        )
    )
}

pub fn parse_tng(input: &[u8]) -> IResult<&[u8], Tng> {
    let (input, version) = parse_tng_instr_tag("Version".to_string())(input)?;
    let (input, sections) = many1(parse_tng_section)(input)?;
    Ok(
        (
            input,
            Tng {
                version: version,
                sections: sections,
            }
        )
    )
}

pub fn parse_tng_instr_tag(name: String) -> impl Fn(&[u8]) -> IResult<&[u8], TngInstr> {
    move |input: &[u8]| {
        let (input, (key, value)) = parse_tng_instr(input)?;

        if key != TngKey::Regular(name.clone()) {
            return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)));
        }

        Ok((input, (key, value)))
    }
}