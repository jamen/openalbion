use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAnimatingObjectDef` — C++ `CAnimatingObjectDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AnimatingObjectDef {
        "Animation" => pub animation: AnimationSet,
    }
}
