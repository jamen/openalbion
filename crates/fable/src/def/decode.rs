use std::io::{Read,Seek};

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag,take_while1};
use nom::character::is_alphanumeric;
use nom::character::complete::{space1,line_ending,multispace0};
use nom::sequence::tuple;
use nom::combinator::{opt,all_consuming};
use nom::multi::many0;

use crate::{Decode,Error};
use crate::script::decode::{decode_comment,decode_expression_list};

use super::{
    Def,
    DefItem,
    Definition,
};

impl<T: Read + Seek> Decode<Def> for T {
    fn decode(&mut self) -> Result<Def, Error> {
        let mut input = Vec::new();
        self.read_to_end(&mut input)?;
        let (_, def) = all_consuming(Def::decode_def)(&input)?;
        Ok(def)
    }
}

impl Def {
    fn decode_def(input: &[u8]) -> IResult<&[u8], Def, Error> {
        let (input, body) = many0(Self::decode_def_item)(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, Def { body }))
    }

    fn decode_def_item(input: &[u8]) -> IResult<&[u8], DefItem, Error> {
        let (input, _) = multispace0(input)?;
        alt((Self::decode_def_item_definition, Self::decode_def_item_comment))(input)
    }

    fn decode_def_item_definition(input: &[u8]) -> IResult<&[u8], DefItem, Error> {
        let (input, definition) = Self::decode_definition(input)?;
        Ok((input, DefItem::Definition(definition)))
    }

    fn decode_def_item_comment(input: &[u8]) -> IResult<&[u8], DefItem, Error> {
        let (input, comment) = decode_comment(input)?;
        let (input, _) = opt(line_ending)(input)?;
        Ok((input, DefItem::Comment(comment)))
    }

    fn decode_definition(input: &[u8]) -> IResult<&[u8], Definition, Error> {
        let (input, directive) = alt((tag("#definition_template"), tag("#definition")))(input)?;

        let is_template = match std::str::from_utf8(&directive) {
            Ok("#definition_template") => true,
            _ => false,
        };

        let (input, _) = space1(input)?;

        let (input, group) = Self::decode_definition_name(input)?;

        let group = match String::from_utf8(group.to_vec()) {
            Ok(group) => group,
            _ => return Err(nom::Err::Error(Error::Utf8Error))
        };

        let (input, _) = space1(input)?;

        let (input, name) = Self::decode_definition_name(input)?;

        let name = match String::from_utf8(name.to_vec()) {
            Ok(name) => name,
            _ => return Err(nom::Err::Error(Error::Utf8Error))
        };

        let (input, specializes) = opt(tuple((space1, tag("specialises"), space1, Self::decode_definition_name)))(input)?;

        let specializes = match specializes {
            Some((_space1, _specializes, _space2, specializes)) => {
                match String::from_utf8(specializes.to_vec()) {
                    Ok(specializes) => Some(specializes),
                    _ => return Err(nom::Err::Error(Error::Utf8Error))
                }
            },
            None => None,
        };

        println!("#definition (is_template {}) {} {} specializes {:?}", is_template, group, name, specializes);

        let (input, body) = decode_expression_list(input)?;


        let (input, _) = multispace0(input)?;
        let (input, _) = tag("#end_definition")(input)?;

        Ok(
            (
                input,
                Definition {
                    is_template: is_template,
                    group: group,
                    name: name,
                    specializes: specializes,
                    body: body,
                }
            )
        )
    }

    pub fn decode_definition_name(input: &[u8]) -> IResult<&[u8], &[u8], Error> {
        take_while1(|c| is_alphanumeric(c) || c == b'_')(input)
    }
}