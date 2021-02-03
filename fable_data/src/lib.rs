//! |        Path        |                            Formats                            |
//! | ------------------ | ------------------------------------------------------------- |
//! | Data/CompiledDefs  | [`DefBin`]                                                    |
//! | Data/Defs          | [`DefBin`], C header                                          |
//! | Data/Bones         | [`Bncfg`]                                                     |
//! | Data/EngineCache   | [`Dat`]                                                       |
//! | Data/Graphics      | [`Big`]                                                       |
//! | Data/LightingTable | Tga                                                           |
//! | Data/Shaders       | [`Big`]                                                       |
//! | Data/Tattoos       | Bmp                                                           |
//! | Data/Levels        | [`Bwd`], [`Gtg`], [`Lev`], [`Qst`], [`Tng`], [`Wad`], [`Wld`] |
//! | Data/Misc          | [`Big`], [`Dat`], [`DefBin`], Dds, Tga, Text script           |
//! | Data/Lang          | [`DefBin`], [`Lut`], [`Big`], Text script                     |
//! | Data/Sound         | [`Lug`], [`Met`], Ogg                                         |
//! | Data/Video         | Wmv, Text script                                              |
//! | *.ini              | [`Ini`]                                                       |
//!
//! [`Bncfg`]: struct.Bncfg.html
//! [`DefBin`]: struct.DefBin.html
//! [`Dat`]: struct.Dat.html
//! [`Big`]: struct.Big.html
//! [`Lut`]: struct.Lut.html
//! [`Bwd`]: struct.Bwd.html
//! [`Gtg`]: struct.Gtg.html
//! [`Lev`]: struct.Lev.html
//! [`Qst`]: struct.Qst.html
//! [`Tng`]: struct.Tng.html
//! [`Wad`]: struct.Wad.html
//! [`Wld`]: struct.Wld.html
//! [`Lug`]: struct.Lug.html
//! [`Met`]: struct.Met.html
//! [`Ini`]: struct.Ini.html

// mod bba;
// mod bbm;
mod big;
// mod bncfg;
// mod bwd;
// mod dat;
// mod def;
// mod entry;
// mod error;
// mod gtg;
// mod ini;
mod lev;
// mod lug;
// mod lut;
// mod met;
// mod qst;
// mod save;
// mod script;
// mod shared;
// mod stb;
// mod tng;
mod wad;
// mod wld;

// pub use bba::*;
// pub use bbm::*;
pub use big::*;
// pub use bncfg::*;
// pub use bwd::*;
// pub use dat::*;
// pub use def::*;
// pub use entry::*;
// pub use error::*;
// pub use gtg::*;
// pub use ini::*;
pub use lev::*;
// pub use lug::*;
// pub use lut::*;
// pub use met::*;
// pub use qst::*;
// pub use save::*;
// pub use script::*;
// pub use shared::*;
// pub use stb::*;
// pub use tng::*;
pub use wad::*;
// pub use wld::*;

use views::{Bytes,BadPos,Look};

pub trait BytesExt: Bytes {
    fn take_with_u32_le_prefix(&mut self) -> Result<&[u8], BadPos> {
        let prefix = self.take_u32_le()?;
        let out = self.take(prefix as usize)?;
        Ok(out)
    }
    fn take_as_str_with_u32_le_prefix(&mut self) -> Result<&str, BadPos> {
        let out = self.take_with_u32_le_prefix()?;
        let out = std::str::from_utf8(out).map_err(|_| BadPos)?;
        Ok(out)
    }
}

impl BytesExt for &[u8] {}
impl BytesExt for &mut [u8] {}
impl<B: AsRef<[u8]>> BytesExt for Look<u8, B> {}