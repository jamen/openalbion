use std::io::{Read,Seek};

use nom::IResult;

use crate::{Decode,Error};
use super::{
    Def,
    DefItem,
    Definition,
};

// impl<T: Read + Seek> Decode<Def> for T {
//     fn decode(&mut self) -> Result<Def, Error> {
//     }
// }

// impl Def {
//     fn decode_def(input: &[u8]) -> IResult<&[u8], Def, Error> {
//     }

//     fn decode_def_item(input: &[u8]) -> IResult<&[u8], DefItem, Error> {
//     }

//     fn decode_definition(input: &[u8]) -> IResult<&[u8], Definition, Error> {
//     }
// }