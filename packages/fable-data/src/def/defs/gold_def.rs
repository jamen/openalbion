use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CGoldDef` — C++ `CGoldDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GoldDef {
        "Gold" => pub gold: i32,
    }
}
