use std::io::Read;

use nom::IResult;
use nom::multi::{many1,many0};
use nom::character::complete::line_ending;
use nom::bytes::complete::tag;
use nom::combinator::{opt,all_consuming};

use crate::shared::{Decode,Error};
use crate::shared::script::InstrValue;
use crate::shared::script::decode::decode_instr_value_call;

use super::Qst;

impl Decode for Qst {
    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, qst) = all_consuming(Self::decode_qst)(&input)?;
        Ok(qst)
    }
}

impl Qst {
    pub fn decode_qst(input: &[u8]) -> IResult<&[u8], Qst, Error> {
        let (maybe_input, body) = many1(Self::decode_call)(input)?;
        Ok((maybe_input, Qst { body: body }))
    }

    pub fn decode_call(input: &[u8]) -> IResult<&[u8], InstrValue, Error> {
        let (maybe_input, _ln) = opt(many0(line_ending))(input)?;
        let (maybe_input, call) = decode_instr_value_call(maybe_input)?;
        let (maybe_input, _semi) = tag(";")(maybe_input)?;
        let (maybe_input, _ln) = many1(line_ending)(maybe_input)?;

        Ok((maybe_input, call))
    }
}