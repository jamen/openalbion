use crate::DefStruct;
use crate::def::values::AnimationSet;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AnimatingObjectDef {
    #[def("Animation")]
    pub animation: AnimationSet,
}
