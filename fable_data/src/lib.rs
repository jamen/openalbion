#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
extern crate core;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(bncfg_parser, "/bncfg_parser.rs");
lalrpop_mod!(ini_parser, "/ini_parser.rs");
lalrpop_mod!(qst_parser, "/qst_parser.rs");
lalrpop_mod!(fields_parser, "/fields_parser.rs");

pub mod anim;
pub mod big;
pub mod bncfg;
pub mod bwd;
pub(crate) mod bytes;
pub(crate) mod crc32;
pub mod def;
pub mod fields;
pub mod gtg;
pub mod ini;
pub mod lev;
pub mod lug;
pub mod met;
pub mod model;
pub mod qst;
pub mod save;
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
pub use ini::*;
pub use lug::*;
pub use met::*;
pub use model::*;
pub use save::*;
pub use texture::*;
// pub(crate) use crc32::*;
pub use def::*;
pub use fields::*;
pub use gtg::*;
pub use lev::*;
pub use qst::*;
pub use stb::*;
pub use tng::*;
pub use wad::*;
pub use wld::*;
