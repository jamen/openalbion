use bytemuck::{AnyBitPattern, NoUninit};

use crate::{
    bytes::{
        Le, TakeError, TakeNullTerminatedUtf8, TakeNullTerminatedUtf16, UnexpectedEnd, put, put_le,
        put_null_terminated_utf8, put_null_terminated_utf16, take, take_le,
        take_null_terminated_utf8, take_null_terminated_utf16,
    },
    crc32::crc,
};

// Control size helpers

pub const ID_BYTE_SIZE: usize = size_of::<u32>();

pub const fn string_control_byte_size(string: &str) -> usize {
    ID_BYTE_SIZE + string.len() + size_of::<u8>()
}

pub fn wstr_control_byte_size(string: &str) -> usize {
    ID_BYTE_SIZE + string.encode_utf16().count() * size_of::<u16>() + size_of::<u16>()
}

pub const fn list_control_byte_size<T>(list: &[T]) -> usize {
    ID_BYTE_SIZE + size_of::<u32>() + std::mem::size_of_val(list)
}

// Control ids

#[derive(Debug)]
pub enum ParseControlErrorReason {
    MalformedId(TakeError),
    /// The control id read didn't match `crc(name)` — i.e. the cursor isn't
    /// aligned to this field's control (wrong order, missing/extra bytes, …).
    WrongId { expected: u32, found: u32 },
    Value(TakeError),
    Utf8(TakeNullTerminatedUtf8),
    Utf16(TakeNullTerminatedUtf16),
    ListCount(TakeError),
    ListItem(usize, TakeError),
}

#[derive(Debug)]
pub struct ParseControlError {
    pub name: &'static str,
    pub reason: ParseControlErrorReason,
}

pub fn parse_id(cur: &mut &[u8], name: &'static str) -> Result<u32, ParseControlError> {
    let id = take_le::<u32>(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::MalformedId(inner),
    })?;

    let expected = crc(name.as_bytes());
    if id != expected {
        return Err(ParseControlError {
            name,
            reason: ParseControlErrorReason::WrongId { expected, found: id },
        });
    }

    Ok(id)
}

pub fn parse_scalar<T: AnyBitPattern + Le>(
    cur: &mut &[u8],
    name: &'static str,
) -> Result<T, ParseControlError> {
    let _id = parse_id(cur, name)?;

    let value = take_le::<T>(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::Value(inner),
    })?;

    Ok(value)
}

pub fn parse_bool(cur: &mut &[u8], name: &'static str) -> Result<bool, ParseControlError> {
    let _id = parse_id(cur, name)?;

    let value = take::<u8>(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::Value(inner),
    })?;

    let value = value != 0x0;

    Ok(value)
}

pub fn parse_string<'a>(
    cur: &mut &'a [u8],
    name: &'static str,
) -> Result<&'a str, ParseControlError> {
    let _id = parse_id(cur, name)?;

    let value = take_null_terminated_utf8(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::Utf8(inner),
    })?;

    Ok(value)
}

pub fn parse_wstr(cur: &mut &[u8], name: &'static str) -> Result<String, ParseControlError> {
    let _id = parse_id(cur, name)?;

    let value = take_null_terminated_utf16(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::Utf16(inner),
    })?;

    Ok(value)
}

pub fn parse_list<T: AnyBitPattern + Le>(
    cur: &mut &[u8],
    name: &'static str,
) -> Result<Vec<T>, ParseControlError> {
    let _id = parse_id(cur, name)?;

    let count = take_le::<u32>(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::ListCount(inner),
    })?;

    let list = (0..count)
        .map(|i| {
            take_le::<T>(cur).map_err(|inner| ParseControlError {
                name,
                reason: ParseControlErrorReason::ListItem(i as usize, inner),
            })
        })
        .collect::<Result<Vec<T>, _>>()?;

    Ok(list)
}

// Control serializers

#[derive(Debug)]
pub enum SerializeControlErrorReason {
    MalformedId(UnexpectedEnd),
    Value(UnexpectedEnd),
    Utf8(UnexpectedEnd),
    Utf16(UnexpectedEnd),
    ListCount(UnexpectedEnd),
    ListItem(usize, UnexpectedEnd),
}

#[derive(Debug)]
pub struct SerializeControlError {
    pub name: &'static str,
    pub reason: SerializeControlErrorReason,
}

pub fn serialize_id(out: &mut &mut [u8], name: &'static str) -> Result<(), SerializeControlError> {
    put_le(out, &crc(name.as_bytes())).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::MalformedId(inner),
    })
}

pub fn serialize_scalar<T: NoUninit + Le>(
    out: &mut &mut [u8],
    name: &'static str,
    value: T,
) -> Result<(), SerializeControlError> {
    serialize_id(out, name)?;

    put_le(out, &value).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::Value(inner),
    })?;

    Ok(())
}

pub fn serialize_bool(
    out: &mut &mut [u8],
    name: &'static str,
    value: bool,
) -> Result<(), SerializeControlError> {
    serialize_id(out, name)?;

    put(out, &(value as u8)).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::Value(inner),
    })?;

    Ok(())
}

pub fn serialize_string(
    out: &mut &mut [u8],
    name: &'static str,
    value: &str,
) -> Result<(), SerializeControlError> {
    serialize_id(out, name)?;

    put_null_terminated_utf8(out, value).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::Utf8(inner),
    })?;

    Ok(())
}

pub fn serialize_wstr(
    out: &mut &mut [u8],
    name: &'static str,
    value: &str,
) -> Result<(), SerializeControlError> {
    serialize_id(out, name)?;

    put_null_terminated_utf16(out, value).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::Utf16(inner),
    })?;

    Ok(())
}

pub fn serialize_list<T: NoUninit + Le>(
    out: &mut &mut [u8],
    name: &'static str,
    list: &[T],
) -> Result<(), SerializeControlError> {
    serialize_id(out, name)?;

    put_le(out, &(list.len() as u32)).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::ListCount(inner),
    })?;

    for (i, &item) in list.iter().enumerate() {
        put_le(out, &item).map_err(|inner| SerializeControlError {
            name,
            reason: SerializeControlErrorReason::ListItem(i, inner),
        })?
    }

    Ok(())
}
