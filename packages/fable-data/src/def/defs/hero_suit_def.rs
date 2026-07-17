use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroSuitDef` — C++ `CHeroSuitDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroSuitDef {
        "SuitParts" => pub suit_parts: VecMap<i32, IdleStateGroup>,
    }
}
