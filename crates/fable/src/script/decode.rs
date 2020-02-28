use nom::IResult;
use nom::Err;
use nom::character::complete::{digit1,line_ending,one_of,space0,alphanumeric1,multispace0};
use nom::character::{is_digit,is_alphabetic};
use nom::combinator::opt;
use nom::bytes::complete::{tag,take_while1,take_until,escaped,is_not};
use nom::branch::alt;
use nom::multi::{many0,many1};
use nom::sequence::tuple;

use crate::{Error,ErrorKind};
use crate::shared::decode_bytes_as_utf8;

use super::{
    Expression,
    Field,
    Reference,
    Accessor,
    AccessorPath,
    Call,
    Markup,
    Comment,
    BinaryOp,
    BinaryOpKind,
};

//
// Expression
//

pub fn decode_expression_list(input: &[u8]) -> IResult<&[u8], Vec<Expression>, Error> {
    many0(decode_expression_list_item)(input)
}

pub fn decode_expression_list_item(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, expression) = decode_expression(input)?;
    let (input, expression) = match expression {
        Expression::Markup(i) => (input, Expression::Markup(i)),
        Expression::Comment(i) => (input, Expression::Comment(i)),
        expression => {
            let (input, _) = space0(input)?;
            let (input, _) = tag(";")(input)?;
            (input, expression)
        }
    };
    Ok((input, expression))
}

pub fn decode_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, _) = multispace0(input)?;
    // println!("expression {:?}", String::from_utf8(input[..10].to_vec()));
    let (input, expression) = alt((
        decode_comment_expression,
        decode_markup_expression,
        decode_bool_literal,
        decode_float_literal,
        decode_integer_literal,
        decode_big_integer_literal,
        decode_string_literal,
        decode_call_expression,
        decode_binary_op_expression,
        decode_field_expression,
        decode_name,
    ))(input)?;
    // println!("expression {:?}", expression);
    let (input, _) = multispace0(input)?;
    Ok((input, expression))
}

//
// Field
//

pub fn decode_field_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, field) = decode_field(input)?;
    Ok((input, Expression::Field(field)))
}

pub fn decode_field(input: &[u8]) -> IResult<&[u8], Field, Error> {
    let (input, _line_ending) = many0(line_ending)(input)?;
    let (input, reference) = decode_reference(input)?;
    let (input, _space) = space0(input)?;
    let (input, expression) = decode_expression(input)?;
    let (input, _line_ending) = multispace0(input)?;

    Ok((input, Field { reference: reference, value: Box::new(expression) }))
}

pub fn decode_field_named(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], Field, Error> {
    move |input: &[u8]| {
        let (input, field) = decode_field(input)?;

        // let field_name = match field.reference {
        //     Reference::Name(x) => x,
        //     Reference::Property(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
        // };

        if field.reference != Reference::Name(name.to_string()) {
            return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
        }

        Ok((input, field))
    }
}

//
// Reference
//

pub fn decode_reference(input: &[u8]) -> IResult<&[u8], Reference, Error> {
    alt((
        decode_reference_accessor,
        decode_reference_name,
    ))(input)
}

pub fn decode_reference_name(input: &[u8]) -> IResult<&[u8], Reference, Error> {
    let (input, name) = decode_key_name(input)?;
    Ok((input, Reference::Name(name)))
}

pub fn decode_reference_accessor(input: &[u8]) -> IResult<&[u8], Reference, Error> {
    let (input, name) = decode_key_name(input)?;
    let (input, path) = many1(decode_accessor_path)(input)?;
    Ok((input, Reference::Accessor(Accessor { name: name, path: path })))
}

pub fn decode_accessor_path(input: &[u8]) -> IResult<&[u8], AccessorPath, Error> {
    let (input, accessor) = one_of(".[")(input)?;

    match accessor {
        '.' => decode_accessor_path_name(input),
        '[' => decode_accessor_path_expression(input),
        _ => Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptProperty))),
    }
}

pub fn decode_accessor_path_name(input: &[u8]) -> IResult<&[u8], AccessorPath, Error> {
    let (input, name) = decode_key_name(input)?;
    Ok((input, AccessorPath::Name(name)))
}

pub fn decode_accessor_path_expression(input: &[u8]) -> IResult<&[u8], AccessorPath, Error> {
    let (input, expression) = decode_expression(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, AccessorPath::Expression(expression)))
}

pub fn decode_key_name(input: &[u8]) -> IResult<&[u8], String, Error> {
    let (input, key) = take_while1(|x| is_alphabetic(x) || is_digit(x) || x == 0x5f)(input)?;
    let (_, key) = decode_bytes_as_utf8(key)?;
    Ok((input, key))
}

//
// Value
//

pub fn decode_bool_literal(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, value) = alt((tag("TRUE"), tag("FALSE")))(input)?;
    let value = match value {
        b"TRUE" => true,
        b"FALSE" => false,
        _ => return Err(Err::Error(Error::Fable(ErrorKind::InvalidValue)))
    };
    Ok((input, Expression::BoolLiteral(value)))
}

pub fn decode_float_literal(input: &[u8]) -> IResult<&[u8], Expression, Error> {
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
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    let value = match value.parse::<f32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    Ok((input, Expression::FloatLiteral(value)))
}

pub fn decode_integer_literal(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, sign) = opt(alt((tag("-"), tag("+"))))(input)?;
    let (input, value) = digit1(input)?;

    let value = if let Some(sign) = sign {
        [ sign, value ].concat()
    } else {
        value.to_vec()
    };

    let value = match String::from_utf8(value) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    let value = match value.parse::<i32>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    Ok((input, Expression::IntegerLiteral(value)))
}

pub fn decode_big_integer_literal(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, value) = digit1(input)?;

    let value = match String::from_utf8(value.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    let value = match value.parse::<u64>() {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    Ok((input, Expression::BigIntegerLiteral(value)))
}

pub fn decode_string_literal(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, _opener) = tag("\"")(input)?;
    let (input, value) = opt(escaped(is_not("\""), '\\', one_of("\"\\")))(input)?;
    let (input, _closer) = tag("\"")(input)?;

    let value = match value {
        Some(value) =>
            match String::from_utf8(value.to_vec()) {
                Ok(value) => value,
                Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
            },
        None => "".to_string(),
    };

    Ok((input, Expression::StringLiteral(value)))
}

pub fn decode_name(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, name) = take_while1(|x| (is_alphabetic(x) || is_digit(x) || x == 0x5f || x == 0x20))(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(value) => value,
        Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidValue))),
    };

    Ok((input, Expression::Name(name)))
}

//
// Call
//

pub fn decode_call_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, call) = decode_call(input)?;
    Ok((input, Expression::Call(call)))
}

pub fn decode_call(input: &[u8]) -> IResult<&[u8], Call, Error> {
    let (input, reference) = decode_reference(input)?;
    let (input, _) = tag("(")(input)?;

    let mut arguments = Vec::new();
    let mut last_input = input;

    loop {
        let (input, argument) = opt(decode_expression)(last_input)?;

        if let Some(argument) = argument {
            arguments.push(argument);

            let (input, next) = opt(tag(","))(input)?;

            if next.is_some() {
                last_input = input;
                continue
            }
        }

        let (input, _) = tag(")")(input)?;
        let (input, _) = space0(input)?;

        last_input = input;

        break
    }

    let input = last_input;

    Ok((input, Call { reference: reference, arguments: arguments }))
}

//
// Markup
//

pub fn decode_markup_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, markup) = decode_markup(input)?;
    Ok((input, Expression::Markup(markup)))
}

pub fn decode_markup(input: &[u8]) -> IResult<&[u8], Markup, Error> {
    let (input, _) = space0(input)?;
    let (input, _) = tag("<")(input)?;
    let (input, name) = alphanumeric1(input)?;

    let name = match String::from_utf8(name.to_vec()) {
        Ok(s) => s,
        _ => return Err(nom::Err::Error(Error::Utf8Error))
    };

    let (input, _) = tag(">")(input)?;

    let (input, body) = decode_expression_list(input)?;

    let (input, _) = multispace0(input)?;
    let closer = &format!("<\\{}>", name)[..];
    let (input, _) = tag(closer)(input)?;

    Ok((input, Markup { name: name, body: body }))
}

//
// Comment
//

pub fn decode_comment_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, comment) = decode_comment(input)?;
    Ok((input, Expression::Comment(comment)))
}

pub fn decode_comment(input: &[u8]) -> IResult<&[u8], Comment, Error> {
    alt((decode_line_comment, decode_block_comment))(input)
}

pub fn decode_line_comment(input: &[u8]) -> IResult<&[u8], Comment, Error> {
    let (input, _comment_symbol) = tag("//")(input)?;

    // Searches for a line ending then backtracks before it.

    let mut ending = 0;

    loop {
        if input[ending] == b'\n' {
            if input[ending - 1] == b'\r' {
                ending -= 1;
            }
            break
        }
        ending += 1;
    }

    let comment = &input[..ending];

    let comment = match String::from_utf8(comment.to_vec()) {
        Ok(s) => s,
        _ => return Err(nom::Err::Error(Error::Utf8Error))
    };

    let input = &input[ending..];

    Ok((input, Comment::Line(comment)))
}

pub fn decode_block_comment(input: &[u8]) -> IResult<&[u8], Comment, Error> {
    let (input, _) = tag("/*")(input)?;
    let (input, comment) = take_until("*/")(input)?;
    let input = &input[2..];

    let comment = match String::from_utf8(comment.to_vec()) {
        Ok(s) => s,
        _ => return Err(nom::Err::Error(Error::Utf8Error))
    };

    Ok((input, Comment::Block(comment)))
}

//
// Binary Operation
//

pub fn decode_binary_op_expression(input: &[u8]) -> IResult<&[u8], Expression, Error> {
    let (input, binary_op) = decode_binary_op(input)?;
    Ok((input, Expression::BinaryOp(binary_op)))
}

pub fn decode_binary_op(input: &[u8]) -> IResult<&[u8], BinaryOp, Error> {
    // This needs a special version of `decode_expression` that omits `decode_binary_op`
    let (input, lhs) = alt((
        decode_comment_expression,
        decode_markup_expression,
        decode_bool_literal,
        decode_float_literal,
        decode_integer_literal,
        decode_big_integer_literal,
        decode_string_literal,
        decode_call_expression,
        // decode_binary_op_expression,
        decode_field_expression,
        decode_name,
    ))(input)?;
    let (input, _) = space0(input)?;
    let (input, kind) = one_of("+-*/|")(input)?;
    let kind = match kind {
        '+' => BinaryOpKind::Add,
        '-' => BinaryOpKind::Subtract,
        '*' => BinaryOpKind::Multiply,
        '/' => BinaryOpKind::Divide,
        '|' => BinaryOpKind::BitOr,
        _ => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidBinaryOp)))
    };
    let (input, _) = space0(input)?;
    let (input, rhs) = decode_expression(input)?;
    Ok((input, BinaryOp { kind: kind, lhs: Box::new(lhs), rhs: Box::new(rhs) }))
}