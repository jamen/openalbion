use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CTCNoiseDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct TCNoiseDef {
        "Type" => pub type_: NoiseType,
        "LifeTime" => pub life_time: f32,
        "DistanceCanBeHeardFrom" => pub distance_can_be_heard_from: f32,
    }
}
