use std::io::{Read,Seek};

use crate::nom::{
    IResult,
    many0,
    many_till,
    all_consuming,
};

use crate::{
    Decode,
    Error,
    ScriptField,
};

use super::{TngThing, TngSection, Tng};

impl Decode for Tng {
    fn decode<Source>(source: &mut Source) -> Result<Self, Error> where
        Source: Read + Seek
    {
        let mut input = Vec::new();
        source.read_to_end(&mut input)?;
        let (_, tng) = all_consuming(Tng::decode_tng)(&input)?;
        Ok(tng)
    }
}

impl Tng {
    pub fn decode_tng(input: &[u8]) -> IResult<&[u8], Tng, Error> {
        let (input, version) = ScriptField::decode_field_named("Version")(input)?;
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
        let (input, section_start) = ScriptField::decode_field_named("XXXSectionStart")(input)?;
        let (input, (things, _end)) = many_till(Self::decode_tng_thing, ScriptField::decode_field_named("XXXSectionEnd"))(input)?;

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
        let (input, new_thing) = ScriptField::decode_field_named("NewThing")(input)?;
        let (input, (fields, _end)) = many_till(ScriptField::decode_field, ScriptField::decode_field_named("EndThing"))(input)?;

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