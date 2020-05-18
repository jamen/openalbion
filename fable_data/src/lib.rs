//! A Rust library for using the assets of Fable, Fable: The Lost Chapters, Fable Anniversary, and mods.
//!
//! Work in progress.
//!
//! | Format       | Description                                      |
//! |--------------|--------------------------------------------------|
//! | [`Bba`]      | Animation format.                                |
//! | [`Bbm`]      | Mesh format.                                     |
//! | [`Big`]      | Graphics archive containing [`Bba`] and [`Bbm`]. |
//! | [`Bncfg`]    | Bone config.                                     |
//! | [`Bwd`]      |                                                  |
//! | [`Def`]      | Definition source code.                          |
//! | [`DefBin`]   | Definition binary.                               |
//! | [`Fmp`]      | Mod packages from [fabletlcmod.com].             |
//! | [`Gtg`]      |                                                  |
//! | [`Ini`]      | Game configs (and debug scripts?)                |
//! | [`Lev`]      | Level heightmap and cell data.                   |
//! | [`Lug`]      |                                                  |
//! | [`Lut`]      |                                                  |
//! | [`Met`]      |                                                  |
//! | [`Qst`]      |                                                  |
//! | [`Save`]     | Game save formats.                               |
//! | [`Stb`]      | A cache of local details generated from [`Lev`]. |
//! | [`Tng`]      | Thing scripts.                                   |
//! | [`Wad`]      | World archive containing [`Lev`] and [`Tng`].    |
//! | [`Wld`]      |                                                  |
//!
//! [`Bba`]: struct.Bba.html
//! [`Bbm`]: struct.Bbm.html
//! [`Big`]: struct.Big.html
//! [`Bncfg`]: struct.Bncfg.html
//! [`Bwd`]: struct.Bwd.html
//! [`Def`]: struct.Def.html
//! [`DefBin`]: struct.DefBin.html
//! [`Fmp`]: struct.Fmp.html
//! [`Gtg`]: struct.Gtg.html
//! [`Ini`]: struct.Ini.html
//! [`Lev`]: struct.Lev.html
//! [`Lug`]: struct.Lug.html
//! [`Lut`]: struct.Lut.html
//! [`Met`]: struct.Met.html
//! [`Qst`]: struct.Qst.html
//! [`Save`]: struct.Save.html
//! [`Stb`]: struct.Stb.html
//! [`Tng`]: struct.Tng.html
//! [`Wad`]: struct.Wad.html
//! [`Wld`]: struct.Wld.html
//! [fabletlcmod.com]: http://fabletlcmod.com

mod bba;
mod bbm;
mod big;
mod bncfg;
mod bwd;
mod def;
mod def_bin;
mod fmp;
mod gtg;
mod ini;
mod lev;
mod lug;
mod lut;
mod met;
mod qst;
mod save;
mod script;
mod shared;
mod stb;
mod tng;
mod wad;
mod wld;

pub use bba::*;
pub use bbm::*;
pub use big::*;
pub use bncfg::*;
pub use def::*;
pub use def_bin::*;
pub use fmp::*;
pub use gtg::*;
pub use ini::*;
pub use lev::*;
pub use lug::*;
pub use lut::*;
pub use met::*;
pub use qst::*;
pub use save::*;
pub use script::*;
pub use shared::*;
pub use stb::*;
pub use tng::*;
pub use wad::*;
pub use wld::*;