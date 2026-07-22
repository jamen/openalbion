use crate::DefStruct;
use crate::def::prelude::*;

/// `CTextureReplacementDef` — C++ `CTextureReplacementDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TextureReplacementDef {
    #[def("Entries")]
    pub entries: Vec<TextureReplacementEntry>,
}
