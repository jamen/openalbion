use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTavernDef` — C++ `CTavernDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TavernDef {
        "BedCost" => pub bed_cost: DefIndex,
    }
}
