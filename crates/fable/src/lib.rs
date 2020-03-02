//! This crate is to use the assets of Fable, Fable: The Lost Chapters, Fable Anniversary, and their mods.
//!
//! See also [`fable_ffi`]. A lot of stuff is baked into the executable instead of the assets.
//!
//! | Format       | Description                                      |
//! |--------------|--------------------------------------------------|
//! | [`Bba`]      | Animation format.                                |
//! | [`Bbm`]      | Mesh format.                                     |
//! | [`Big`]      | Graphics archive containing [`bba`] and [`bbm`]. |
//! | [`Bncfg`]    | Bone config.                                     |
//! | [`Def`]      | Definition source code.                          |
//! | [`DefBin`]  | Definition binary.                                |
//! | [`Fmp`]      | Mod packages from [fabletlcmod.com].             |
//! | [`Gtg`]      |                                                  |
//! | [`Ini`]      | Game configs (and debug scripts?)                |
//! | [`Lev`]      | Level heightmap and cell data.                   |
//! | [`Lut`]      |                                                  |
//! | [`Met`]      |                                                  |
//! | [`Qst`]      |                                                  |
//! | [`Save`]     | Game save format.                                |
//! | [`SaveBin`] | Bin file included with save files.                |
//! | [`Stb`]      | Archive containing [`stb_lev`].                  |
//! | [`StbLev`]  |                                                   |
//! | [`Tng`]      | Thing scripts.                                   |
//! | [`Wad`]      | World archive containing [`lev`] and [`tng`].    |
//! | [`Wld`]      |                                                  |
//!
//! [`Bba`]: struct.Bba.html
//! [`Bbm`]: struct.Bbm.html
//! [`Big`]: struct.Big.html
//! [`Bncfg`]: struct.Bncfg.html
//! [`Def`]: struct.Def.html
//! [`DefBin`]: struct.DefBin.html
//! [`Fmp`]: struct.Fmp.html
//! [`Gtg`]: struct.Gtg.html
//! [`Ini`]: struct.Ini.html
//! [`Lev`]: struct.Lev.html
//! [`Lut`]: struct.Lut.html
//! [`Met`]: struct.Met.html
//! [`Qst`]: struct.Qst.html
//! [`Save`]: struct.Save.html
//! [`SaveBin`]: struct.SaveBin.html
//! [`Stb`]: struct.Stb.html
//! [`StbLev`]: struct.StbLev.html
//! [`Tng`]: struct.Tng.html
//! [`Wad`]: struct.Wad.html
//! [`Wld`]: struct.Wld.html
//! [`fable_ffi`]: ../fable_ffi/index.html
//! [fabletlcmod.com]: http://fabletlcmod.com

pub mod script;
pub mod shared;

mod bba;
mod bbm;
mod big;
mod bncfg;
mod bwd;
mod def;
mod def_bin;
mod error;
mod fmp;
mod gtg;
mod ini;
mod lev;
mod lug;
mod lut;
mod met;
mod qst;
mod save;
mod save_bin;
mod stb;
mod stb_lev;
mod tng;
mod wad;
mod wld;

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
pub use lug::*;
pub use lut::*;
pub use met::*;
pub use qst::*;
pub use save::*;
// pub use shared::*;
// pub use script::*;
pub use stb::*;
pub use stb_lev::*;
pub use tng::*;
pub use wad::*;
pub use wld::*;

use std::io::{Read,Write,Seek};

/// The trait that all decoders implement.
pub trait Decode<Item>: Read + Seek {
    fn decode(&mut self) -> Result<Item, Error>;
}

/// The trait that all encoders implement.
pub trait Encode<Item>: Write + Seek {
    fn encode(&mut self, item: Item) -> Result<(), Error>;
}