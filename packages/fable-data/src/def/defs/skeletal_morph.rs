use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SkeletalMorphDef {
    #[def("SkeletalMorphs")]
    pub skeletal_morphs: Vec<DefString>,
}
