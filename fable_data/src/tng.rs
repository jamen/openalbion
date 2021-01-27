// use crate::script::ScriptField;

pub struct Tng {}

// #[derive(Debug,PartialEq)]
// pub struct Tng {
//     pub version: u32,
//     pub sections: Vec<TngSection>,
// }

// #[derive(Debug,PartialEq)]
// pub struct TngSection {
//     pub xxx_section_start: Option<String>,
//     pub things: Vec<Thing>,
// }

// #[derive(Debug,PartialEq)]
// pub struct Thing {
//     pub new_thing: ScriptField,
//     pub fields: Vec<ScriptField>,
// }

// impl Tng {
//     pub fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
//         let mut input = Vec::new();
//         source.read_to_end(&mut input)?;
//         let (_, tng) = all_consuming(Tng::decode_tng)(&input)?;
//         Ok(tng)
//     }

//     pub fn decode_tng(input: &[u8]) -> IResult<&[u8], Tng, Error> {
//         let (input, version) = ScriptField::decode_field_named("Version")(input)?;
//         let (input, sections) = many0(Self::decode_tng_section)(input)?;

//         Ok(
//             (
//                 input,
//                 Tng {
//                     version: version,
//                     sections: sections,
//                 }
//             )
//         )
//     }

//     pub fn decode_tng_section(input: &[u8]) -> IResult<&[u8], TngSection, Error> {
//         let (input, section_start) = ScriptField::decode_field_named("XXXSectionStart")(input)?;
//         let (input, (things, _end)) = many_till(Self::decode_tng_thing, ScriptField::decode_field_named("XXXSectionEnd"))(input)?;

//         Ok(
//             (
//                 input,
//                 TngSection {
//                     section_start: section_start,
//                     things: things,
//                 }
//             )
//         )
//     }

//     pub fn decode_tng_thing(input: &[u8]) -> IResult<&[u8], TngThing, Error> {
//         let (input, new_thing) = ScriptField::decode_field_named("NewThing")(input)?;
//         let (input, (fields, _end)) = many_till(ScriptField::decode_field, ScriptField::decode_field_named("EndThing"))(input)?;

//         Ok(
//             (
//                 input,
//                 TngThing {
//                     new_thing: new_thing,
//                     fields: fields
//                 }
//             )
//         )
//     }
// }