//! A `#![no_std]` library for parsing and serializing Fable's game files.
//!
//! ## Features
//!
//! - `std`: Use `std` instead of `core` and `alloc`.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(bncfg_parser, "/bncfg_parser.rs");
lalrpop_mod!(script_parser, "/script_parser.rs");

pub mod anim;
pub mod big;
pub mod bncfg;
pub mod bwd;
pub(crate) mod bytes;
pub(crate) mod crc32;
pub mod def;
pub mod gtg;
pub mod ini;
pub mod lev;
pub mod lug;
pub mod met;
pub mod model;
pub mod qst;
pub mod save;
pub(crate) mod script;
pub mod stb;
pub mod texture;
pub mod tng;
pub mod wad;
pub mod wld;

pub use anim::*;
pub use big::*;
pub use bncfg::*;
pub use bwd::*;
pub(crate) use bytes::*;
// pub(crate) use crc32::*;
pub use def::*;
pub use gtg::*;
pub use ini::*;
pub use lev::*;
pub use lug::*;
pub use met::*;
pub use model::*;
pub use qst::*;
pub use save::*;
pub(crate) use script::*;
pub use stb::*;
pub use texture::*;
pub use tng::*;
pub use wad::*;
pub use wld::*;
