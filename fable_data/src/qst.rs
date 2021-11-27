pub use crate::qst_parser::QstParser;

pub struct Qst {}

impl Qst {
    // pub fn decode<Source: Read + Seek>(source: &mut Source) -> Result<Self, Error> {
    // }
}

// use crate::script::ScriptCall;

// #[derive(Debug,PartialEq)]
// pub struct Qst {
//     pub body: Vec<ScriptCall>
// }

// use std::io::{Read,Seek};

// use crate::{
//     IResult,
//     all_consuming,
//     line_ending,
//     many1,
//     many0,
//     opt,
//     tag,
// };

// use crate::{
//     ScriptCall,
//     Decode,
//     Error,
//     Qst,
// };

// impl Decode for Qst {
//     fn decode<Source>(source: &mut Source) -> Result<Self, Error> where
//         Source: Read + Seek
//     {
//         let mut data = Vec::new();
//         source.read_to_end(&mut data)?;
//         let (_, qst) = all_consuming(Qst::decode_qst)(&data)?;
//         Ok(qst)
//     }
// }

// impl Qst {
//     pub fn decode_qst(input: &[u8]) -> IResult<&[u8], Qst, Error> {
//         let (input, body) = many1(Self::decode_call)(input)?;
//         Ok((input, Qst { body: body }))
//     }

//     pub fn decode_call(input: &[u8]) -> IResult<&[u8], ScriptCall, Error> {
//         let (input, _) = opt(many0(line_ending))(input)?;
//         let (input, call) = ScriptCall::decode_call(input)?;
//         let (input, _) = tag(";")(input)?;
//         let (input, _) = many1(line_ending)(input)?;
//         Ok((input, call))
//     }
// }
