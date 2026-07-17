use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CShoveHaPennyRegionDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct ShoveHaPennyRegionDef {
        "Start" => pub start: f32,
        "Score" => pub score: f32,
    }
}
