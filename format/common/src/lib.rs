#![feature(slice_take)]

use core::mem::size_of;

pub use bytemuck;

use crate::bytemuck::{bytes_of, Pod};

pub fn take<T: Pod, E>(mut src: &[u8], err: E) -> Result<&T, (E, &[u8])> {
    let Some(data) = src.take(..size_of::<T>()) else {
        return Err((err, src))
    };
    bytemuck::try_from_bytes(data).map_err(|_| (err, src))
}

pub fn put<'a, T: Pod, E>(mut out: &'a mut [u8], item: &T, err: E) -> Result<(), E> {
    out.take_mut(..size_of::<T>())
        .ok_or_else(|| err)?
        .copy_from_slice(bytes_of(item));
    Ok(())
}
