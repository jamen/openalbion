use nom::IResult;
use nom::multi::many1;
use nom::character::complete::line_ending;
use nom::bytes::complete::tag;
use crate::parser::util::script::{parse_instr_value_call,InstrValue};

pub struct Qst {
    pub body: Vec<InstrValue>
}

pub fn parse_qst(input: &[u8]) -> IResult<&[u8], Qst> {
    let (maybe_input, body) = many1(parse_call)(input)?;
    Ok((maybe_input, Qst { body: body }))
}

pub fn parse_call(input: &[u8]) -> IResult<&[u8], InstrValue> {
    let (maybe_input, call) = parse_instr_value_call(input)?;
    let (maybe_input, _semi) = tag(";")(maybe_input)?;
    let (maybe_input, _ln) = line_ending(maybe_input)?;
    Ok((maybe_input, call))
}