use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `PLAYER` — C++ `CPlayerDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlayerDef {
        "CharacterDef" => pub character_def: i32,
        "Colour" => pub colour: RGBColour,
    }
}
