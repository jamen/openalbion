use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CTCNoiseDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TCNoiseDef {
    #[def("Type")]
    pub type_: NoiseType,
    #[def("LifeTime")]
    pub life_time: f32,
    #[def("DistanceCanBeHeardFrom")]
    pub distance_can_be_heard_from: f32,
}
