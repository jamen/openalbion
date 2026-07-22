use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroSuitDef {
    #[def("SuitParts")]
    pub suit_parts: VecMap<i32, IdleStateGroup>,
}
