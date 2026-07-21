//! Typed Fable def model and text-parsing infrastructure.
//!
//! - [`binary`] — compiled def container (`DefBinary`, `DefBody`, etc.)
//! - [`defs`] — individual def-class structs (one module per type)
//! - [`dispatch`] — maps wire names to typed bodies ([`GameBody`])
//! - [`enums`] / [`values`] — shared enum and compound-value types
//! - [`wire`] — wire-level parse/serialize ([`Wire`])
//! - [`text`] — text-based definition parsing
//! - [`object`] — object-def utilities

pub mod binary;
pub mod defs;
pub mod dispatch;
pub mod enums;
#[macro_use]
pub mod game_def_table;
pub mod object;
pub mod prelude;
pub mod semantic;
pub mod text;
pub mod values;
pub mod visit;
pub mod wire;

pub use self::defs::*;
pub use self::dispatch::GameBody;
pub use self::enums::*;
pub use self::values::*;
pub use self::wire::{DefIndex, DefString, PString, VecMap, WStr, Wire};

pub use self::text::{DefParseError, Definition, Expr, PathSegment, Statement, parse_def_file};
