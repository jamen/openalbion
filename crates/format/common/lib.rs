#![feature(slice_take)]

use core::mem::size_of;

pub use bytemuck;

use crate::bytemuck::{bytes_of, Pod, PodCastError};

pub fn take<'a, T: Pod>(src: &'_ mut &'a [u8]) -> Result<&'a T, PodCastError> {
    let Some(data) = src.take(..size_of::<T>()) else {
        return Err(PodCastError::SizeMismatch)
    };
    bytemuck::try_from_bytes(data)
}

pub fn put<'a, T: Pod, E>(mut out: &'a mut [u8], item: &T, err: E) -> Result<(), E> {
    out.take_mut(..size_of::<T>())
        .ok_or_else(|| err)?
        .copy_from_slice(bytes_of(item));
    Ok(())
}
