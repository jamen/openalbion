use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CDrunkennessDef` — C++ `CDrunkennessDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DrunkennessDef {
        "DrunkennessThresholdMult" => pub drunkenness_threshold_mult: f32,
    }
}
