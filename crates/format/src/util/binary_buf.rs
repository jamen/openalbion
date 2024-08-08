use bytemuck::{AnyBitPattern, NoUninit, PodCastError};
use std::mem;

pub struct BinaryParser<'a> {
    original_len: usize,
    bytes: &'a [u8],
}

impl<'a> BinaryParser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            original_len: bytes.len(),
            bytes,
        }
    }

    pub(crate) fn pos(&self) -> usize {
        self.original_len - self.bytes.len()
    }

    pub(crate) fn new_error<SectionId>(
        &self,
        section_id: SectionId,
        cast_error: Option<PodCastError>,
    ) -> BinaryParserError<SectionId> {
        BinaryParserError::new(self.pos(), section_id, cast_error)
    }

    /// Take bytes from the front of a byte slice.
    ///
    /// If successful, a byte slice is returned and the original slice is advanced forward.
    /// Upon failure, the part identifier and length of the slice are returned, and the slice remains as-is.
    pub(crate) fn take_bytes<SectionId: Copy>(
        &mut self,
        split_index: usize,
        section_id: SectionId,
    ) -> Result<&'a [u8], BinaryParserError<SectionId>> {
        if split_index > self.bytes.len() {
            return Err(self.new_error(section_id, None));
        }
        let (front, back) = self.bytes.split_at(split_index);
        self.bytes = back;
        Ok(front)
    }

    /// Take a value from the front of a byte slice.
    ///
    /// The value must satisfy `bytemuck::AnyBitPattern`, which includes any value satisfying `bytemuck::Pod`.
    pub(crate) fn take<T: AnyBitPattern, SectionId: Copy>(
        &mut self,
        section_id: SectionId,
    ) -> Result<T, BinaryParserError<SectionId>> {
        let split_index = mem::size_of::<T>();

        let front = self.take_bytes(split_index, section_id)?;

        bytemuck::try_pod_read_unaligned(front)
            .map_err(|cast_error| self.new_error(section_id, Some(cast_error)))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BinaryParserError<SectionId> {
    pub pos: usize,
    pub section_id: SectionId,
    pub cast_error: Option<bytemuck::PodCastError>,
}

impl<SectionId> BinaryParserError<SectionId> {
    pub fn new(
        pos: usize,
        section_id: SectionId,
        cast_error: Option<bytemuck::PodCastError>,
    ) -> Self {
        Self {
            pos,
            section_id,
            cast_error,
        }
    }
}

pub struct BinarySerializer<'a> {
    original_len: usize,
    bytes: &'a mut [u8],
}

impl<'a> BinarySerializer<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self {
            original_len: bytes.len(),
            bytes,
        }
    }

    pub(crate) fn pos(&self) -> usize {
        self.original_len - self.bytes.len()
    }

    pub(crate) fn new_error<SectionId>(
        &self,
        section_id: SectionId,
    ) -> BinarySerializerError<SectionId> {
        BinarySerializerError::new(self.pos(), section_id)
    }

    /// Put bytes in the front of a byte slice.
    ///
    /// The original byte slice is advanced forward.
    pub(crate) fn put_bytes<SectionId>(
        &mut self,
        bytes: &[u8],
        section_id: SectionId,
    ) -> Result<(), BinarySerializerError<SectionId>> {
        let split_index = bytes.len();
        if split_index > self.bytes.len() {
            return Err(self.new_error(section_id));
        }
        let (front, back) = mem::take(&mut self.bytes).split_at_mut(split_index);
        self.bytes = back;
        front.copy_from_slice(bytes);
        Ok(())
    }

    /// Put a value in the front of a byte slice.
    ///
    /// The value must satisfy `bytemuck::NoUninit`, which includes any value satisfying `bytemuck::Pod`.
    pub(crate) fn put<T: NoUninit, SectionId>(
        &mut self,
        value: &T,
        section_id: SectionId,
    ) -> Result<(), BinarySerializerError<SectionId>> {
        self.put_bytes(bytemuck::bytes_of(value), section_id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BinarySerializerError<SectionId> {
    pub pos: usize,
    pub section_id: SectionId,
}

impl<SectionId> BinarySerializerError<SectionId> {
    pub fn new(pos: usize, section_id: SectionId) -> Self {
        Self { pos, section_id }
    }
}
