use std::io::{Read,Seek};

use nom::IResult;
use nom::character::complete::line_ending;
use nom::combinator::{opt,all_consuming};
use nom::bytes::complete::tag;
use nom::multi::{many0,many1};

use crate::{Decode,Error,ErrorKind};
use crate::script::{Field,Reference};
use crate::script::{decode_reference,decode_expression};

use crate::Tng;

use super::Gtg;

static NEWMAP: &'static str = "NEWMAP";
static ENDMAP: &'static str = "ENDMAP";

impl<T: Read + Seek> Decode<Gtg> for T {
    fn decode(&mut self) -> Result<Gtg, Error> {
        let mut input = Vec::new();
        self.read_to_end(&mut input)?;
        let (_, gtg) = all_consuming(Gtg::decode_gtg)(&input)?;
        Ok(gtg)
    }
}

impl Gtg {
    pub fn decode_gtg(input: &[u8]) -> IResult<&[u8], Gtg, Error> {
        let (input, maps) = many1(Self::decode_gtg_map)(input)?;
        Ok((input, Gtg { maps: maps }))
    }

    pub fn decode_gtg_map(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (input, _start) = Self::decode_gtg_field_named(NEWMAP)(input)?;
        let (input, tng) = Tng::decode_tng(input)?;
        let (input, _end) = Self::decode_gtg_field_named(ENDMAP)(input)?;
        Ok((input, tng))
    }

    /// This is a variation of `fable::script::decode_tagged_field` because "NEWMAP" and "ENDMAP" don't use semicolons.
    pub fn decode_gtg_field_named(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], Field, Error> {
        move |input: &[u8]| {
            let (input, _line_ending) = many0(line_ending)(input)?;
            let (input, reference) = decode_reference(input)?;
            let (input, _space) = opt(tag(" "))(input)?;
            let (input, expression) = decode_expression(input)?;
            let (input, _line_ending) = many1(line_ending)(input)?;

           let field_name = match reference {
                Reference::Name(x) => x,
                Reference::Accessor(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
            };

            if field_name != name {
                return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
            }

            Ok(
                (
                    input,
                    Field {
                        reference: Reference::Name(field_name),
                        value: Box::new(expression),
                    }
                )
            )
        }
    }
}
