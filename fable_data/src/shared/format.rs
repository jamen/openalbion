use std::io::{self,Read,Write,Seek,SeekFrom,Take};

use crate::Error;

/// Trait that decoders implement.
pub trait Decode: Sized {
    fn decode<Source>(input: &mut Source) -> Result<Self, Error> where
        Source: Read + Seek;
}

/// Trait that encoders implement.
pub trait Encode {
    fn encode<Sink>(&self, output: &mut Sink) -> Result<(), Error> where
        Sink: Write + Seek;
}

/// An entry to another section of the data.
pub trait Entry: Sized {
    fn len(&self) -> u64;
    fn pos(&self) -> u64;
    /// Creates a reader for the entry for further decoding.
    ///
    /// Using a `std::io::BufReader` source is recommended. Especially when reading multiple and/or small entries.
    fn reader<'a, Source: Read + Seek>(&mut self, source: &'a mut Source) -> Result<Take<&'a mut Source>, io::Error> {
        source.seek(SeekFrom::Start(self.pos()))?;
        Ok(source.take(self.len()))
    }
}