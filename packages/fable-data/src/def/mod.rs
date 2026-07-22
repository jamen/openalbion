//! Typed Fable def model and text-parsing infrastructure.
//!
//! - [`binary`] — compiled def container (`DefBinary`, `DefBody`, etc.)
//! - [`defs`] — individual def-class structs (one module per type)
//! - [`dispatch`] — per-type metadata utilities (`def_name_has_subdef_table`)
//! - [`enums`] / [`values`] — shared enum and compound-value types
//! - [`wire`] — wire-level parse/serialize ([`Wire`])
//! - [`text`] — text-based definition parsing
//! - [`object`] — object-def utilities

pub mod binary;
pub mod defs;
pub mod dispatch;
pub mod enums;
pub mod object;
pub mod semantic;
pub mod text;
pub mod values;
pub mod visit;
pub mod wire;

pub use self::defs::*;
pub use self::enums::*;
pub use self::values::*;
pub use self::wire::{DefIndex, DefString, PString, VecMap, WStr, Wire};

pub use self::text::{DefParseError, Definition, Expr, PathSegment, Statement, parse_def_file};
