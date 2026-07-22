use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct VillagerInteractionsDef {
    #[def("Animation1")]
    pub animation1: DefString,
    #[def("Animation2")]
    pub animation2: DefString,
    #[def("Distance")]
    pub distance: f32,
}
