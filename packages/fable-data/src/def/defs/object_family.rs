use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ObjectFamilyDef {
    #[def("Objects")]
    pub objects: Vec<ObjectFamilyEntry>,
}
