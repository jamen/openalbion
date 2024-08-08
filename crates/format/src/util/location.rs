pub(crate) struct Loc<Part> {
    progressed_slice_len: usize,
    part: Part,
}

impl<Part> Loc<Part> {
    pub fn new(i: &[u8], part: Part) -> Self {
        Self {
            progressed_slice_len: i.len(),
            part,
        }
    }
}
