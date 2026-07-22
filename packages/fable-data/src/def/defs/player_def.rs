use crate::DefStruct;
use crate::def::prelude::*;

/// `PLAYER` — C++ `CPlayerDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct PlayerDef {
    #[def("CharacterDef")]
    pub character_def: DefIndex,
    #[def("Colour")]
    pub colour: RGBColour,
}
