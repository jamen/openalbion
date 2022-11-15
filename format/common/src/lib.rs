#![feature(slice_take)]

use core::mem::size_of;

pub use bytemuck;

use crate::bytemuck::{bytes_of, Pod, PodCastError};

pub trait ReadPod {
    fn read_pod<T: Pod>(&mut self) -> Result<&T, PodCastError>;
}

impl ReadPod for &[u8] {
    fn read_pod<T: Pod>(&mut self) -> Result<&T, PodCastError> {
        bytemuck::try_from_bytes(
            self.take(..size_of::<T>())
                .ok_or(PodCastError::SizeMismatch)?,
        )
    }
}

pub trait WritePod {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError>;
}

impl WritePod for &mut [u8] {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError> {
        self.take_mut(..size_of::<T>())
            .ok_or(PodCastError::SizeMismatch)?
            .copy_from_slice(bytes_of(item));
        Ok(())
    }
}

impl<const N: usize> WritePod for &mut [u8; N] {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError> {
        (&mut self[..]).write_pod(item)
    }
}
