use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CPrizeScoreDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct PrizeScoreDef {
    #[def("Score")]
    pub score: f32,
    #[def("Mult")]
    pub mult: DefIndex,
}
