use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSkeletalMorphDef` — C++ `CSkeletalMorphDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SkeletalMorphDef {
        "SkeletalMorphs" => pub skeletal_morphs: Vec<DefString>,
    }
}
