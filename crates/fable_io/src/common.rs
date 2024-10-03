use derive_more::{Display, From};
use fable_format::UnexpectedEnd;

#[derive(Copy, Clone, Debug, PartialEq, Eq, From, Display)]
#[display("{error} at position {position}")]
pub struct OffsetError<E> {
    pub position: usize,
    pub error: E,
}

impl<E> OffsetError<E> {
    pub fn new(original: &[u8], partially_parsed: &[u8], error: E) -> Result<Self, UnexpectedEnd> {
        let position = original
            .len()
            .checked_sub(partially_parsed.len())
            .ok_or_else(|| UnexpectedEnd)?;

        Ok(Self { position, error })
    }
}
