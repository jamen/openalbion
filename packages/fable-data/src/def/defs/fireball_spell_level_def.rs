use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CFireballSpellLevelDef` — C++ `CFireballSpellLevelDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FireballSpellLevelDef {
        "Level" => pub level: i32,
    }
}
