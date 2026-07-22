use crate::DefStruct;
use crate::def::values::ObjectFamilyEntry;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ObjectFamilyDef {
    #[def("Objects")]
    pub objects: Vec<ObjectFamilyEntry>,
}
