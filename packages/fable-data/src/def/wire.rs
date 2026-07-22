//! The def wire model.
//!
//! A def body is a sequence of *controls*: `u32 crc32(field name)` followed by
//! the field's value. [`Wire`] describes how a VALUE (the bytes after the id)
//! parses, serializes, and measures — the field's Rust type fully determines
//! its wire format, so a def's parse/serialize/size can never disagree with
//! its struct declaration.
//!
//! Value encodings (verified byte-exactly against the retail `game.bin`,
//! `frontend.bin`, and `script.bin` — see `fable-decomp/defs-spec.json`):
//!
//! | Rust type            | wire                                            |
//! |----------------------|-------------------------------------------------|
//! | `f32`/`i32`/`u32`    | 4 bytes LE                                      |
//! | `bool`               | 1 byte (`0`/`1`; any non-zero parses as `true`) |
//! | `String`             | `CCharString`: UTF-8, NUL-terminated            |
//! | [`WStr`]             | `CWideString`: UTF-16LE, NUL(2)-terminated      |
//! | `Vec<T>`             | `u32` count, then `count × T`                   |
//! | `[T; N]`             | `N × T` (no count)                              |
//! | `BTreeMap<K, V>`     | `u32` count, then `count × (K, V)` in key order (C++ `std::map`) |
//! | [`VecMap<K, V>`]     | like `BTreeMap` but preserving stored order (C++ `CVectorMap`) |
//! | [`def_enum!`] types  | 4 bytes LE, must be in the C++ enum table       |
//! | [`def_flags!`] types | 4 bytes LE, any value                           |
//! | [`wire_struct!`] types | member values in order (no inner ids)         |
//!
//! [`def_enum!`]: crate::enums
//! [`def_flags!`]: crate::enums

use std::collections::BTreeMap;

use crate::bytes::{
    TakeError, TakeNullTerminatedUtf8, TakeNullTerminatedUtf16, UnexpectedEnd, put, put_le,
    put_null_terminated_utf8, put_null_terminated_utf16, take, take_le, take_null_terminated_utf8,
    take_null_terminated_utf16,
};
use crate::crc32::crc;
use crate::def::binary::control::{
    ID_BYTE_SIZE, ParseControlError, ParseControlErrorReason, SerializeControlError,
    SerializeControlErrorReason,
};

// ── Wire ──────────────────────────────────────────────────────────────────────

/// A value on the def wire: the bytes of one field after its control id (or of
/// one container element / compound member, which carry no ids of their own).
pub trait Wire: Sized {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError>;

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd>;

    /// Exact number of bytes [`Wire::serialize`] writes.
    fn wire_size(&self) -> usize;
}

#[derive(Debug)]
pub enum ParseWireError {
    UnexpectedEnd,
    /// The value isn't in the field's C++ enum table.
    InvalidEnumValue { value: i32 },
    /// A tagged-union tag with no matching case.
    InvalidVariantTag(u32),
    Utf8(TakeNullTerminatedUtf8),
    Utf16(TakeNullTerminatedUtf16),
    /// Error inside the `index`th element of a container.
    Item { index: usize, inner: Box<ParseWireError> },
    /// Error inside a named member of a [`wire_struct!`] compound.
    Member { name: &'static str, inner: Box<ParseWireError> },
    /// Error inside a sub-component: a value that is itself a stream of
    /// id-carrying field controls (e.g. one `UiDef` state).
    Field(Box<ParseControlError>),
}

impl From<ParseControlError> for ParseWireError {
    fn from(error: ParseControlError) -> Self {
        ParseWireError::Field(Box::new(error))
    }
}

impl SerializeControlError {
    /// All field-serialize failures are buffer exhaustion; unwrap to the plain
    /// [`UnexpectedEnd`] that [`Wire::serialize`] reports.
    pub fn unexpected_end(self) -> UnexpectedEnd {
        UnexpectedEnd
    }
}

impl From<TakeError> for ParseWireError {
    fn from(error: TakeError) -> Self {
        match error {
            // Def wire scalars are plain little-endian values; a `PodCast`
            // failure can only mean the input ran out mid-value.
            TakeError::UnexpectedEnd(UnexpectedEnd) | TakeError::PodCast(_) => {
                ParseWireError::UnexpectedEnd
            }
        }
    }
}

impl ParseWireError {
    pub fn item(index: usize) -> impl FnOnce(ParseWireError) -> ParseWireError {
        move |inner| ParseWireError::Item { index, inner: Box::new(inner) }
    }

    pub fn member(name: &'static str) -> impl FnOnce(ParseWireError) -> ParseWireError {
        move |inner| ParseWireError::Member { name, inner: Box::new(inner) }
    }
}

// ── scalars ───────────────────────────────────────────────────────────────────

macro_rules! wire_scalar {
    ($($ty:ty),+) => {
        $(
            impl Wire for $ty {
                fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
                    Ok(take_le::<$ty>(cur)?)
                }

                fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
                    put_le(out, self)
                }

                fn wire_size(&self) -> usize {
                    size_of::<$ty>()
                }
            }
        )+
    };
}

wire_scalar!(f32, i32, u32);

macro_rules! wire_scalar_int {
    ($($ty:ty),+) => {
        $(
            impl Wire for $ty {
                fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
                    Ok(take_le::<$ty>(cur)?)
                }
                fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
                    put_le(out, self)
                }
                fn wire_size(&self) -> usize {
                    size_of::<$ty>()
                }
            }
        )+
    };
}

wire_scalar_int!(u64, u16, u8, i16, i8);

/// A `CDefString`: a 4-byte reference into the compiled name table (its
/// `TablePos`). Distinct from a plain `i32` so def-string fields are visible in
/// the type; resolving the referenced string is a link-time concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DefString(pub i32);

/// A `CDefIndex`: a 4-byte reference to another def's global entry index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DefIndex(pub i32);

macro_rules! wire_newtype_i32 {
    ($($ty:ident),+) => {
        $(
            impl Wire for $ty {
                fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
                    Ok($ty(i32::parse(cur)?))
                }
                fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
                    self.0.serialize(out)
                }
                fn wire_size(&self) -> usize {
                    size_of::<i32>()
                }
            }
        )+
    };
}

wire_newtype_i32!(DefString, DefIndex);

/// A length-prefixed byte string (`u32` length + raw bytes, no terminator), as
/// written by the game's `WriteString`/`ReadString`. Stored as bytes since it
/// isn't guaranteed valid UTF-8.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PString(pub Vec<u8>);

impl Wire for PString {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        let len = u32::parse(cur)? as usize;
        if cur.len() < len {
            return Err(ParseWireError::UnexpectedEnd);
        }
        let (bytes, rest) = cur.split_at(len);
        *cur = rest;
        Ok(PString(bytes.to_vec()))
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        (self.0.len() as u32).serialize(out)?;
        crate::bytes::put_bytes(out, &self.0)
    }

    fn wire_size(&self) -> usize {
        size_of::<u32>() + self.0.len()
    }
}

impl Wire for bool {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        Ok(take::<u8>(cur).map_err(ParseWireError::from)? != 0)
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put(out, &(*self as u8))
    }

    fn wire_size(&self) -> usize {
        size_of::<u8>()
    }
}

// ── strings ───────────────────────────────────────────────────────────────────

/// `CCharString`: UTF-8, NUL-terminated.
impl Wire for String {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        take_null_terminated_utf8(cur)
            .map(str::to_owned)
            .map_err(ParseWireError::Utf8)
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_null_terminated_utf8(out, self)
    }

    fn wire_size(&self) -> usize {
        self.len() + 1
    }
}

/// `CWideString`: UTF-16LE, NUL-terminated. A distinct type so the field's
/// declaration picks the encoding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WStr(pub String);

impl Wire for WStr {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        take_null_terminated_utf16(cur)
            .map(WStr)
            .map_err(ParseWireError::Utf16)
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        put_null_terminated_utf16(out, &self.0)
    }

    fn wire_size(&self) -> usize {
        (self.0.encode_utf16().count() + 1) * size_of::<u16>()
    }
}

impl std::fmt::Display for WStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for WStr {
    fn from(value: &str) -> Self {
        WStr(value.to_owned())
    }
}

impl From<String> for WStr {
    fn from(value: String) -> Self {
        WStr(value)
    }
}

// ── containers ────────────────────────────────────────────────────────────────

/// `u32` count, then the elements.
impl<T: Wire> Wire for Vec<T> {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        let count = u32::parse(cur)?;
        (0..count as usize)
            .map(|i| T::parse(cur).map_err(ParseWireError::item(i)))
            .collect()
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        (self.len() as u32).serialize(out)?;
        self.iter().try_for_each(|item| item.serialize(out))
    }

    fn wire_size(&self) -> usize {
        size_of::<u32>() + self.iter().map(Wire::wire_size).sum::<usize>()
    }
}

/// Fixed-size run of values, no count prefix.
impl<T: Wire, const N: usize> Wire for [T; N] {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        let mut items = Vec::with_capacity(N);
        for i in 0..N {
            items.push(T::parse(cur).map_err(ParseWireError::item(i))?);
        }
        Ok(items.try_into().unwrap_or_else(|_| unreachable!()))
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        self.iter().try_for_each(|item| item.serialize(out))
    }

    fn wire_size(&self) -> usize {
        self.iter().map(Wire::wire_size).sum()
    }
}

/// C++ `std::map`: `u32` count, then `(key, value)` pairs in key order.
impl<K: Wire + Ord, V: Wire> Wire for BTreeMap<K, V> {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        let count = u32::parse(cur)?;
        (0..count as usize)
            .map(|i| {
                let key = K::parse(cur).map_err(ParseWireError::item(i))?;
                let value = V::parse(cur).map_err(ParseWireError::item(i))?;
                Ok((key, value))
            })
            .collect()
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        (self.len() as u32).serialize(out)?;
        self.iter().try_for_each(|(key, value)| {
            key.serialize(out)?;
            value.serialize(out)
        })
    }

    fn wire_size(&self) -> usize {
        size_of::<u32>()
            + self
                .iter()
                .map(|(key, value)| key.wire_size() + value.wire_size())
                .sum::<usize>()
    }
}

/// A map that preserves stored pair order (C++ `CVectorMap` and friends).
/// Same wire shape as [`BTreeMap`], but round-trips whatever order the data
/// has instead of imposing key order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VecMap<K, V>(pub Vec<(K, V)>);

impl<K: Wire, V: Wire> Wire for VecMap<K, V> {
    fn parse(cur: &mut &[u8]) -> Result<Self, ParseWireError> {
        let count = u32::parse(cur)?;
        (0..count as usize)
            .map(|i| {
                let key = K::parse(cur).map_err(ParseWireError::item(i))?;
                let value = V::parse(cur).map_err(ParseWireError::item(i))?;
                Ok((key, value))
            })
            .collect::<Result<_, _>>()
            .map(VecMap)
    }

    fn serialize(&self, out: &mut &mut [u8]) -> Result<(), UnexpectedEnd> {
        (self.0.len() as u32).serialize(out)?;
        self.0.iter().try_for_each(|(key, value)| {
            key.serialize(out)?;
            value.serialize(out)
        })
    }

    fn wire_size(&self) -> usize {
        size_of::<u32>()
            + self
                .0
                .iter()
                .map(|(key, value)| key.wire_size() + value.wire_size())
                .sum::<usize>()
    }
}

impl<K: PartialEq, V> VecMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Replace the value of an existing key or append a new pair.
    pub fn insert(&mut self, key: K, value: V) {
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => *v = value,
            None => self.0.push((key, value)),
        }
    }
}

// ── compounds ─────────────────────────────────────────────────────────────────

/// A compound wire value: a fixed sequence of member values with NO control
/// ids, as produced by the game's `TransferBinaryIn`/`Out` for value types
/// (`C3DVector`, `CEngineGraphic`, `CActionInputControl`, …).
#[macro_export]
macro_rules! wire_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$fmeta:meta])* pub $field:ident: $ty:ty $(= $default:expr)?, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $( $(#[$fmeta])* pub $field: $ty, )+
        }

        impl $crate::def::visit::DefDefault for $name {
            fn def_default() -> Self {
                Self { $( $field: $crate::def_field_default!($ty $(, $default)?), )+ }
            }
        }

        impl $crate::def::wire::Wire for $name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, $crate::def::wire::ParseWireError> {
                Ok(Self {
                    $(
                        $field: <$ty as $crate::def::wire::Wire>::parse(cur)
                            .map_err($crate::def::wire::ParseWireError::member(
                                stringify!($field),
                            ))?,
                    )+
                })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), $crate::bytes::UnexpectedEnd> {
                $( self.$field.serialize(out)?; )+
                Ok(())
            }

            fn wire_size(&self) -> usize {
                0 $( + self.$field.wire_size() )+
            }
        }

        impl $crate::def::visit::StructSlot for $name {
            fn type_name(&self) -> &'static str {
                stringify!($name)
            }
            fn member_count(&self) -> usize {
                0 $(+ { let _ = stringify!($field); 1 })+
            }
            fn member_name(&self, index: usize) -> Option<&'static str> {
                [$(stringify!($field)),+].get(index).copied()
            }
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn member<'b>(&'b mut self, index: usize) -> Option<$crate::def::visit::FieldRef<'b>> {
                let mut i = 0usize;
                $(
                    if i == index {
                        return Some($crate::def::visit::AsField::as_field(&mut self.$field));
                    }
                    i += 1;
                )+
                None
            }
        }

        impl $crate::def::visit::AsField for $name {
            fn as_field(&mut self) -> $crate::def::visit::FieldRef<'_> {
                $crate::def::visit::FieldRef::Struct(self)
            }
        }
    };
}

// ── def classes ───────────────────────────────────────────────────────────────

/// A def class: a struct whose body is a sequence of field controls, each
/// `u32 crc32(field name)` + [`Wire`] value, in declaration order.
///
/// ```ignore
/// def_struct! {
///     /// `ENGINE` — `CEngineDef`.
///     #[derive(Debug, Clone, PartialEq, Default)]
///     pub struct EngineDef {
///         "LODErrorTolerance" => pub lod_error_tolerance: f32,
///         "SeaHeight" => pub sea_height: f32,
///     }
/// }
/// ```
///
/// Generates the struct plus `parse`/`serialize`/`byte_size` (control-level,
/// with field-name error context) and a delegating [`Wire`] impl — a def class
/// is itself a wire value, because sub-component defs appear inline in other
/// defs' bodies (and as elements of their lists).
/// A field's default in a [`def_struct!`]: the explicit `= expr` if given, else
/// the type's [`DefDefault`]. Some def fields have non-zero game constructor
/// defaults (e.g. `ReceiveShadows` defaults to `true`); the `= expr` form
/// captures those without special-casing lowering.
#[macro_export]
macro_rules! def_field_default {
    ($ty:ty) => { <$ty as $crate::def::visit::DefDefault>::def_default() };
    ($ty:ty, $default:expr) => { $default };
}

#[macro_export]
macro_rules! def_struct {
    // One or more structs in a single macro call.
    (
        $(
            $(#[$meta:meta])*
            pub struct $name:ident {
                $( $(#[$fmeta:meta])* $wire_name:literal => pub $field:ident: $ty:ty $(= $default:expr)?, )+
            }
        )+
    ) => {
        $(
            $(#[$meta])*
            pub struct $name {
                $( $(#[$fmeta])* pub $field: $ty, )+
            }

            impl Default for $name {
                fn default() -> Self {
                    Self { $( $field: $crate::def_field_default!($ty $(, $default)?), )+ }
                }
            }

            impl $crate::def::visit::DefDefault for $name {
                fn def_default() -> Self {
                    Self::default()
                }
            }

            impl $name {
                pub(crate) fn parse(
                    cur: &mut &[u8],
                ) -> Result<Self, $crate::def::binary::control::ParseControlError> {
                    Ok(Self {
                        $( $field: $crate::def::wire::parse_field(cur, $wire_name)?, )+
                    })
                }

                pub(crate) fn serialize(
                    &self,
                    out: &mut &mut [u8],
                ) -> Result<(), $crate::def::binary::control::SerializeControlError> {
                    $( $crate::def::wire::serialize_field(out, $wire_name, &self.$field)?; )+
                    Ok(())
                }

                pub(crate) fn byte_size(&self) -> usize {
                    0 $( + $crate::def::wire::field_size(&self.$field) )+
                }

                /// Hand each field to `visitor` as a typed
                /// [`FieldRef`](crate::def::visit::FieldRef), in declaration order.
                pub fn visit_fields<V: $crate::def::visit::FieldVisitor>(&mut self, visitor: &mut V) {
                    $(
                        visitor.field(
                            $wire_name,
                            $crate::def::visit::AsField::as_field(&mut self.$field),
                        );
                    )+
                }
            }

            impl $crate::def::visit::VisitFields for $name {
                fn visit_fields<V: $crate::def::visit::FieldVisitor>(&mut self, visitor: &mut V) {
                    $name::visit_fields(self, visitor);
                }
            }

            impl $crate::def::wire::Wire for $name {
                fn parse(
                    cur: &mut &[u8],
                ) -> Result<Self, $crate::def::wire::ParseWireError> {
                    $name::parse(cur).map_err($crate::def::wire::ParseWireError::from)
                }

                fn serialize(
                    &self,
                    out: &mut &mut [u8],
                ) -> Result<(), $crate::bytes::UnexpectedEnd> {
                    $name::serialize(self, out)
                        .map_err($crate::def::binary::control::SerializeControlError::unexpected_end)
                }

                fn wire_size(&self) -> usize {
                    self.byte_size()
                }
            }

            impl $crate::def::visit::StructSlot for $name {
                fn type_name(&self) -> &'static str {
                    stringify!($name)
                }
                fn member_count(&self) -> usize {
                    0 $(+ { let _ = stringify!($field); 1 })+
                }
                fn member_name(&self, index: usize) -> Option<&'static str> {
                    [$(stringify!($field)),+].get(index).copied()
                }
                #[allow(unused_assignments, unused_variables, unused_mut)]
                fn member<'b>(&'b mut self, index: usize) -> Option<$crate::def::visit::FieldRef<'b>> {
                    let mut i = 0usize;
                    $(
                        if i == index {
                            return Some($crate::def::visit::AsField::as_field(&mut self.$field));
                        }
                        i += 1;
                    )+
                    None
                }
                fn visit_named(&mut self, visitor: &mut dyn $crate::def::visit::FieldVisitor) -> bool {
                    // `def_struct!` fields carry their def-script names, so a
                    // def_struct used as a `Vec` element / map value can be
                    // lowered by name (nested `Field[i].Subfield …`).
                    let mut fwd: &mut dyn $crate::def::visit::FieldVisitor = visitor;
                    $name::visit_fields(self, &mut fwd);
                    true
                }
            }

            impl $crate::def::visit::AsField for $name {
                fn as_field(&mut self) -> $crate::def::visit::FieldRef<'_> {
                    $crate::def::visit::FieldRef::Struct(self)
                }
            }
        )+
    };
}

/// A tagged-union wire value: a `u32` tag then case-specific fields (the game's
/// polymorphic sub-types — animation components, physical primitives, reaction
/// matches, …). Each case is a tuple-struct-like variant of named wire fields.
///
/// ```ignore
/// def_variant! {
///     pub enum PhysicalPrimitive: u32 {
///         0 => Null {},
///         1 => Sphere { base_name: DefIndex, radius: f32 },
///         2 => Cylinder { base_name: DefIndex, radius: f32, height: f32 },
///     }
/// }
/// ```
#[macro_export]
macro_rules! def_variant {
    (
        $(#[$meta:meta])*
        pub enum $name:ident: u32 {
            $( $tag:literal => $variant:ident { $( $field:ident: $ty:ty ),* $(,)? } ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $( $variant { $( $field: $ty ),* } ),+
        }

        impl $crate::def::visit::DefDefault for $name {
            fn def_default() -> Self {
                // The first (tag-0) variant is the conventional default.
                [$( Self::$variant { $( $field: $crate::def::visit::DefDefault::def_default() ),* } ),+]
                    .into_iter()
                    .next()
                    .unwrap()
            }
        }

        impl $crate::def::wire::Wire for $name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, $crate::def::wire::ParseWireError> {
                let tag = <u32 as $crate::def::wire::Wire>::parse(cur)?;
                Ok(match tag {
                    $(
                        $tag => Self::$variant {
                            $( $field: <$ty as $crate::def::wire::Wire>::parse(cur)
                                .map_err($crate::def::wire::ParseWireError::member(
                                    stringify!($field),
                                ))?, )*
                        },
                    )+
                    other => {
                        return Err($crate::def::wire::ParseWireError::InvalidVariantTag(other));
                    }
                })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), $crate::bytes::UnexpectedEnd> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            <u32 as $crate::def::wire::Wire>::serialize(&$tag, out)?;
                            $( $field.serialize(out)?; )*
                        }
                    )+
                }
                Ok(())
            }

            fn wire_size(&self) -> usize {
                size_of::<u32>() + match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            0 $( + $field.wire_size() )*
                        }
                    )+
                }
            }
        }

        impl $crate::def::visit::VariantSlot for $name {
            fn type_name(&self) -> &'static str {
                stringify!($name)
            }
            fn tag(&self) -> u32 {
                match self {
                    $( Self::$variant { .. } => $tag, )+
                }
            }
            fn set_tag(&mut self, tag: u32) -> bool {
                match tag {
                    $(
                        $tag => {
                            *self = Self::$variant { $( $field: $crate::def::visit::DefDefault::def_default(), )* };
                            true
                        }
                    )+
                    _ => false,
                }
            }
            fn member_count(&self) -> usize {
                match self {
                    $( Self::$variant { .. } => 0 $(+ { let _ = stringify!($field); 1 })*, )+
                }
            }
            fn member_name(&self, index: usize) -> Option<&'static str> {
                match self {
                    $( Self::$variant { .. } => [$(stringify!($field)),*].get(index).copied(), )+
                }
            }
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn member<'b>(&'b mut self, index: usize) -> Option<$crate::def::visit::FieldRef<'b>> {
                match self {
                    $(
                        Self::$variant { $( $field ),* } => {
                            let mut i = 0usize;
                            $(
                                if i == index {
                                    return Some($crate::def::visit::AsField::as_field($field));
                                }
                                i += 1;
                            )*
                            None
                        }
                    )+
                }
            }
        }

        impl $crate::def::visit::AsField for $name {
            fn as_field(&mut self) -> $crate::def::visit::FieldRef<'_> {
                $crate::def::visit::FieldRef::Variant(self)
            }
        }
    };
}

// ── def classes ───────────────────────────────────────────────────────────
/// Parse one field control: `u32 crc32(name)` (validated) then the value.
pub fn parse_field<T: Wire>(
    cur: &mut &[u8],
    name: &'static str,
) -> Result<T, ParseControlError> {
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

    T::parse(cur).map_err(|inner| ParseControlError {
        name,
        reason: ParseControlErrorReason::Wire(inner),
    })
}

/// Serialize one field control: `u32 crc32(name)` then the value.
pub fn serialize_field<T: Wire>(
    out: &mut &mut [u8],
    name: &'static str,
    value: &T,
) -> Result<(), SerializeControlError> {
    put_le(out, &crc(name.as_bytes())).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::MalformedId(inner),
    })?;

    value.serialize(out).map_err(|inner| SerializeControlError {
        name,
        reason: SerializeControlErrorReason::Value(inner),
    })
}

/// Size of one field control: id + value.
pub fn field_size<T: Wire>(value: &T) -> usize {
    ID_BYTE_SIZE + value.wire_size()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def::enums::{ControllerType, TableExpansion};

    fn round_trip<T: Wire + PartialEq + std::fmt::Debug>(value: T) {
        let mut buf = vec![0u8; value.wire_size()];
        let mut out = &mut buf[..];
        value.serialize(&mut out).unwrap();
        assert!(out.is_empty(), "wire_size over-estimated");
        let mut cur = &buf[..];
        let parsed = T::parse(&mut cur).unwrap();
        assert!(cur.is_empty(), "parse under-consumed");
        assert_eq!(parsed, value);
    }

    #[test]
    fn scalars_round_trip() {
        round_trip(1.5f32);
        round_trip(-7i32);
        round_trip(0xdead_beefu32);
        round_trip(true);
        round_trip(false);
    }

    #[test]
    fn strings_round_trip() {
        round_trip(String::from("ENG_ARIAL_24"));
        round_trip(String::new());
        round_trip(WStr::from("Data\\Video\\intro.wmv"));
        round_trip(WStr::default());
    }

    #[test]
    fn containers_round_trip() {
        round_trip(vec![1i32, 2, 3]);
        round_trip(Vec::<f32>::new());
        round_trip([0.5f32, -0.5]);
        round_trip(BTreeMap::from([(1u32, String::from("a")), (2, String::from("b"))]));
        round_trip(VecMap(vec![
            (String::from("MINIMAP_ARENA"), 103i32),
            (String::from("MINIMAP_ABYSS"), 7),
        ]));
    }

    #[test]
    fn enums_round_trip() {
        round_trip(ControllerType::Mouse);
        round_trip(TableExpansion::HORIZONTAL | TableExpansion::VERTICAL);
    }

    #[test]
    fn enum_rejects_out_of_table() {
        let bytes = 99i32.to_le_bytes();
        let mut cur = &bytes[..];
        assert!(matches!(
            ControllerType::parse(&mut cur),
            Err(ParseWireError::InvalidEnumValue { value: 99 })
        ));
    }

    #[test]
    fn field_controls() {
        let mut buf = vec![0u8; field_size(&1.5f32)];
        let mut out = &mut buf[..];
        serialize_field(&mut out, "SeaHeight", &1.5f32).unwrap();
        let mut cur = &buf[..];
        let value: f32 = parse_field(&mut cur, "SeaHeight").unwrap();
        assert_eq!(value, 1.5);

        // wrong field name → WrongId
        let mut cur = &buf[..];
        assert!(parse_field::<f32>(&mut cur, "NotSeaHeight").is_err());
    }

    wire_struct! {
        /// Test compound.
        pub struct TestCompound {
            pub scale: f32,
            pub kind: ControllerType,
            pub names: Vec<String>,
        }
    }

    #[test]
    fn wire_struct_round_trip_and_member_errors() {
        let value = TestCompound {
            scale: 2.0,
            kind: ControllerType::Keyboard,
            names: vec![String::from("a"), String::from("b")],
        };
        let mut buf = vec![0u8; value.wire_size()];
        let mut out = &mut buf[..];
        value.serialize(&mut out).unwrap();
        round_trip(value);

        // truncated input names the failing member
        let mut cur = &buf[..2];
        match TestCompound::parse(&mut cur) {
            Err(ParseWireError::Member { name: "scale", .. }) => {}
            other => panic!("expected member error on scale, got {other:?}"),
        }
    }
}
