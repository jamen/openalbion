use crate::DefStruct;
use crate::def::wire::DefIndex;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct PrizeScoreDef {
    #[def("Score")]
    pub score: f32,
    #[def("Mult")]
    pub mult: DefIndex,
}
