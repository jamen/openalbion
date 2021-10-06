pub use crate::gtg_parser::GtgParser;

pub struct Gtg {}

impl Gtg {
    // pub fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
    //     todo!()
    // }
}

// use crate::Tng;

// #[derive(Debug,PartialEq)]
// pub struct Gtg {
//     maps: Vec<Tng>
// }

// use std::io::{Read,Seek};

// use crate::{
//     IResult,
//     all_consuming,
//     line_ending,
//     many0,
//     many1,
//     tag,
//     opt,
// };

// use crate::{
//     Error,
//     ErrorKind,
//     Tng,
// };

// impl Gtg {
//     pub fn decode<Source>(source: &mut Source) -> Result<Gtg, Error> where
//         Source: Read + Seek
//     {
//         let mut input = Vec::new();
//         source.read_to_end(&mut input)?;
//         let (_, gtg) = all_consuming(Gtg::decode_gtg)(&input)?;
//         Ok(gtg)
//     }

//     pub fn decode_gtg(input: &[u8]) -> IResult<&[u8], Gtg, Error> {
//         let (input, maps) = many1(Self::decode_gtg_map)(input)?;
//         Ok((input, Gtg { maps: maps }))
//     }

//     pub fn decode_gtg_map(input: &[u8]) -> IResult<&[u8], Tng, Error> {
//         let (input, _start) = Self::decode_gtg_field_named("NEWMAP")(input)?;
//         let (input, tng) = Tng::decode_tng(input)?;
//         let (input, _end) = Self::decode_gtg_field_named("ENDMAP")(input)?;
//         Ok((input, tng))
//     }

//     /// This is a variation of `fable::script::decode_tagged_field` because "NEWMAP" and "ENDMAP"
// don't use semicolons.     pub fn decode_gtg_field_named(name: &'static str) -> impl Fn(&[u8]) ->
// IResult<&[u8], ScriptField, Error> {         move |input: &[u8]| {
//             let (input, _line_ending) = many0(line_ending)(input)?;
//             let (input, reference) = ScriptReference::decode_reference(input)?;
//             let (input, _space) = opt(tag(" "))(input)?;
//             let (input, expression) = ScriptExpression::decode_expression(input)?;
//             let (input, _line_ending) = many1(line_ending)(input)?;

//            let field_name = match reference {
//                 ScriptReference::Name(x) => x,
//                 ScriptReference::ScriptAccessor(_) => return
// Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidInstruction))),             };

//             if field_name != name {
//                 return Err(nom::Err::Error(Error::Fable(ErrorKind::InvalidTagName)));
//             }

//             Ok(
//                 (
//                     input,
//                     ScriptField {
//                         reference: ScriptReference::Name(field_name),
//                         value: Box::new(expression),
//                     }
//                 )
//             )
//         }
//     }
// }
