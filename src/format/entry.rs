use std::io::{self,Read,Write,Seek,SeekFrom};

pub struct Entry<S> {
    start: u64,
    end: u64,
    source: S,
}

impl<S: Seek> Entry<S> {
    pub fn new(mut source: S, start: u64, end: u64) -> Result<Self, io::Error> {
        source.seek(SeekFrom::Start(start))?;
        Ok(Self { start, end, source })
    }
    pub fn into_inner(self) -> S {
        self.source
    }
}

impl<S: Seek> Seek for Entry<S> {
    fn seek(&mut self, seek_from: SeekFrom) -> io::Result<u64> {
        let seek_from = match seek_from {
            SeekFrom::Start(x) => {
                let position = self.start + x;

                if position < self.end {
                    Some(SeekFrom::Start(position))
                } else {
                    None
                }
            },
            SeekFrom::End(x) => {
                let position = if x >= 0 {
                    self.end + x as u64
                } else {
                    self.end - (x * -1) as u64
                };

                if position >= self.start && position < self.end {
                    Some(SeekFrom::End(x + self.end as i64 * -1))
                } else {
                    None
                }
            },
            SeekFrom::Current(x) => {
                let current_position = self.source.seek(SeekFrom::Current(0))?;

                let position = if x >= 0 {
                    current_position.checked_add(x as u64)
                } else {
                    current_position.checked_sub((x * -1) as u64)
                };

                if let Some(position) = position {
                    if position >= self.start && position < self.end {
                        Some(SeekFrom::Current(x))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        };

        match seek_from {
            Some(seek_from) => self.source.seek(seek_from),
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek to a negative or overflowing position.")),
        }
    }
}

impl<S: Seek + Read> Read for Entry<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let end = buf.len().min((self.end - self.source.seek(SeekFrom::Current(0))?) as usize);
        self.source.read(&mut buf[..end])
    }
}

impl<S: Seek + Write> Write for Entry<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let end = buf.len().min((self.end - self.source.seek(SeekFrom::Current(0))?) as usize);
        self.source.write(&buf[..end])
    }
    fn flush(&mut self) -> io::Result<()> {
        self.source.flush()
    }
}