use crate::DefStruct;
use crate::def::prelude::*;

/// `CAnimatingObjectDef` — C++ `CAnimatingObjectDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AnimatingObjectDef {
    #[def("Animation")]
    pub animation: AnimationSet,
}
