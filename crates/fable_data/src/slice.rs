pub trait TakeSliceExt<T> {
    fn grab_first(&mut self) -> Option<&T>;
    // fn grab(&mut self, n: usize) -> Option<&[T]>;
}

impl<T> TakeSliceExt<T> for &[T] {
    fn grab_first(&mut self) -> Option<&T> {
        let (first, rest) = match self.split_first() {
            Some(res) => res,
            None => return None,
        };
        *self = rest;
        Some(first)
    }

    // fn grab(&mut self, n: usize) -> Option<&[T]> {
    //     let (first, rest) = self.split_at(n);
    //     *self = rest;
    //     Some(first)
    // }
}
