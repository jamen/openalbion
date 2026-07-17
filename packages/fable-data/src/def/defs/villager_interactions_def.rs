use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `VILLAGER_INTERACTION` — C++ `CVillagerInteractionsDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct VillagerInteractionsDef {
        "Animation1" => pub animation1: DefString,
        "Animation2" => pub animation2: DefString,
        "Distance" => pub distance: f32,
    }
}
