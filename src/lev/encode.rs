use std::io::{Write,Seek};

use crate::{
    Error,
    Encode,
};

use super::{
    Lev,
};

// impl Encode for Lev {
//     fn encode<Sink>(sink: &Sink) -> Result<Self, Error> where
//         Sink: Write + Seek
//     {
//     }
// }