use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `OBJECT_FAMILY` — C++ `CObjectFamilyDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ObjectFamilyDef {
        "Objects" => pub objects: Vec<ObjectFamilyEntry>,
    }
}
