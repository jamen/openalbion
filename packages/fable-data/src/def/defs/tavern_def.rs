use crate::DefStruct;
use crate::def::prelude::*;

/// `CTavernDef` — C++ `CTavernDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TavernDef {
    #[def("BedCost")]
    pub bed_cost: DefIndex,
}
