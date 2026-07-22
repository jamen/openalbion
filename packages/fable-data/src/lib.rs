pub mod big;
pub mod bytes;
pub mod crc32;
pub mod def;
pub mod environment;
pub mod kv;
pub mod texture;
pub mod tga;
pub mod tng;
pub mod wad;
// mod bba;
pub mod mesh;
// mod bbm;
// mod bncfg;
// mod bwd;
// mod gtg;
// mod ini;
// mod lug;
// mod lut;
pub mod lev;
// mod met;
// mod qst;
// mod save;
// mod stb;
pub mod wld;

/// Proc-macro derives for the def wire model (see [`def::wire`] / [`def::enums`]).
/// Re-exported at the crate root so the generated `crate::def::…` paths resolve
/// and def modules can `use crate::{DefStruct, WireStruct, …}`.
pub use fable_data_derive::{DefEnum, DefFlags, DefStruct, DefVariant, WireStruct};
