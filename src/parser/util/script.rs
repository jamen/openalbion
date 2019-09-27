use nom::IResult;
use nom::character::complete::{alphanumeric1,digit1,line_ending,one_of,space0};
use nom::character::{is_digit,is_alphabetic};
use nom::combinator::opt;
use nom::bytes::complete::{tag,take_while1,escaped,is_not};
use nom::branch::alt;
use nom::combinator::peek;
use nom::multi::{many_till,many0,many1};
use nom::sequence::{terminated,preceded};

pub type Instr = (InstrKey, InstrValue);

#[derive(Debug,PartialEq)]
pub enum InstrKey {
    Name(String),
    Index(u32),
    Property(Vec<InstrKey>),
}

#[derive(Debug,PartialEq)]
pub enum InstrValue {
    None,
    Bool(bool),
    Number(i32),
    BigNumber(u64),
    Float(f32),
    String(String),
    Call((String, Vec<InstrValue>)),
    Name(String),
}

pub fn parse_instr_key(input: &[u8]) -> IResult<&[u8], InstrKey> {
    alt((
        parse_instr_key_property,
        parse_instr_key_index,
        parse_instr_key_name
    ))(input)
}

pub fn parse_instr_key_property(input: &[u8]) -> IResult<&[u8], InstrKey> {
    let (maybe_input, key_name) = parse_instr_key_name(input)?;
    let (maybe_input, mut parts) = many1(parse_instr_key_property_access)(maybe_input)?;

    parts.insert(0, key_name);

    Ok((maybe_input, InstrKey::Property(parts)))
}

pub fn parse_instr_key_property_access(input: &[u8]) -> IResult<&[u8], InstrKey> {
    let (maybe_input, accessor) = one_of(".[")(input)?;

    if accessor == '[' {
        terminated(parse_instr_key, tag("]"))(maybe_input)
    } else if accessor == '.' {
        parse_instr_key_name(maybe_input)
    } else {
        Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)))
    }
}

pub fn parse_instr_key_index(input: &[u8]) -> IResult<&[u8], InstrKey> {
    let (maybe_input, index) = parse_instr_value_number(input)?;

    let index = match index {
        InstrValue::Number(index) => index,
        _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    Ok((maybe_input, InstrKey::Index(index as u32)))
}

pub fn parse_instr_key_name(input: &[u8]) -> IResult<&[u8], InstrKey> {
    let (maybe_input, key) = take_while1(|x| is_alphabetic(x) || is_digit(x) || x == 0x5f)(input)?;

    let key = match String::from_utf8(key.to_vec()) {
        Ok(key) => key,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    Ok((maybe_input, InstrKey::Name(key)))
}

pub fn parse_instr_value(input: &[u8]) -> IResult<&[u8], InstrValue> {
    alt((
        parse_instr_value_none,
        parse_instr_value_bool,
        parse_instr_value_float,
        parse_instr_value_number,
        parse_instr_value_big_number,
        parse_instr_value_string,
        parse_instr_value_call,
        parse_instr_value_name,
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

pub fn parse_instr_value_float(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (input, negative) = opt(tag("-"))(input)?;
    let (maybe_input, value) = take_while1(|x| is_digit(x) || x == 0x2e)(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    let value = match value.parse::<f32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = if negative.is_none() { value } else { -value };

    Ok((maybe_input, InstrValue::Float(value)))
}

pub fn parse_instr_value_number(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, negative) = opt(tag("-"))(input)?;
    let (maybe_input, value) = digit1(maybe_input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    let value = match value.parse::<i32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

    let value = if negative.is_none() { value } else { -value };

   Ok((maybe_input, InstrValue::Number(value)))
}

pub fn parse_instr_value_big_number(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    let value = match value.parse::<u64>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    };

   Ok((maybe_input, InstrValue::BigNumber(value)))
}

pub fn parse_instr_value_string(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, _opener) = tag("\"")(input)?;
    let (maybe_input, value) = opt(escaped(is_not("\""), '\\', one_of("\"\\")))(maybe_input)?;
    let (maybe_input, _closer) = tag("\"")(maybe_input)?;

    let value = match value {
        Some(value) =>
            match String::from_utf8(value.to_vec()) {
                Ok(value) => value,
                Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
            },
        None => "".to_string(),
    };

    Ok((maybe_input, InstrValue::String(value)))
}

// TODO: Add leniency on space between parameters.

pub fn parse_instr_value_call(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, name) = parse_instr_value_name(input)?;

    let name = match name {
        InstrValue::Name(value) => value,
        _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    let (maybe_input, _start) = tag("(")(maybe_input)?;
    let (maybe_input, (mut values, last)) = many_till(
        preceded(space0, terminated(terminated(parse_instr_value, space0), tag(","))),
        preceded(space0, terminated(terminated(parse_instr_value, space0), tag(")")))
    )(maybe_input)?;

    values.push(last);

    Ok((maybe_input, InstrValue::Call((name, values))))
}

pub fn parse_instr_value_call_tag(name: String) -> impl Fn(&[u8]) -> IResult<&[u8], InstrValue> {
    move |input: &[u8]| {
        let (maybe_input, func) = parse_instr_value_call(input)?;

        let (key, values) = match func {
            InstrValue::Call((key, values)) => (key, values),
            _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
        };

        if key != name.clone() {
            return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)));
        }

        Ok((maybe_input, InstrValue::Call((key, values))))
    }
}

pub fn parse_instr_value_name(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, name) = take_while1(|x| (is_alphabetic(x) || is_digit(x) || x == 0x5f || x == 0x20))(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
    };

    Ok((maybe_input, InstrValue::Name(name)))
}

pub fn parse_instr(input: &[u8]) -> IResult<&[u8], Instr> {
    let (maybe_input, _line_ending) = many0(line_ending)(input)?;
    let (maybe_input, key) = parse_instr_key(maybe_input)?;
    let (maybe_input, _space) = opt(tag(" "))(maybe_input)?;
    let (maybe_input, value) = parse_instr_value(maybe_input)?;
    let (maybe_input, _semicolon) = tag(";")(maybe_input)?;
    let (maybe_input, _line_ending) = many1(line_ending)(maybe_input)?;

    Ok((maybe_input, (key, value)))
}

pub fn parse_instr_tag(name: String) -> impl Fn(&[u8]) -> IResult<&[u8], Instr> {
    move |input: &[u8]| {
        let (maybe_input, (key, value)) = parse_instr(input)?;

        let key = match key {
            InstrKey::Name(x) => x,
            InstrKey::Index(_) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
            InstrKey::Property(_) => return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo))),
        };

        // println!("{:?} == {:?}", name, key);

        if key != name {
            return Err(nom::Err::Error((input, nom::error::ErrorKind::ParseTo)));
        }

        Ok((maybe_input, (InstrKey::Name(key), value)))
    }
}