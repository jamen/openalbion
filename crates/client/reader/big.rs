use big::BigHeader;
use std::io::Read;

pub struct BigReader<T: Read> {
    source: T,
    header: BigHeader,
}

impl<T: Read> BigReader<T> {
    pub fn new(mut source: T) -> Self {
        let header = {
            let mut bytes = [0u8; BigHeader::TOTAL_BYTE_SIZE];
            source.read_exact(&mut bytes[..]).unwrap();
            BigHeader::parse(&bytes).unwrap()
        };

        println!("{:?}", header);

        Self { source, header }
    }
}
