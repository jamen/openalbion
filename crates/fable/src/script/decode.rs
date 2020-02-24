use nom::IResult;
use nom::Err;
use nom::character::complete::{digit1,line_ending,one_of,space0};
use nom::character::{is_digit,is_alphabetic};
use nom::combinator::opt;
use nom::bytes::complete::{tag,take_while1,escaped,is_not};
use nom::branch::alt;
use nom::multi::{many_till,many0,many1};
use nom::sequence::{terminated,preceded,tuple};

use crate::{Error,ErrorKind};
use crate::shared::decode_bytes_as_utf8;

use super::{
    ScriptField,
    ScriptReference,
    ScriptAccessor,
    ScriptValue,
    ScriptCall,
};

//
// Field
//

pub fn decode_field(input: &[u8]) -> IResult<&[u8], ScriptField, Error> {
    let (input, _line_ending) = many0(line_ending)(input)?;
    let (input, reference) = decode_reference(input)?;
    let (input, _space) = space0(input)?;
    let (input, value) = decode_value(input)?;
    let (input, _semicolon) = tag(";")(input)?;
    let (input, _line_ending) = many1(line_ending)(input)?;

    Ok((input, ScriptField { reference: reference, value: value }))
}

pub fn decode_field_named(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], ScriptField, Error> {
    move |input: &[u8]| {
        let (input, field) = decode_field(input)?;

        // let field_name = match field.reference {
        //     ScriptReference::Name(x) => x,
        //     ScriptReference::Property(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
        // };

        if field.reference != ScriptReference::Name(name.to_string()) {
            return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
        }

        Ok((input, field))
    }
}

//
// Reference
//

pub fn decode_reference(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
    alt((
        decode_reference_name,
        decode_reference_property,
    ))(input)
}

pub fn decode_reference_name(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
    let (input, name) = decode_key_name(input)?;
    Ok((input, ScriptReference::Name(name)))
}

pub fn decode_reference_property(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
    let (input, name) = decode_key_name(input)?;
    let (input, access) = many1(decode_accessor)(input)?;

    Ok((input, ScriptReference::Property((name, access))))
}

pub fn decode_accessor(input: &[u8]) -> IResult<&[u8], ScriptAccessor, Error> {
    let (input, accessor) = one_of(".[")(input)?;

    match accessor {
        '.' => decode_accessor_name(input),
        '[' => decode_accessor_index(input),
        _ => Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptProperty))),
    }
}

pub fn decode_accessor_name(input: &[u8]) -> IResult<&[u8], ScriptAccessor, Error> {
    let (input, name) = decode_key_name(input)?;
    Ok((input, ScriptAccessor::Name(name)))
}

pub fn decode_accessor_index(input: &[u8]) -> IResult<&[u8], ScriptAccessor, Error> {
    match decode_number(input) {
        Ok((input, ScriptValue::Number(index))) => Ok((input, ScriptAccessor::Index(index))),
        _ => {
            match decode_key_name(input) {
                Ok((input, index_name)) => Ok((input, ScriptAccessor::IndexName(index_name))),
                Err(_) => Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptProperty)))
            }
        }
    }
}

pub fn decode_key_name(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, key) = take_while1(|x| is_alphabetic(x) || is_digit(x) || x == 0x5f)(input)?;
    let (_, key) = decode_bytes_as_utf8(key)?;
    Ok((input, key))
}

//
// Value
//

pub fn decode_value(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    alt((
        decode_none,
        decode_bool,
        decode_float,
        decode_number,
        decode_big_number,
        decode_string,
        decode_name,
    ))(input)
}

pub fn decode_bool(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, value) = alt((tag("TRUE"), tag("FALSE")))(input)?;
    let value = match value {
        b"TRUE" => true,
        b"FALSE" => false,
        _ => return Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptValue)))
    };
    Ok((input, ScriptValue::Bool(value)))
}

pub fn decode_none(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    match alt((tag(";"), line_ending))(input) {
        Ok((_input, _tag)) => Ok((input, ScriptValue::None)),
        Err(error) => Err(error)
    }
}

pub fn decode_float(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, sign) = opt(alt((tag("-"), tag("+"))))(input)?;
    let (input, (integer_part, dot, fractional_part)) = tuple(( digit1, tag("."), digit1 ))(input)?;

    let value = [ integer_part, dot, fractional_part ].concat();

    let value = if let Some(sign) = sign {
        [ sign, &value ].concat()
    } else {
        value.to_vec()
    };

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    let value = match value.parse::<f32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    Ok((input, ScriptValue::Float(value)))
}

pub fn decode_number(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, sign) = opt(alt((tag("-"), tag("+"))))(input)?;
    let (input, value) = digit1(input)?;

    let value = if let Some(sign) = sign {
        [ sign, value ].concat()
    } else {
        value.to_vec()
    };

    let value = match String::from_utf8(value) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    let value = match value.parse::<i32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    Ok((input, ScriptValue::Number(value)))
}

pub fn decode_big_number(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    let value = match value.parse::<u64>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    Ok((input, ScriptValue::BigNumber(value)))
}

pub fn decode_string(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, _opener) = tag("\"")(input)?;
    let (input, value) = opt(escaped(is_not("\""), '\\', one_of("\"\\")))(input)?;
    let (input, _closer) = tag("\"")(input)?;

    let value = match value {
        Some(value) =>
            match String::from_utf8(value.to_vec()) {
                Ok(value) => value,
                Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
            },
        None => "".to_string(),
    };

    Ok((input, ScriptValue::String(value)))
}

// TODO: Add leniency on space between parameters.

pub fn decode_call(input: &[u8]) -> IResult<&[u8], ScriptCall, Error> {
    let (input, reference) = decode_reference(input)?;
    let (input, _start) = tag("(")(input)?;
    let (input, (mut arguments, ending_argument)) = many_till(
        preceded(space0, terminated(terminated(decode_value, space0), tag(","))),
        preceded(space0, terminated(terminated(decode_value, space0), tag(")")))
    )(input)?;

    arguments.push(ending_argument);

    Ok((input, ScriptCall { reference: reference, arguments: arguments }))
}

pub fn decode_name(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
    let (input, name) = take_while1(|x| (is_alphabetic(x) || is_digit(x) || x == 0x5f || x == 0x20))(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
    };

    Ok((input, ScriptValue::Name(name)))
}