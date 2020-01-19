use std::io::Read;

use nom::IResult;
use nom::character::complete::line_ending;
use nom::combinator::{opt,all_consuming};
use nom::bytes::complete::tag;
use nom::multi::{many0,many1};

use crate::shared::{Decode,Error,ErrorKind};
use crate::shared::script::{Instr,InstrKey};
use crate::shared::script::decode::{decode_instr_value,decode_instr_key};

use crate::script::tng::Tng;

use super::Gtg;

static NEWMAP: &'static str = "NEWMAP";
static ENDMAP: &'static str = "ENDMAP";

impl Decode for Gtg {
    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, gtg) = all_consuming(Self::decode_gtg)(&input)?;
        Ok(gtg)
    }
}

impl Gtg {
    pub fn decode_gtg(input: &[u8]) -> IResult<&[u8], Gtg, Error> {
        let (maybe_input, maps) = many1(Gtg::decode_gtg_map)(input)?;
        Ok((maybe_input, Gtg { maps: maps }))
    }

    pub fn decode_gtg_map(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (maybe_input, _start) = Gtg::decode_gtg_map_instr(NEWMAP)(input)?;
        let (maybe_input, tng) = Tng::decode_tng(maybe_input)?;
        let (maybe_input, _end) = Gtg::decode_gtg_map_instr(ENDMAP)(maybe_input)?;
        Ok((maybe_input, tng))
    }

    // Note: Variation of shared::script::decode_instr_tag because "NEWMAP" and "ENDMAP" don't use semicolons
    pub fn decode_gtg_map_instr(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], Instr, Error> {
        move |input: &[u8]| {
            let (maybe_input, _line_ending) = many0(line_ending)(input)?;
            let (maybe_input, key) = decode_instr_key(maybe_input)?;
            let (maybe_input, _space) = opt(tag(" "))(maybe_input)?;
            let (maybe_input, value) = decode_instr_value(maybe_input)?;
            let (maybe_input, _line_ending) = many1(line_ending)(maybe_input)?;

           let key_string = match key {
                InstrKey::Name(x) => x,
                InstrKey::Index(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
                InstrKey::Property(_) => return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),
            };

            if key_string != name {
                return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
            }

            Ok((maybe_input, (InstrKey::Name(key_string), value)))
        }
    }
}
