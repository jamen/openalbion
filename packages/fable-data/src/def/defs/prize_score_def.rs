use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CPrizeScoreDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct PrizeScoreDef {
        "Score" => pub score: f32,
        "Mult" => pub mult: DefIndex,
    }
}
