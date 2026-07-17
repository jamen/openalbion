use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSummonableCreatureDef` — C++ `CSummonableCreatureDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SummonableCreatureDef {
        "Rank" => pub rank: f32,
        "SummonedBrain" => pub summoned_brain: DefString,
    }
}
