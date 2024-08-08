use bytemuck::{bytes_of, AnyBitPattern, NoUninit};
use core::mem;

/// Take bytes from the front of a byte slice.
///
/// If successful, a byte slice is returned and the original slice is advanced forward.
/// Upon failure, the part identifier and length of the slice are returned, and the slice remains as-is.
pub(crate) fn take_bytes<'a>(source: &mut &'a [u8], split_index: usize) -> Option<&'a [u8]> {
    if split_index > source.len() {
        return None;
    }
    let (front, back) = source.split_at(split_index);
    *source = back;
    Some(front)
}

/// Take a value from the front of a byte slice.
///
/// The value must satisfy `bytemuck::AnyBitPattern`, which includes any value satisfying `bytemuck::Pod`.
pub(crate) fn take<'a, T: AnyBitPattern>(source: &mut &'a [u8]) -> Option<T> {
    let split_index = mem::size_of::<T>();
    let front = take_bytes(source, split_index)?;
    bytemuck::try_pod_read_unaligned(front).ok()
}

/// Take a value

/// Take a null terminated buffer from the front of a a byte slice.
pub(crate) fn take_null_terminated_buf<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = i.iter().position(|c| *c == 0u8)?;
    let value = take_bytes(i, len)?;
    let _null = take_bytes(i, 1)?;
    Some(value)
}

/// Take a run-length buffer from a byte slice, with a little-endian u32 integer as the run-length.
pub(crate) fn take_rle_buf_with_le_u32<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = take::<u32>(i)?.to_le();
    let len = usize::try_from(len).ok()?;
    take_bytes(i, len)
}

/// Put bytes in the front of a byte slice.
///
/// The original byte slice is advanced forward.
pub(crate) fn put_bytes<'a>(source: &mut &'a mut [u8], bytes: &[u8]) -> Option<()> {
    let split_index = bytes.len();
    if split_index > source.len() {
        return None;
    }
    let (front, back) = mem::take(source).split_at_mut(split_index);
    *source = back;
    front.copy_from_slice(bytes);
    Some(())
}

/// Put a value in the front of a byte slice.
///
/// The value must satisfy `bytemuck::NoUninit`, which includes any value satisfying `bytemuck::Pod`.
pub(crate) fn put<'a, T: NoUninit>(source: &mut &'a mut [u8], value: &T) -> Option<()> {
    put_bytes(source, bytes_of(value))
}
