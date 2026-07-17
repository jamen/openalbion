use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTextureReplacementDef` — C++ `CTextureReplacementDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TextureReplacementDef {
        "Entries" => pub entries: Vec<TextureReplacementEntry>,
    }
}
