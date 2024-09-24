use bytemuck::{AnyBitPattern, NoUninit, PodCastError};
use derive_more::{Display, From};
use std::mem;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display)]
#[display("Unexpected end of input")]
struct UnexpectedEnd;

/// Take bytes from the front of a slice.
pub fn take_bytes<'a>(bytes: &mut &'a [u8], size: usize) -> Result<&'a [u8], UnexpectedEnd> {
    if size > bytes.len() {
        Err(UnexpectedEnd)?
    }
    let (front, back) = bytes.split_at(size);
    *bytes = back;
    Ok(front)
}

/// Take bytes up until a NUL byte.
pub fn take_bytes_nul_terminated<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], UnexpectedEnd> {
    let size = bytes
        .iter()
        .position(|&x| x == 0)
        .ok_or_else(|| UnexpectedEnd)?;
    take_bytes(bytes, size)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, From, Display)]
enum TakeError {
    UnexpectedEnd(UnexpectedEnd),
    PodCast(PodCastError),
}

/// Take a value that implements `bytemuck::AnyBitPattern` from the front of a byte slice.
pub fn take<T: AnyBitPattern>(bytes: &mut &[u8]) -> Result<T, TakeError> {
    let size = mem::size_of::<T>();
    let front = take_bytes(bytes, size)?;
    Ok(bytemuck::try_pod_read_unaligned(front)?)
}

/// Put bytes in front of a byte slice and advance forward.
pub fn put_bytes(out: &mut &mut [u8], inp: &[u8]) -> Result<(), UnexpectedEnd> {
    let split_index = inp.len();
    if split_index > out.len() {
        Err(UnexpectedEnd)?
    }
    let (front, back) = mem::take(out).split_at_mut(split_index);
    *out = back;
    front.copy_from_slice(inp);
    Ok(())
}

/// Put a value implementing `bytemuck::NoUninit` in front of a byte slice and advance forward.
pub fn put<T: NoUninit>(out: &mut &mut [u8], value: &T) -> Result<(), UnexpectedEnd> {
    put_bytes(out, bytemuck::bytes_of(value))
}
