//! Generic field walking.
//!
//! [`def_struct!`](crate::def_struct) emits a `visit_fields` method that hands
//! each field to a [`FieldVisitor`] as a typed [`FieldRef`]. This lets a
//! consumer in another crate (the def compiler) drive uniform per-field logic —
//! e.g. applying text-def overrides — without fable-data depending on it, and
//! without orphan-rule trouble: fable-data produces the `FieldRef`, the
//! consumer only reads it.

use crate::def::wire::WStr;

/// A closed def enum viewed as its wire `i32`, for generic field access.
pub trait EnumSlot {
    fn get_i32(&self) -> i32;
    /// Set from a wire/text value; `Err` carries an out-of-table value.
    fn set_i32(&mut self, value: i32) -> Result<(), i32>;
}

/// A def flags value viewed as its wire `i32` (total — any value is valid).
pub trait FlagsSlot {
    fn get_i32(&self) -> i32;
    fn set_i32(&mut self, value: i32);
}

/// A typed, mutable handle to one def field.
pub enum FieldRef<'a> {
    F32(&'a mut f32),
    I32(&'a mut i32),
    U32(&'a mut u32),
    Bool(&'a mut bool),
    /// `CCharString` (UTF-8).
    Str(&'a mut String),
    /// `CWideString` (UTF-16).
    WStr(&'a mut WStr),
    Enum(&'a mut dyn EnumSlot),
    Flags(&'a mut dyn FlagsSlot),
    /// A field the generic walk doesn't cover (lists, maps, sub-defs). Carries
    /// the field's Rust type name for diagnostics; the consumer handles these
    /// explicitly. `mut` access isn't offered here.
    Complex(&'static str),
}

/// Expose `&mut self` as a [`FieldRef`]. Implemented for the scalar/string
/// wire types here, for enum/flags types by the enum macros, and for the
/// container types (as [`FieldRef::Complex`]) below.
pub trait AsField {
    fn as_field(&mut self) -> FieldRef<'_>;
}

/// Receives each field of a def during [`visit_fields`](crate::def_struct).
pub trait FieldVisitor {
    fn field(&mut self, name: &'static str, field: FieldRef<'_>);
}

impl AsField for f32 {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::F32(self)
    }
}
impl AsField for i32 {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::I32(self)
    }
}
impl AsField for u32 {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::U32(self)
    }
}
impl AsField for bool {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Bool(self)
    }
}
impl AsField for String {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Str(self)
    }
}
impl AsField for WStr {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::WStr(self)
    }
}

// Wire types the generic walk doesn't cover yet; the consumer handles these
// explicitly (they only occur in game.bin defs, which aren't text-lowered).
macro_rules! as_complex {
    ($($ty:ty => $label:literal),+ $(,)?) => {
        $(
            impl AsField for $ty {
                fn as_field(&mut self) -> FieldRef<'_> {
                    FieldRef::Complex($label)
                }
            }
        )+
    };
}

as_complex! {
    u8 => "u8",
    u16 => "u16",
    u64 => "u64",
    i8 => "i8",
    i16 => "i16",
    crate::def::wire::DefString => "DefString",
    crate::def::wire::DefIndex => "DefIndex",
    crate::def::wire::PString => "PString",
}

impl<T> AsField for Vec<T> {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Complex("Vec")
    }
}
impl<K, V> AsField for std::collections::BTreeMap<K, V> {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Complex("BTreeMap")
    }
}
impl<K, V> AsField for crate::def::wire::VecMap<K, V> {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Complex("VecMap")
    }
}
impl<T, const N: usize> AsField for [T; N] {
    fn as_field(&mut self) -> FieldRef<'_> {
        FieldRef::Complex("array")
    }
}
