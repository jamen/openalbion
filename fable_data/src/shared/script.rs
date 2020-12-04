use crate::{
    Error,
    IResult,
    alt,
    decode_bytes_as_utf8_string,
    digit1,
    do_parse,
    line_ending,
    many0,
    opt,
    space0,
    tag,
    tag_no_case,
    take_till,
    take_while,
};


#[derive(Debug,PartialEq)]
pub struct Field {
    pub name: String,
    pub indices: Vec<Index>,
    pub value: Value,
}

#[derive(Debug,PartialEq)]
/// The index of a key such as `Foo[0]` or `Foo[Name]`
pub enum Index {
    /// Accessor such as `Foo[Value]`
    Box(Value),
    /// Accessor such as `Foo.Name`
    Dot(String),
}

#[derive(Debug,PartialEq)]
pub enum Value {
    Integer(isize),
    Float(f32),
    Name(String),
    String(String),
    Bool(bool),
    Call(Call),
    Empty,
}

#[derive(Debug,PartialEq)]
pub struct Call {
    pub name: String,
    pub parameters: Vec<Value>,
}

pub fn decode_field(input: &[u8]) -> IResult<&[u8], Field, Error> {
    do_parse!(input,
        space0 >>
        name: decode_name >>
        indices: decode_indices >>
        space0 >>
        value: decode_value >>
        space0 >>
        tag!(";") >>
        line_endings >>
        (Field { name, indices, value })
    )
}

pub fn line_endings(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>, Error> {
    Ok(many0(line_ending)(input)?)
}

pub fn decode_indices(input: &[u8]) -> IResult<&[u8], Vec<Index>, Error> {
    many0(decode_index)(input)
}

pub fn decode_field_tagged<'a>(name: &'a str)-> impl Fn(&'a [u8]) -> IResult<&[u8], Field, Error> {
    move |input: &[u8]| {
        let (input, field) = decode_field(input)?;

        if field.name == name.to_string() {
            Ok((input, field))
        } else {
            Err(nom::Err::Error(Error::InvalidTagName))
        }
    }
}

pub fn decode_value(input: &[u8]) -> IResult<&[u8], Value, Error> {
    if let (input, Some(float)) = opt(decode_float)(input)? {
        Ok((input, Value::Float(float)))
    }
    else if let (input, Some(integer)) = opt(decode_integer)(input)? {
        Ok((input, Value::Integer(integer)))
    }
    else if let (input, Some(b)) = opt(decode_bool)(input)? {
        Ok((input, Value::Bool(b)))
    }
    else if let (input, Some(string)) = opt(decode_string)(input)? {
        Ok((input, Value::String(string)))
    }
    else if let (input, Some(name)) = opt(decode_name)(input)? {
        Ok((input, Value::Name(name)))
    }
    else if let (input, Some(call)) = opt(decode_call)(input)? {
        Ok((input, Value::Call(call)))
    }
    else {
        Ok((input, Value::Empty))
    }
}

pub fn decode_index(input: &[u8]) -> IResult<&[u8], Index, Error> {
    match input[0] {
        b'.' => {
            let (input, name) = decode_name(&input[1..])?;
            Ok((input, Index::Dot(name)))
        }
        b'[' => {
            let (input, value) = decode_value(&input[1..])?;
            let (input, _) = tag("]")(input)?;
            Ok((input, Index::Box(value)))
        }
        _ => Err(nom::Err::Error(Error::NotIndexed))
    }
}

pub fn decode_integer(input: &[u8]) -> IResult<&[u8], isize, Error> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, number) = digit1(input)?;

    let (_, number) = decode_bytes_as_utf8_string(number)?;

    let number = match number.parse::<isize>() {
        Ok(n) => n,
        Err(_) => return Err(nom::Err::Error(Error::NotAnInteger))
    };

    let number = match sign {
        Some(_) => -number,
        None => number,
    };

    Ok((input, number))
}

pub fn decode_float(input: &[u8]) -> IResult<&[u8], f32, Error> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, whole) = digit1(input)?;
    let (input, _) = tag(".")(input)?;
    let (input, fraction) = digit1(input)?;

    let (_, whole) = decode_bytes_as_utf8_string(whole)?;
    let (_, fraction) = decode_bytes_as_utf8_string(fraction)?;
    let float = whole + "." + &fraction;

    let number = match float.parse::<f32>() {
        Ok(n) => n,
        Err(_) => return Err(nom::Err::Error(Error::NotAnInteger))
    };

    let number = match sign {
        Some(_) => -number,
        None => number,
    };

    Ok((input, number))
}

pub fn decode_bool(input: &[u8]) -> IResult<&[u8], bool, Error> {
    let (input, b) = alt((tag_no_case("TRUE"), tag_no_case("FALSE")))(input)?;

    let b = match b.to_ascii_lowercase().as_slice() {
        b"true" => true,
        b"false" => false,
        _ => unreachable!(),
    };

    Ok((input, b))
}

pub fn decode_string(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, _) = tag("\"")(input)?;
    let (input, string) = take_till(|x| x == b'"')(input)?;
    let (input, _) = tag("\"")(input)?;
    let (_, string) = decode_bytes_as_utf8_string(string)?;
    Ok((input, string))
}

pub fn decode_name(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, name) = take_while(|x: u8| (x as char).is_alphanumeric() || x == b'_')(input)?;

    if name.len() < 1 {
        return Err(nom::Err::Error(Error::NotAName))
    } else {
        let name = decode_bytes_as_utf8_string(name)?.1;

        Ok((input, name))
    }

}

pub fn decode_call(input: &[u8]) -> IResult<&[u8], Call, Error> {
    let (input, name) = decode_name(input)?;
    let (input, _) = tag("(")(input)?;

    let mut parameters: Vec<Value> = Vec::new();

    let mut last = input;

    loop {
        let (input, value) = decode_value(last)?;

        parameters.push(value);

        let (input, cont) = opt(tag(","))(input)?;

        last = input;

        if cont.is_none() {
            let (input, _) = tag(")")(input)?;
            last = input;
            break
        }
    }

    Ok((last, Call { name, parameters }))
}