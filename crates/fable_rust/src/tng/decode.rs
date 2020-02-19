use std::io::Read;

use nom::IResult;
use nom::multi::{many0,many_till};
use nom::combinator::all_consuming;

use crate::shared::{Decode,Error};
use crate::shared::script::decode::{decode_instr,decode_instr_tag};

use super::{TngThing, TngSection, Tng};

static VERSION: &'static str = "Version";
static XXXSECTIONSTART: &'static str = "XXXSectionStart";
static XXXSECTIONEND: &'static str = "XXXSectionEnd";
static NEWTHING: &'static str = "NewThing";
static ENDTHING: &'static str = "EndThing";

impl Decode for Tng {
    fn decode(source: &mut impl Read) -> Result<Self, Error> {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, tng) = all_consuming(Self::decode_tng)(&input)?;
        Ok(tng)
    }
}

impl Tng {
    pub fn decode_tng(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (maybe_input, version) = decode_instr_tag(VERSION)(input)?;
        let (maybe_input, sections) = many0(Self::decode_tng_section)(maybe_input)?;

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

    pub fn decode_tng_section(input: &[u8]) -> IResult<&[u8], TngSection, Error> {
        let (maybe_input, section_start) = decode_instr_tag(XXXSECTIONSTART)(input)?;
        let (maybe_input, (things, _end)) = many_till(Self::decode_tng_thing, decode_instr_tag(XXXSECTIONEND))(maybe_input)?;

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

    pub fn decode_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing, Error> {
        let (maybe_input, new_thing) = decode_instr_tag(NEWTHING)(input)?;
        let (maybe_input, (instrs, _end)) = many_till(decode_instr, decode_instr_tag(ENDTHING))(maybe_input)?;

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
}