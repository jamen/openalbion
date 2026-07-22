use crate::DefStruct;
use crate::def::prelude::*;

/// `OBJECT_FAMILY` — C++ `CObjectFamilyDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ObjectFamilyDef {
    #[def("Objects")]
    pub objects: Vec<ObjectFamilyEntry>,
}
