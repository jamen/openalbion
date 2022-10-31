#![feature(slice_take)]

pub use bytemuck;
use bytemuck::bytes_of;

use crate::bytemuck::{Pod, PodCastError};

use core::mem::size_of;

pub trait ReadPod {
    fn read_pod<T: Pod>(&mut self) -> Result<&T, PodCastError>;
}

impl ReadPod for &[u8] {
    fn read_pod<T: Pod>(&mut self) -> Result<&T, PodCastError> {
        if self.len() < size_of::<T>() {
            return Err(PodCastError::SizeMismatch);
        }
        let (a, b) = self.split_at(size_of::<T>());
        let res = bytemuck::try_from_bytes(a);
        if res.is_ok() {
            *self = b;
        }
        res
    }
}

pub trait WritePod {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError>;
}

impl WritePod for &mut [u8] {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError> {
        let a = self
            .take_mut(..size_of::<T>())
            .ok_or(PodCastError::SizeMismatch)?;
        a.copy_from_slice(bytes_of(item));
        Ok(())
    }
}

impl<const N: usize> WritePod for &mut [u8; N] {
    fn write_pod<T: Pod>(&mut self, item: &T) -> Result<(), PodCastError> {
        (&mut self[..]).write_pod(item)
    }
}

// Working example
//
// #![feature(slice_take)]
// pub fn put<'a, 'b, T: Pod>(out: &'a mut &'b mut [u8], item: &T) -> Result<(), PodCastError> {
//     let a = out
//         .take_mut(..size_of::<T>())
//         .ok_or(PodCastError::SizeMismatch)?;
//     a.copy_from_slice(bytes_of(item));
//     Ok(())
// }
