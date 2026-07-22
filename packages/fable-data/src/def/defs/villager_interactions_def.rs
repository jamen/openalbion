use crate::DefStruct;
use crate::def::prelude::*;

/// `VILLAGER_INTERACTION` — C++ `CVillagerInteractionsDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct VillagerInteractionsDef {
    #[def("Animation1")]
    pub animation1: DefString,
    #[def("Animation2")]
    pub animation2: DefString,
    #[def("Distance")]
    pub distance: f32,
}
