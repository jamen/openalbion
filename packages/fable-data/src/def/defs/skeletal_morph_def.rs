use crate::DefStruct;
use crate::def::prelude::*;

/// `CSkeletalMorphDef` — C++ `CSkeletalMorphDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SkeletalMorphDef {
    #[def("SkeletalMorphs")]
    pub skeletal_morphs: Vec<DefString>,
}
