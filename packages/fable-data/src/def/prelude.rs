//! Internal prelude for the generated def and value modules: the field types a
//! def body can reference. Macros are imported explicitly per file.

pub use crate::def::enums::*;
pub use crate::def::values::*;
pub use crate::def::wire::{DefIndex, DefString, PString, VecMap, WStr, Wire};
pub use std::collections::BTreeMap;

// Sub-def classes appear inline in other defs' bodies (and their lists), so the
// def modules need each other's types in scope.
pub use crate::def::defs::*;
