use std::io::{Read,Seek};

use nom::IResult;
use nom::multi::{many0,many_till};
use nom::combinator::all_consuming;

use crate::{Decode,Error};
use crate::script::{decode_field,decode_field_named};

use super::{TngThing, TngSection, Tng};

impl<T: Read + Seek> Decode<Tng> for T {
    fn decode(&mut self) -> Result<Tng, Error> {
        let mut input = Vec::new();
        self.read_to_end(&mut input)?;
        let (_, tng) = all_consuming(Tng::decode_tng)(&input)?;
        Ok(tng)
    }
}

impl Tng {
    pub fn decode_tng(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (input, version) = decode_field_named("Version")(input)?;
        let (input, sections) = many0(Self::decode_tng_section)(input)?;

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

    pub fn decode_tng_section(input: &[u8]) -> IResult<&[u8], TngSection, Error> {
        let (input, section_start) = decode_field_named("XXXSectionStart")(input)?;
        let (input, (things, _end)) = many_till(Self::decode_tng_thing, decode_field_named("XXXSectionEnd"))(input)?;

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

    pub fn decode_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing, Error> {
        let (input, new_thing) = decode_field_named("NewThing")(input)?;
        let (input, (fields, _end)) = many_till(decode_field, decode_field_named("EndThing"))(input)?;

        Ok(
            (
                input,
                TngThing {
                    new_thing: new_thing,
                    fields: fields
                }
            )
        )
    }
}