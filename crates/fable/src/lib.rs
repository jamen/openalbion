//! This crate is to use the assets of Fable, Fable: The Lost Chapters, Fable Anniversary, and their mods.
//!
//! See also [`fable_ffi`]. A lot of stuff is baked into the executable instead of the assets.
//!
//! | Format       | Description                                      |
//! |--------------|--------------------------------------------------|
//! | [`bba`]      | Animation format.                                |
//! | [`bbm`]      | Mesh format.                                     |
//! | [`big`]      | Graphics archive containing [`bba`] and [`bbm`]. |
//! | [`bncfg`]    | Bone config.                                     |
//! | [`def`]      | Definition source code.                          |
//! | [`def_bin`]  | Definition binary.                               |
//! | [`fmp`]      | Mod packages from [fabletlcmod.com].             |
//! | [`gtg`]      |                                                  |
//! | [`ini`]      | Game configs (and debug scripts?)                |
//! | [`lev`]      | Level heightmap and cell data.                   |
//! | [`lut`]      |                                                  |
//! | [`met`]      |                                                  |
//! | [`qst`]      |                                                  |
//! | [`save`]     | Game save format.                                |
//! | [`save_bin`] | Bin file included with save files.               |
//! | [`stb`]      | Archive containing [`stb_lev`].                  |
//! | [`stb_lev`]  |                                                  |
//! | [`tng`]      | Thing scripts.                                   |
//! | [`wad`]      | World archive containing [`lev`] and [`tng`].    |
//! | [`wld`]      |                                                  |
//!
//! [`bba`]: bba/index.html
//! [`bbm`]: bbm/index.html
//! [`big`]: big/index.html
//! [`bncfg`]: bncfg/index.html
//! [`def`]: def/index.html
//! [`def_bin`]: def_bin/index.html
//! [`fmp`]: fmp/index.html
//! [`gtg`]: gtg/index.html
//! [`ini`]: ini/index.html
//! [`lev`]: lev/index.html
//! [`lut`]: lut/index.html
//! [`met`]: met/index.html
//! [`qst`]: qst/index.html
//! [`save`]: save/index.html
//! [`save_bin`]: save_bin/index.html
//! [`stb`]: stb/index.html
//! [`stb_lev`]: stb_lev/index.html
//! [`tng`]: tng/index.html
//! [`wad`]: wad/index.html
//! [`wld`]: wld/index.html
//! [`fable_ffi`]: ../fable_ffi/index.html
//! [fabletlcmod.com]: http://fabletlcmod.com

pub mod bba;
pub mod bbm;
pub mod big;
pub mod bncfg;
pub mod bwd;
pub mod def;
pub mod def_bin;
pub mod error;
pub mod fmp;
pub mod gtg;
pub mod ini;
pub mod lev;
pub mod lut;
pub mod met;
pub mod qst;
pub mod save;
pub mod save_bin;
pub mod script;
pub mod shared;
pub mod stb;
pub mod stb_lev;
pub mod tng;
pub mod wad;
pub mod wld;

pub use bba::*;
pub use bbm::*;
pub use big::*;
pub use bncfg::*;
pub use def::*;
pub use def_bin::*;
pub use error::*;
pub use fmp::*;
pub use gtg::*;
pub use lev::*;
pub use lut::*;
pub use met::*;
pub use qst::*;
pub use save::*;
// pub use shared::*;
pub use script::*;
pub use stb::*;
pub use stb_lev::*;
pub use tng::*;
pub use wad::*;
pub use wld::*;

use std::io::{Read,Write,Seek};

/// The trait that all decoders implement. (See implementors)
pub trait Decode<Item: Sized>: Read + Seek {
    fn decode(&mut self) -> Result<Item, Error>;
}

/// The trait that all encoders implement. (See implementors)
pub trait Encode<Item: Sized>: Write + Seek {
    fn encode(&mut self, item: Item) -> Result<(), Error>;
}