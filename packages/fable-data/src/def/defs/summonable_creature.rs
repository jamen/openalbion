use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SummonableCreatureDef {
    #[def("Rank")]
    pub rank: f32,
    #[def("SummonedBrain")]
    pub summoned_brain: DefString,
}
