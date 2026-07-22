use crate::DefStruct;
use crate::def::{
    enums::IdleStateGroup,
    wire::VecMap,
};

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct HeroSuitDef {
    #[def("SuitParts")]
    pub suit_parts: VecMap<i32, IdleStateGroup>,
}
