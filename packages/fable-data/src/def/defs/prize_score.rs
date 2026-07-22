use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct PrizeScoreDef {
    #[def("Score")]
    pub score: f32,
    #[def("Mult")]
    pub mult: DefIndex,
}
