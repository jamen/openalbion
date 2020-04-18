use crate::nom::{
    IResult,
    Err,
    digit1,
    line_ending,
    one_of,
    space0,
    alphanumeric1,
    multispace0,
    is_digit,
    is_alphabetic,
    opt,
    tag,
    take_while1,
    take_until,
    escaped,
    is_not,
    alt,
    many0,
    many1,
    tuple,
};

use crate::{
    Error,
    ErrorKind,
    ScriptExpression,
    ScriptField,
    ScriptReference,
    ScriptAccessor,
    ScriptAccessorPath,
    ScriptCall,
    ScriptMarkup,
    ScriptComment,
    ScriptBinaryOp,
    ScriptBinaryOpKind,
    ScriptValue,
    decode_bytes_as_utf8_string,
    recover,
};

impl ScriptExpression {
    pub fn decode_expression_list(input: &[u8]) -> IResult<&[u8], Vec<ScriptExpression>, Error> {
        many0(Self::decode_expression_list_item)(input)
    }

    pub fn decode_expression_list_item(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let rest = &input;
        let (rest, expression) = recover(Self::decode_expression, input)(rest)?;
        let (rest, expression) = match expression {
            ScriptExpression::ScriptMarkup(i) => (rest, ScriptExpression::ScriptMarkup(i)),
            ScriptExpression::ScriptComment(i) => (rest, ScriptExpression::ScriptComment(i)),
            expression => {
                let (rest, _) = recover(space0, input)(rest)?;
                let (rest, _) = recover(tag(";"), input)(rest)?;
                (rest, expression)
            }
        };
        Ok((rest, expression))
    }

    pub fn decode_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, _) = multispace0(input)?;
        // println!("expression {:?}", String::from_utf8(input[..10].to_vec()));
        let (input, expression) = alt((
            Self::decode_comment_expression,
            Self::decode_markup_expression,
            Self::decode_bool_literal_expression,
            Self::decode_float_literal_expression,
            Self::decode_integer_literal_expression,
            Self::decode_big_integer_literal_expression,
            Self::decode_string_literal_expression,
            Self::decode_call_expression,
            Self::decode_binary_op_expression,
            Self::decode_field_expression,
            Self::decode_name_expression,
        ))(input)?;
        // println!("expression {:?}", expression);
        let (input, _) = multispace0(input)?;
        Ok((input, expression))
    }

    pub fn decode_field_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, field) = ScriptField::decode_field(input)?;
        Ok((input, ScriptExpression::ScriptField(field)))
    }

    pub fn decode_markup_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, markup) = ScriptMarkup::decode_markup(input)?;
        Ok((input, ScriptExpression::ScriptMarkup(markup)))
    }

    pub fn decode_comment_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, comment) = ScriptComment::decode_comment(input)?;
        Ok((input, ScriptExpression::ScriptComment(comment)))
    }

    pub fn decode_binary_op_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, binary_op) = ScriptBinaryOp::decode_binary_op(input)?;
        Ok((input, ScriptExpression::ScriptBinaryOp(binary_op)))
    }

    pub fn decode_call_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, call) = ScriptCall::decode_call(input)?;
        Ok((input, ScriptExpression::ScriptCall(call)))
    }

    pub fn decode_bool_literal_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_bool_literal(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }

    pub fn decode_float_literal_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_float_literal(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }

    pub fn decode_integer_literal_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_integer_literal(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }

    pub fn decode_big_integer_literal_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_big_integer_literal(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }

    pub fn decode_string_literal_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_string_literal(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }

    pub fn decode_name_expression(input: &[u8]) -> IResult<&[u8], ScriptExpression, Error> {
        let (input, value) = ScriptValue::decode_name(input)?;
        Ok((input, ScriptExpression::ScriptValue(value)))
    }
}

impl ScriptField {
    pub fn decode_field(input: &[u8]) -> IResult<&[u8], ScriptField, Error> {
        let (input, _line_ending) = many0(line_ending)(input)?;
        let (input, reference) = ScriptReference::decode_reference(input)?;
        let (input, _space) = space0(input)?;
        let (input, expression) = ScriptExpression::decode_expression(input)?;
        let (input, _line_ending) = multispace0(input)?;

        Ok((input, ScriptField { reference: reference, value: Box::new(expression) }))
    }

    pub fn decode_field_named(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], ScriptField, Error> {
        move |input: &[u8]| {
            let (input, field) = Self::decode_field(input)?;

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
}

impl ScriptReference {
    pub fn decode_reference(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
        alt((
            Self::decode_reference_accessor,
            Self::decode_reference_name,
        ))(input)
    }

    pub fn decode_reference_name(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
        let (input, name) = Self::decode_key_name(input)?;
        Ok((input, ScriptReference::Name(name)))
    }

    pub fn decode_reference_accessor(input: &[u8]) -> IResult<&[u8], ScriptReference, Error> {
        let (input, name) = Self::decode_key_name(input)?;
        let (input, path) = many1(Self::decode_accessor_path)(input)?;
        Ok((input, ScriptReference::ScriptAccessor(ScriptAccessor { name: name, path: path })))
    }

    pub fn decode_accessor_path(input: &[u8]) -> IResult<&[u8], ScriptAccessorPath, Error> {
        let (input, accessor) = one_of(".[")(input)?;

        match accessor {
            '.' => Self::decode_accessor_path_name(input),
            '[' => Self::decode_accessor_path_expression(input),
            _ => Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptProperty))),
        }
    }

    pub fn decode_accessor_path_name(input: &[u8]) -> IResult<&[u8], ScriptAccessorPath, Error> {
        let (input, name) = Self::decode_key_name(input)?;
        Ok((input, ScriptAccessorPath::Name(name)))
    }

    pub fn decode_accessor_path_expression(input: &[u8]) -> IResult<&[u8], ScriptAccessorPath, Error> {
        let (input, expression) = ScriptExpression::decode_expression(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("]")(input)?;
        Ok((input, ScriptAccessorPath::Expression(expression)))
    }

    pub fn decode_key_name(input: &[u8]) -> IResult<&[u8], String, Error> {
        let (input, key) = take_while1(|x| is_alphabetic(x) || is_digit(x) || x == 0x5f)(input)?;
        let (_, key) = decode_bytes_as_utf8_string(key)?;
        Ok((input, key))
    }
}

impl ScriptValue {
    pub fn decode_bool_literal(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
        let (input, value) = alt((tag("TRUE"), tag("FALSE")))(input)?;
        let value = match value {
            b"TRUE" => true,
            b"FALSE" => false,
            _ => return Err(Err::Error(Error::Fable(ErrorKind::InvalidScriptValue)))
        };
        Ok((input, ScriptValue::BoolLiteral(value)))
    }

    pub fn decode_float_literal(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
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

        Ok((input, ScriptValue::FloatLiteral(value)))
    }

    pub fn decode_integer_literal(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
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

        Ok((input, ScriptValue::IntegerLiteral(value)))
    }

    pub fn decode_big_integer_literal(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
        let (input, value) = digit1(input)?;

        let value = match String::from_utf8(value.to_vec()) {
            Ok(value) => value,
            Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
        };

        let value = match value.parse::<u64>() {
            Ok(value) => value,
            Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
        };

        Ok((input, ScriptValue::BigIntegerLiteral(value)))
    }

    pub fn decode_string_literal(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
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

        Ok((input, ScriptValue::StringLiteral(value)))
    }

    pub fn decode_name(input: &[u8]) -> IResult<&[u8], ScriptValue, Error> {
        let (input, name) = take_while1(|x| (is_alphabetic(x) || is_digit(x) || x == 0x5f || x == 0x20))(input)?;

        let name = match String::from_utf8(name.to_vec()) {
            Ok(value) => value,
            Err(_error) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptValue))),
        };

        Ok((input, ScriptValue::Name(name)))
    }
}

impl ScriptCall {
    pub fn decode_call(input: &[u8]) -> IResult<&[u8], ScriptCall, Error> {
        let (input, reference) = ScriptReference::decode_reference(input)?;
        let (input, _) = tag("(")(input)?;

        let mut arguments = Vec::new();
        let mut last_input = input;

        loop {
            let (input, argument) = opt(ScriptExpression::decode_expression)(last_input)?;

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

        Ok((input, ScriptCall { reference: reference, arguments: arguments }))
    }
}

impl ScriptMarkup {
    pub fn decode_markup(input: &[u8]) -> IResult<&[u8], ScriptMarkup, Error> {
        let (input, _) = space0(input)?;
        let (input, _) = tag("<")(input)?;
        let (input, name) = alphanumeric1(input)?;

        let name = match String::from_utf8(name.to_vec()) {
            Ok(s) => s,
            _ => return Err(nom::Err::Error(Error::Utf8Error))
        };

        let (input, _) = tag(">")(input)?;

        let (input, body) = ScriptExpression::decode_expression_list(input)?;

        let (input, _) = multispace0(input)?;
        let closer = &format!("<\\{}>", name)[..];
        let (input, _) = tag(closer)(input)?;

        Ok((input, ScriptMarkup { name: name, body: body }))
    }
}

impl ScriptComment {
    pub fn decode_comment(input: &[u8]) -> IResult<&[u8], ScriptComment, Error> {
        alt((
            Self::decode_line_comment,
            Self::decode_block_comment
        ))(input)
    }

    pub fn decode_line_comment(input: &[u8]) -> IResult<&[u8], ScriptComment, Error> {
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

        Ok((input, ScriptComment::Line(comment)))
    }

    pub fn decode_block_comment(input: &[u8]) -> IResult<&[u8], ScriptComment, Error> {
        let (input, _) = tag("/*")(input)?;
        let (input, comment) = take_until("*/")(input)?;
        let input = &input[2..];

        let comment = match String::from_utf8(comment.to_vec()) {
            Ok(s) => s,
            _ => return Err(nom::Err::Error(Error::Utf8Error))
        };

        Ok((input, ScriptComment::Block(comment)))
    }
}

impl ScriptBinaryOp {
    pub fn decode_binary_op(input: &[u8]) -> IResult<&[u8], ScriptBinaryOp, Error> {
        // A variation of `ScriptExpression::decode_expression` that omits `ScriptExpression::decode_binary_op_expression`.
        let (input, lhs) = alt((
            ScriptExpression::decode_comment_expression,
            ScriptExpression::decode_markup_expression,
            ScriptExpression::decode_bool_literal_expression,
            ScriptExpression::decode_float_literal_expression,
            ScriptExpression::decode_integer_literal_expression,
            ScriptExpression::decode_big_integer_literal_expression,
            ScriptExpression::decode_string_literal_expression,
            ScriptExpression::decode_call_expression,
            // ScriptExpression::decode_binary_op_expression,
            ScriptExpression::decode_field_expression,
            ScriptExpression::decode_name_expression,
        ))(input)?;
        let (input, _) = space0(input)?;
        let (input, kind) = one_of("+-*/|")(input)?;
        let kind = match kind {
            '+' => ScriptBinaryOpKind::Add,
            '-' => ScriptBinaryOpKind::Subtract,
            '*' => ScriptBinaryOpKind::Multiply,
            '/' => ScriptBinaryOpKind::Divide,
            '|' => ScriptBinaryOpKind::BitOr,
            _ => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidScriptBinaryOp)))
        };
        let (input, _) = space0(input)?;
        let (input, rhs) = ScriptExpression::decode_expression(input)?;
        Ok((input, ScriptBinaryOp { kind: kind, lhs: Box::new(lhs), rhs: Box::new(rhs) }))
    }
}