use nom::IResult;
use nom::multi::{many0,many_till};

use crate::script::decode::{decode_instr,decode_instr_tag};

use crate::tng::{
    TngThing,
    TngSection,
    Tng,
};

pub fn decode_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing> {
    let (maybe_input, new_thing) = decode_instr_tag("NewThing".to_string())(input)?;
    let (maybe_input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag("EndThing".to_string()))(maybe_input)?;

    Ok(
        (
            maybe_input,
            TngThing {
                new_thing: new_thing,
                instrs: instrs,
            }
        )
    )
}

pub fn decode_tng_section(input: &[u8]) -> IResult<&[u8], TngSection> {
    let (maybe_input, section_start) = decode_instr_tag("XXXSectionStart".to_string())(input)?;
    let (maybe_input, (things, _end)) = many_till(decode_tng_thing, decode_instr_tag("XXXSectionEnd".to_string()))(maybe_input)?;

    Ok(
        (
            maybe_input,
            TngSection {
                section_start: section_start,
                things: things,
            }
        )
    )
}

pub fn decode_tng(input: &[u8]) -> IResult<&[u8], Tng> {
    let (maybe_input, version) = decode_instr_tag("Version".to_string())(input)?;
    let (maybe_input, sections) = many0(decode_tng_section)(maybe_input)?;

    Ok(
        (
            maybe_input,
            Tng {
                version: version,
                sections: sections,
            }
        )
    )
}