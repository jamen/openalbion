use crate::DefStruct;

/// `CFireballSpellLevelDef` — C++ `CFireballSpellLevelDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct FireballSpellLevelDef {
    #[def("Level")]
    pub level: i32,
}
