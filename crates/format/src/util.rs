use bytemuck::{bytes_of, AnyBitPattern, NoUninit};
use core::mem::size_of;

pub(crate) fn take<T: AnyBitPattern>(source: &mut &[u8]) -> Option<T> {
    bytemuck::try_pod_read_unaligned(source.take(..size_of::<T>())?).ok()
}

pub(crate) fn put<T: NoUninit>(output: &mut &mut [u8], value: &T) -> Option<()> {
    output
        .take_mut(..size_of::<T>())?
        .copy_from_slice(bytes_of(value));
    Some(())
}

pub(crate) fn null_terminated_buf<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = i.iter().position(|c| *c == 0u8)?;
    let value = i.take(..len)?;
    let _null = i.take(..1)?;
    Some(value)
}

pub(crate) fn run_le_u32_buf<'a>(i: &mut &'a [u8]) -> Option<&'a [u8]> {
    let len = take::<u32>(i)?.to_le();
    let len = usize::try_from(len).ok()?;
    i.take(..len)
}
