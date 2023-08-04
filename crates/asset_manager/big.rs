use std::io::BufRead;

pub struct Big<T: BufRead> {
    source: T,
}

impl<T: BufRead> Big<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }
}
