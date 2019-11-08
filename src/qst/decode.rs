use nom::IResult;
use nom::multi::{many1,many0};
use nom::character::complete::line_ending;
use nom::bytes::complete::tag;
use nom::combinator::opt;

use crate::script::InstrValue;
use crate::script::decode::decode_instr_value_call;

use crate::qst::Qst;

pub fn decode_qst(input: &[u8]) -> IResult<&[u8], Qst> {
    let (maybe_input, body) = many1(decode_call)(input)?;
    Ok((maybe_input, Qst { body: body }))
}

pub fn decode_call(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, _ln) = opt(many0(line_ending))(input)?;
    let (maybe_input, call) = decode_instr_value_call(maybe_input)?;
    let (maybe_input, _semi) = tag(";")(maybe_input)?;
    let (maybe_input, _ln) = many1(line_ending)(maybe_input)?;

    Ok((maybe_input, call))
}