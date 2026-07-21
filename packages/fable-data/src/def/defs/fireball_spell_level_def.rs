use crate::def_struct;

def_struct! {
    /// `CFireballSpellLevelDef` — C++ `CFireballSpellLevelDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct FireballSpellLevelDef {
        "Level" => pub level: i32,
    }
}
