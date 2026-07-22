use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TavernDef {
    #[def("BedCost")]
    pub bed_cost: DefIndex,
}
