use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernTableDef` — C++ `CTavernTableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernTableDef {
        "AddTankards" => pub add_tankards: bool,
        "HighQualityTankards" => pub high_quality_tankards: bool,
    }
}
