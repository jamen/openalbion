use std::io::{Read,Write,Seek};

use super::Error;

pub trait Decode: Sized {
    fn decode(source: &mut (impl Read + Seek)) -> Result<Self, Error>;
}

pub trait Encode {
    fn encode(&self, sink: &mut (impl Write + Seek)) -> Result<(), Error>;
}