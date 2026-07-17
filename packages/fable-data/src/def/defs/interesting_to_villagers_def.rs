use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CInterestingToVillagersDef` — C++ `CInterestingToVillagersDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct InterestingToVillagersDef {
        "Interest" => pub interest: f32,
    }
}
