use bytemuck::{bytes_of, Pod};
use core::mem::size_of;

pub(crate) fn take<'a, T: Pod>(src: &'_ mut &'a [u8]) -> Option<&'a T> {
    bytemuck::try_from_bytes(src.take(..size_of::<T>())?).ok()
}

pub(crate) fn put<'a, T: Pod>(out: &mut &'a mut [u8], item: &T) -> Option<()> {
    out.take_mut(..size_of::<T>())?
        .copy_from_slice(bytes_of(item));
    Some(())
}

pub(crate) fn take_null_terminated<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = i.iter().position(|c| *c == 0u8)?;
    let value = i.take(..len)?;
    let _null = i.take(..1)?;
    Some(value)
}

pub(crate) fn take_run_length_le_u32<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = take::<u32>(i)?.to_le();
    let len = usize::try_from(len).ok()?;
    i.take(..len)
}
