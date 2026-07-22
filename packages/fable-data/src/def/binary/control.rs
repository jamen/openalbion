//! Control-level types shared by every def body.
//!
//! A def body is a stream of *controls*: `u32 crc32(field name)` followed by
//! the field's value. The value model lives in [`wire`](crate::wire);
//! this module owns the control-level error types that attach the field name
//! to whatever went wrong.

use crate::bytes::{TakeError, UnexpectedEnd};
use crate::def::wire::ParseWireError;

/// Size of the `u32 crc32(field name)` id that precedes every field value.
pub const ID_BYTE_SIZE: usize = size_of::<u32>();

#[derive(Debug)]
pub enum ParseControlErrorReason {
    MalformedId(TakeError),
    /// The control id read didn't match `crc(name)` — i.e. the cursor isn't
    /// aligned to this field's control (wrong order, missing/extra bytes, …).
    WrongId {
        expected: u32,
        found: u32,
    },
    /// The id matched but the value failed to parse.
    Wire(ParseWireError),
}

#[derive(Debug)]
pub struct ParseControlError {
    pub name: &'static str,
    pub reason: ParseControlErrorReason,
}

#[derive(Debug)]
pub enum SerializeControlErrorReason {
    MalformedId(UnexpectedEnd),
    Value(UnexpectedEnd),
}

#[derive(Debug)]
pub struct SerializeControlError {
    pub name: &'static str,
    pub reason: SerializeControlErrorReason,
}
