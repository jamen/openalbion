use bytemuck::{bytes_of, AnyBitPattern, NoUninit};
use core::mem;

pub(crate) fn take_bytes<'a>(source: &mut &'a [u8], split_index: usize) -> Option<&'a [u8]> {
    if split_index > source.len() {
        return None;
    }
    let (front, back) = source.split_at(split_index);
    *source = back;
    Some(front)
}

pub(crate) fn take_as<'a, T: AnyBitPattern>(source: &mut &'a [u8]) -> Option<T> {
    let split_index = mem::size_of::<T>();
    let front = take_bytes(source, split_index)?;
    bytemuck::try_pod_read_unaligned(front).ok()
}

pub(crate) fn take_null_terminated_buf<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = i.iter().position(|c| *c == 0u8)?;
    let value = take_bytes(i, len)?;
    let _null = take_bytes(i, 1)?;
    Some(value)
}

pub(crate) fn take_le_u32_run_buf<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = take_as::<u32>(i)?.to_le();
    let len = usize::try_from(len).ok()?;
    take_bytes(i, len)
}

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

pub(crate) fn put<'a, T: NoUninit>(source: &mut &'a mut [u8], value: &T) -> Option<()> {
    put_bytes(source, bytes_of(value))
}
