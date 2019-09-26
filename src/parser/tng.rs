use nom::IResult;
use nom::multi::{many1,many_till};
use crate::parser::util::script::{parse_instr,parse_instr_tag,Instr};

#[derive(Debug,PartialEq)]
pub struct Tng {
    pub version: Instr,
    pub sections: Vec<TngSection>,
}

#[derive(Debug,PartialEq)]
pub struct TngSection {
    pub section_start: Instr,
    pub things: Vec<TngThing>,
}

#[derive(Debug,PartialEq)]
pub struct TngThing {
    pub new_thing: Instr,
    pub instrs: Vec<Instr>
    // TODO: Parse instrs more thoroughly into fields.
}

pub fn parse_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing> {
    let (input, new_thing) = parse_instr_tag("NewThing".to_string())(input)?;
    let (input, (instrs, _end_instr)) = many_till(parse_instr, parse_instr_tag("EndThing".to_string()))(input)?;
    Ok(
        (
            input,
            TngThing {
                new_thing: new_thing,
                instrs: instrs,
            }
        )
    )
}

pub fn parse_tng_section(input: &[u8]) -> IResult<&[u8], TngSection> {
    let (input, section_start) = parse_instr_tag("XXXSectionStart".to_string())(input)?;
    let (input, (things, _end_instr)) = many_till(parse_tng_thing, parse_instr_tag("XXXSectionEnd".to_string()))(input)?;
    Ok(
        (
            input,
            TngSection {
                section_start: section_start,
                things: things,
            }
        )
    )
}

pub fn parse_tng(input: &[u8]) -> IResult<&[u8], Tng> {
    let (input, version) = parse_instr_tag("Version".to_string())(input)?;
    let (input, sections) = many1(parse_tng_section)(input)?;
    Ok(
        (
            input,
            Tng {
                version: version,
                sections: sections,
            }
        )
    )
}