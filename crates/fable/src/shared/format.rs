use std::io::{Read,Write};

use super::Error;

pub trait Decode: Sized {
    fn decode(source: &mut impl Read) -> Result<Self, Error>;
}

pub trait Encode {
    fn encode(&self, sink: &mut impl Write) -> Result<(), Error>;
}