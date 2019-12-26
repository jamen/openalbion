use fable_base::nom::IResult;
use fable_base::nom::Err;
use fable_base::nom::error::ErrorKind;
use fable_base::nom::character::complete::line_ending;
use fable_base::nom::combinator::opt;
use fable_base::nom::bytes::complete::tag;
use fable_base::nom::multi::{many0,many1};

use crate::shared::{Instr,InstrKey};
use crate::shared::decode::{decode_instr_value,decode_instr_key};

use crate::tng::Tng;
use crate::tng::decode::decode_tng;
use crate::gtg::Gtg;

pub fn decode_gtg_map(input: &[u8]) -> IResult<&[u8], Tng> {
    let (maybe_input, _start) = decode_gtg_map_instr("NEWMAP")(input)?;
    let (maybe_input, tng) = decode_tng(maybe_input)?;
    let (maybe_input, _end) = decode_gtg_map_instr("ENDMAP")(maybe_input)?;
    Ok((maybe_input, tng))
}

// "NEWMAP" and "ENDMAP" don't use semicolons, so this is an alternative of parser::util::decode_instr_tag
pub fn decode_gtg_map_instr(name: &'static str) -> impl Fn(&[u8]) -> IResult<&[u8], Instr> {
    move |input: &[u8]| {
        let (maybe_input, _line_ending) = many0(line_ending)(input)?;
        let (maybe_input, key) = decode_instr_key(maybe_input)?;
        let (maybe_input, _space) = opt(tag(" "))(maybe_input)?;
        let (maybe_input, value) = decode_instr_value(maybe_input)?;
        let (maybe_input, _line_ending) = many1(line_ending)(maybe_input)?;

       let key_string = match key {
            InstrKey::Name(x) => x,
            InstrKey::Index(_) => return Err(Err::Error((input, ErrorKind::ParseTo))),
            InstrKey::Property(_) => return Err(Err::Error((input, ErrorKind::ParseTo))),
        };

        // println!("{:?} == {:?}", name, key);

        if key_string != name {
            return Err(Err::Error((input, ErrorKind::ParseTo)));
        }

        Ok((maybe_input, (InstrKey::Name(key_string), value)))
    }
}

pub fn decode_gtg(input: &[u8]) -> IResult<&[u8], Gtg> {
    let (maybe_input, maps) = many1(decode_gtg_map)(input)?;
    Ok((maybe_input, Gtg { maps: maps }))
}