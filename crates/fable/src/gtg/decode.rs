use std::io::{Read,Seek};

use nom::IResult;
use nom::character::complete::line_ending;
use nom::combinator::{opt,all_consuming};
use nom::bytes::complete::tag;
use nom::multi::{many0,many1};

use crate::{Decode,Error,ErrorKind};
use crate::script::{ScriptField,ScriptReference};
use crate::script::decode::{decode_reference,decode_value};

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
        let (maybe_input, maps) = many1(Self::decode_gtg_map)(input)?;
        Ok((maybe_input, Gtg { maps: maps }))
    }

    pub fn decode_gtg_map(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (maybe_input, _start) = Self::decode_gtg_field_named(NEWMAP)(input)?;
        let (maybe_input, tng) = Tng::decode_tng(maybe_input)?;
        let (maybe_input, _end) = Self::decode_gtg_field_named(ENDMAP)(maybe_input)?;
        Ok((maybe_input, tng))
    }

    /// This is a variation of `fable::script::decode_tagged_field` because "NEWMAP" and "ENDMAP" don't use semicolons.
    pub fn decode_gtg_field_named(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], ScriptField, Error> {
        move |input: &[u8]| {
            let (maybe_input, _line_ending) = many0(line_ending)(input)?;
            let (maybe_input, reference) = decode_reference(maybe_input)?;
            let (maybe_input, _space) = opt(tag(" "))(maybe_input)?;
            let (maybe_input, value) = decode_value(maybe_input)?;
            let (maybe_input, _line_ending) = many1(line_ending)(maybe_input)?;

           let field_name = match reference {
                ScriptReference::Name(x) => x,
                ScriptReference::Property(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
            };

            if field_name != name {
                return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
            }

            Ok(
                (
                    input,
                    ScriptField {
                        reference: ScriptReference::Name(field_name),
                        value: value
                    }
                )
            )
        }
    }
}
