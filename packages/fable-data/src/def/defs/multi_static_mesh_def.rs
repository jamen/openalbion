use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CMultiStaticMeshDef` — C++ `CMultiStaticMeshDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MultiStaticMeshDef {
        "Meshes" => pub meshes: Vec<MultiStaticMeshEntryDef>,
    }
}
